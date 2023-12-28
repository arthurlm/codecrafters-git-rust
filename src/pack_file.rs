use std::io::{self, Read};

use bytes::{Buf, Bytes};
use flate2::bufread::ZlibDecoder;

use crate::{header::GitObjectHeader, object::GitObject, GitError};

#[derive(Debug, PartialEq, Eq)]
pub struct PackFile {}

impl PackFile {
    const OBJ_COMMIT: u8 = 1;
    const OBJ_TREE: u8 = 2;
    const OBJ_BLOB: u8 = 3;
    const OBJ_TAG: u8 = 4;
    const OBJ_OFS_DELTA: u8 = 6;
    const OBJ_REF_DELTA: u8 = 7;

    pub fn read<R: io::BufRead>(mut reader: R) -> Result<(), GitError> {
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
            let object = match obj_type {
                Self::OBJ_COMMIT => {
                    read_git_object(&mut reader, GitObjectHeader::Commit { size: obj_len })?
                }
                Self::OBJ_TREE => {
                    read_git_object(&mut reader, GitObjectHeader::Tree { size: obj_len })?
                }
                Self::OBJ_BLOB => {
                    read_git_object(&mut reader, GitObjectHeader::Blob { len: obj_len })?
                }
                Self::OBJ_REF_DELTA => {
                    // Read base object hash.
                    let mut base_object_hash = [0; 20];
                    reader.read_exact(&mut base_object_hash)?;

                    // Read compressed data.
                    let mut data = Vec::with_capacity(obj_len);
                    let mut decompress = ZlibDecoder::new(&mut reader);
                    decompress.read_to_end(&mut data)?;

                    let data = Bytes::from(data);
                    println!("data: {data:?}");
                    GitObject::Blob(Bytes::new())
                }
                Self::OBJ_OFS_DELTA | Self::OBJ_TAG => {
                    unimplemented!("Unimplemented object type decoding: {obj_type}")
                }
                // If we get another value, then there is a bug 🐞 or data are corrupted.
                _ => {
                    return Err(GitError::InvalidContent(format!(
                        "Invalid object type received: {obj_type}"
                    )))
                }
            };

            println!("object: {object:?}");
        }

        Ok(())
    }
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

fn read_git_object<R: io::BufRead>(
    reader: &mut R,
    header: GitObjectHeader,
) -> Result<GitObject, GitError> {
    // Read compressed data.
    let mut data = Vec::with_capacity(header.len());
    let mut decompress = ZlibDecoder::new(reader);
    decompress.read_to_end(&mut data)?;

    // Parse object
    let object = GitObject::read_with_header(&mut data.reader(), header)?;

    Ok(object)
}
