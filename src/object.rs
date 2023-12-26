use std::io;

use sha1::{Digest, Sha1};

use crate::{header::GitObjectHeader, GitError};

#[derive(Debug, PartialEq, Eq)]
pub enum GitObject {
    Blob(Vec<u8>),
}

impl GitObject {
    pub fn read<R: io::BufRead>(input: &mut R) -> Result<Self, GitError> {
        match GitObjectHeader::read(input)? {
            GitObjectHeader::Blob { len } => {
                let mut content = vec![0; len];
                input.read_exact(&mut content)?;
                Ok(Self::Blob(content))
            }
        }
    }

    pub fn write<W: io::Write>(&self, output: &mut W) -> Result<[u8; 20], GitError> {
        let mut hasher = Sha1::new();

        match self {
            GitObject::Blob(content) => {
                let header = GitObjectHeader::Blob { len: content.len() };
                let mut header_data = Vec::with_capacity(50);
                header.write(&mut header_data)?;

                hasher.update(&header_data);
                output.write_all(&header_data)?;

                hasher.update(content);
                output.write_all(content)?;
            }
        }

        Ok(hasher.finalize().into())
    }
}
