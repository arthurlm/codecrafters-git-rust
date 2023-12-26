use std::io;

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
}
