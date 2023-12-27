use std::io::{self, Write};

use sha1::{Digest, Sha1};

use crate::{header::GitObjectHeader, GitError, HashCode};

#[derive(Debug, PartialEq, Eq)]
pub enum GitObject {
    Blob(Vec<u8>),
    Tree(Vec<GitTreeItem>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct GitTreeItem {
    pub mode: u32,
    pub name: String,
    pub hash_code: HashCode,
}

impl GitObject {
    pub fn read<R: io::BufRead>(input: &mut R) -> Result<Self, GitError> {
        match GitObjectHeader::read(input)? {
            GitObjectHeader::Blob { len } => {
                let mut content = vec![0; len];
                input.read_exact(&mut content)?;
                Ok(Self::Blob(content))
            }
            GitObjectHeader::Tree { size } => {
                let mut content = vec![0; size];
                input.read_exact(&mut content)?;

                // NOTE: Maybe there is a nicer way to do bellow code 🤔
                let mut items = Vec::new();
                while !content.is_empty() {
                    let end_offset = content
                        .iter()
                        .position(|x| *x == 0)
                        .ok_or(GitError::InvalidObjectPayload("Missing end byte"))?;

                    // Read mode + name
                    let chunk: Vec<_> = content.drain(..end_offset).collect();

                    let text = std::str::from_utf8(&chunk)?;
                    let mut text_iter = text.split_whitespace();
                    let mode = text_iter
                        .next()
                        .ok_or(GitError::InvalidObjectPayload("Missing tree item mode"))?
                        .parse()
                        .map_err(|_err| GitError::InvalidObjectPayload("Invalid tree item mode"))?;
                    let name = text_iter
                        .next()
                        .ok_or(GitError::InvalidObjectPayload("Missing tree item name"))?
                        .to_string();

                    // Rend end byte + hash code
                    content.drain(..1);

                    let mut hash_code = [0_u8; 20];
                    hash_code.copy_from_slice(content.drain(..20).as_slice());
                    items.push(GitTreeItem {
                        mode,
                        name,
                        hash_code,
                    });
                }

                Ok(Self::Tree(items))
            }
        }
    }

    pub fn write<W: io::Write>(&self, output: &mut W) -> Result<HashCode, GitError> {
        let mut hasher = Sha1::new();

        match self {
            Self::Blob(content) => {
                let header = GitObjectHeader::Blob { len: content.len() };
                let mut header_data = Vec::with_capacity(50);
                header.write(&mut header_data)?;

                hasher.update(&header_data);
                output.write_all(&header_data)?;

                hasher.update(content);
                output.write_all(content)?;
            }
            Self::Tree(items) => {
                let mut content = Vec::new();

                for item in items {
                    write!(content, "{} {}\0", item.mode, item.name)?;
                    content.write_all(&item.hash_code)?;
                }

                let header = GitObjectHeader::Tree {
                    size: content.len(),
                };
                let mut header_data = Vec::with_capacity(50);
                header.write(&mut header_data)?;

                hasher.update(&header_data);
                output.write_all(&header_data)?;

                hasher.update(&content);
                output.write_all(&content)?;
            }
        }

        Ok(hasher.finalize().into())
    }

    pub fn to_bytes_vec(&self) -> Result<(HashCode, Vec<u8>), GitError> {
        let mut bytes = Vec::new();
        let hash_code = self.write(&mut bytes)?;
        Ok((hash_code, bytes))
    }
}
