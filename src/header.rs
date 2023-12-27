use std::io;

use crate::GitError;

#[derive(Debug, PartialEq, Eq)]
pub enum GitObjectHeader {
    Blob { len: usize },
    Tree { size: usize },
    Commit { size: usize },
}

impl GitObjectHeader {
    pub fn read<R: io::BufRead>(input: &mut R) -> Result<Self, GitError> {
        let mut output = Vec::with_capacity(50);
        input.read_until(0, &mut output)?;

        let header_raw = String::from_utf8(output)?;
        let mut header_raw_iter = header_raw.trim_end_matches('\0').split_whitespace();
        let header_type = header_raw_iter
            .next()
            .ok_or(GitError::InvalidObjectHeader("missing header type"))?;

        let len = header_raw_iter
            .next()
            .and_then(|x| x.parse().ok())
            .ok_or(GitError::InvalidObjectHeader("bad header len"))?;

        match header_type {
            "blob" => Ok(Self::Blob { len }),
            "tree" => Ok(Self::Tree { size: len }),
            "commit" => Ok(Self::Commit { size: len }),
            _ => Err(GitError::InvalidObjectHeader("bad header type")),
        }
    }

    pub fn write<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        match self {
            Self::Blob { len } => {
                write!(output, "blob {len}\0")
            }
            Self::Tree { size } => {
                write!(output, "tree {size}\0")
            }
            Self::Commit { size } => {
                write!(output, "commit {size}\0")
            }
        }
    }
}
