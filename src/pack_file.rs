use std::{
    io::{self, Read},
    path::Path,
};

use bytes::Buf;
use flate2::bufread::ZlibDecoder;

use crate::{
    fs_utils::{read_compressed_at, write_compressed_at},
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
        let (obj_type, obj_len) = read_object_pack_header(&mut reader)?;

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

                // Find base object data from git DB.
                let mut base_reader =
                    read_compressed_at(&hex::encode(base_object_hash), dst.clone())?;

                let mut base_data = Vec::new();
                base_reader.read_to_end(&mut base_data)?;

                // Read compressed data.
                let mut patch_data = Vec::with_capacity(obj_len);
                let mut decompress = ZlibDecoder::new(&mut reader);
                decompress.read_to_end(&mut patch_data)?;

                // Apply patch to rebuild git object.
                let object = apply_patch(&base_data, &patch_data)?;

                // Write object to dst.
                let (hash_code, obj_content) = object.to_bytes_vec()?;
                write_compressed_at(hash_code, &obj_content, dst.clone())?;
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

pub fn read_object_pack_header<R: io::Read>(reader: &mut R) -> io::Result<(u8, usize)> {
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
        msb = buf[0] >> 7;
        value |= ((buf[0] & 0b0111_1111) as usize) << offset;
        offset += 7;
    }

    Ok((obj_type, value))
}

pub fn read_var_int(data: &[u8]) -> Result<(&[u8], usize), GitError> {
    let mut msb = 1;
    let mut index = 0;
    let mut value = 0_usize;

    while msb != 0 {
        // Read next byte.
        let Some(byte) = data.get(index) else {
            return Err(GitError::invalid_content(
                "Missing next byte in var int read",
            ));
        };

        // Read payload.
        msb = byte >> 7;
        value |= ((byte & 0b0111_1111) as usize) << (index * 7);
        index += 1;
    }

    Ok((&data[index..], value))
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

fn apply_patch(base: &[u8], patch: &[u8]) -> Result<GitObject, GitError> {
    // Split header and payload from base data.
    let Some(base_header_offset) = base.iter().position(|x| *x == 0) else {
        return Err(GitError::invalid_content("base miss header"));
    };

    let (base_header, base_payload) = base.split_at(base_header_offset + 1);
    let base_header = GitObjectHeader::read(&mut base_header.reader())?;

    // Read header and init output object.
    let (patch, source_len) = read_var_int(patch)?;
    assert_eq!(
        source_len,
        base_payload.len(),
        "Base len is different from information stored in patch {base:?}"
    );

    let (mut patch, output_len) = read_var_int(patch)?;
    let mut output = Vec::with_capacity(output_len);

    // Loop over instruction and rebuild output object.
    while !patch.is_empty() {
        let (next_patch, instr) = DeltaInstructionType::from_bytes(patch);
        patch = next_patch;

        match instr {
            DeltaInstructionType::Copy { offset, size } => {
                output.extend_from_slice(&base_payload[offset..offset + size]);
            }
            DeltaInstructionType::Insert { size } => {
                let (next_payload, next_patch) = patch.split_at(size);
                patch = next_patch;
                output.extend_from_slice(next_payload);
            }
            DeltaInstructionType::Reserved => {}
        }
    }

    // Build git object from rebuild content + header
    let object = GitObject::read_with_header(
        &mut output.reader(),
        GitObjectHeader {
            len: output_len,
            r#type: base_header.r#type,
        },
    )?;

    Ok(object)
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeltaInstructionType {
    Copy { offset: usize, size: usize },
    Insert { size: usize },
    Reserved,
}

impl DeltaInstructionType {
    pub fn from_bytes(input: &[u8]) -> (&[u8], Self) {
        let instr = input[0];

        // If MSB is set: then it is a copy.
        if instr >> 7 == 1 {
            let mut bytes_read = 1;

            // Read next 4 optional bytes to get offset to copy from.
            let mut offset = 0;
            if instr & 0x01 != 0 {
                offset |= input[bytes_read] as usize;
                bytes_read += 1;
            }
            if instr & 0x02 != 0 {
                offset |= (input[bytes_read] as usize) << 8;
                bytes_read += 1;
            }
            if instr & 0x04 != 0 {
                offset |= (input[bytes_read] as usize) << 16;
                bytes_read += 1;
            }
            if instr & 0x08 != 0 {
                offset |= (input[bytes_read] as usize) << 24;
                bytes_read += 1;
            }

            // Read next 3 optional bytes to get object size.
            let mut size = 0;
            if instr & 0x10 != 0 {
                size |= input[bytes_read] as usize;
                bytes_read += 1;
            }
            if instr & 0x20 != 0 {
                size |= (input[bytes_read] as usize) << 8;
                bytes_read += 1;
            }
            if instr & 0x40 != 0 {
                size |= (input[bytes_read] as usize) << 16;
                bytes_read += 1;
            }

            // If size is 0: then set size to a special value.
            // See: https://github.com/git/git/blob/795ea8776befc95ea2becd8020c7a284677b4161/Documentation/gitformat-pack.txt#L156
            if size == 0 {
                size = 0x10000;
            }

            (&input[bytes_read..], Self::Copy { offset, size })
        } else {
            // Otherwise it is an insert instruction
            let size = (instr & 0b0111_1111) as usize;

            // Handle case of 0 size instruction.
            // See: https://github.com/git/git/blob/795ea8776befc95ea2becd8020c7a284677b4161/Documentation/gitformat-pack.txt#L169
            if size == 0 {
                (&input[1..], Self::Reserved)
            } else {
                (&input[1..], Self::Insert { size })
            }
        }
    }
}
