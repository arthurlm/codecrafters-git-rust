use std::{
    io::{self, Read},
    path::Path,
};

use bytes::{Buf, Bytes};
use flate2::bufread::ZlibDecoder;

use crate::{
    fs_utils::write_compressed_at,
    header::{GitObjectHeader, GitObjectHeaderType},
    object::GitObject,
    GitError,
};

const OBJ_COMMIT: u8 = 1;
const OBJ_TREE: u8 = 2;
const OBJ_BLOB: u8 = 3;
const OBJ_TAG: u8 = 4;
const OBJ_OFS_DELTA: u8 = 6;
const OBJ_REF_DELTA: u8 = 7;

pub fn unpack_into<R, P>(mut reader: R, dst: P) -> Result<(), GitError>
where
    R: io::BufRead,
    P: AsRef<Path> + Clone,
{
    let mut buf = [0_u8; 4];

    // Check magic.
    reader.read_exact(&mut buf)?;
    if buf != [b'P', b'A', b'C', b'K'] {
        return Err(GitError::invalid_content("Invalid magic PACK"));
    }

    // Check version.
    reader.read_exact(&mut buf)?;
    let version = u32::from_be_bytes(buf);
    if version != 2 {
        return Err(GitError::invalid_content("Invalid PACK version"));
    }

    // Get object count.
    reader.read_exact(&mut buf)?;
    let object_count = u32::from_be_bytes(buf);

    // Read object.
    for _object_id in 0..object_count {
        // Read object header.
        let (obj_type, obj_len) = read_object_size(&mut reader)?;

        // Decode object content based on its type.
        match obj_type {
            OBJ_COMMIT => unpack_git_object(
                &mut reader,
                GitObjectHeader {
                    len: obj_len,
                    r#type: GitObjectHeaderType::Commit,
                },
                dst.clone(),
            )?,
            OBJ_TREE => unpack_git_object(
                &mut reader,
                GitObjectHeader {
                    len: obj_len,
                    r#type: GitObjectHeaderType::Tree,
                },
                dst.clone(),
            )?,
            OBJ_BLOB => unpack_git_object(
                &mut reader,
                GitObjectHeader {
                    len: obj_len,
                    r#type: GitObjectHeaderType::Blob,
                },
                dst.clone(),
            )?,
            OBJ_REF_DELTA => {
                // Read base object hash.
                let mut base_object_hash = [0; 20];
                reader.read_exact(&mut base_object_hash)?;

                // Read compressed data.
                let mut data = Vec::with_capacity(obj_len);
                let mut decompress = ZlibDecoder::new(&mut reader);
                decompress.read_to_end(&mut data)?;

                let data = Bytes::from(data);
                println!("data: {data:?}");
            }
            OBJ_OFS_DELTA | OBJ_TAG => {
                unimplemented!("Unimplemented object type decoding: {obj_type}")
            }
            // If we get another value, then there is a bug 🐞 or data are corrupted.
            _ => {
                return Err(GitError::InvalidContent(format!(
                    "Invalid object type received: {obj_type}"
                )))
            }
        }
    }

    Ok(())
}

pub fn read_object_size<R: io::Read>(reader: &mut R) -> io::Result<(u8, usize)> {
    // Read first byte.
    let mut buf = [0_u8; 1];
    reader.read_exact(&mut buf)?;

    // Decode first byte.
    let mut msb = buf[0] & 0b1000_0000;
    let obj_type = (buf[0] & 0b0111_0000) >> 4;
    let mut value = (buf[0] & 0b0000_1111) as usize;

    let mut offset = 4;
    while msb != 0 {
        // Read next byte.
        let mut buf = [0_u8; 1];
        reader.read_exact(&mut buf)?;

        // Read payload.
        msb = buf[0] & 0b1000_0000;

        // Big endian.
        value |= ((buf[0] & 0b0111_1111) as usize) << offset;
        offset += 7;
    }

    Ok((obj_type, value))
}

fn unpack_git_object<R, P>(reader: &mut R, header: GitObjectHeader, dst: P) -> Result<(), GitError>
where
    R: io::BufRead,
    P: AsRef<Path>,
{
    // Read compressed data.
    let mut data = Vec::with_capacity(header.len);
    let mut decompress = ZlibDecoder::new(reader);
    decompress.read_to_end(&mut data)?;

    // Parse object
    let object = GitObject::read_with_header(&mut data.reader(), header)?;

    // Write object to dst.
    let (hash_code, obj_content) = object.to_bytes_vec()?;
    write_compressed_at(hash_code, &obj_content, dst)?;

    Ok(())
}
