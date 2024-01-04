use std::io;

use sha1::{Digest, Sha1};

use crate::{header::GitObjectHeader, GitError, HashCode};

#[derive(Debug, PartialEq, Eq)]
pub enum GitObject {
    Blob(Vec<u8>),
    Tree(Vec<GitTreeItem>),
    Commit {
        tree: HashCode,
        parent: Option<HashCode>,
        message: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct GitTreeItem {
    pub mode: u32,
    pub name: String,
    pub hash_code: HashCode,
}

impl GitObject {
    pub fn read<R: io::BufRead>(input: &mut R) -> Result<Self, GitError> {
        let header = GitObjectHeader::read(input)?;
        Self::read_with_header(input, header)
    }

    pub fn read_with_header<R: io::BufRead>(
        input: &mut R,
        header: GitObjectHeader,
    ) -> Result<Self, GitError> {
        match header {
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
            GitObjectHeader::Commit { .. } => {
                unimplemented!()
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
                use std::io::Write;

                // Write payload first
                let mut content = Vec::new();

                for item in items {
                    write!(content, "{} {}\0", item.mode, item.name)?;
                    content.write_all(&item.hash_code)?;
                }

                // Then, write object
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
            Self::Commit {
                tree,
                parent,
                message,
            } => {
                use std::fmt::Write;

                // Write payload first
                let mut payload = String::with_capacity(512);
                writeln!(payload, "tree {}", hex::encode(tree))?;

                if let Some(parent) = parent {
                    writeln!(payload, "parent {}", hex::encode(parent))?;
                }

                writeln!(
                    payload,
                    "author Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703674545 +0100",
                )?;
                writeln!(
                    payload,
                    "committer Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703675206 +0100",
                )?;

                writeln!(payload)?;
                writeln!(payload, "{message}")?;

                // Then, write object
                let header = GitObjectHeader::Commit {
                    size: payload.len(),
                };
                let mut header_data = Vec::with_capacity(50);
                header.write(&mut header_data)?;

                hasher.update(&header_data);
                output.write_all(&header_data)?;

                hasher.update(&payload);
                output.write_all(payload.as_bytes())?;
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
