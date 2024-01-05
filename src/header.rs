use std::io;

use crate::GitError;

#[derive(Debug, PartialEq, Eq)]
pub struct GitObjectHeader {
    pub len: usize,
    pub r#type: GitObjectHeaderType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GitObjectHeaderType {
    Blob,
    Tree,
    Commit,
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

        let r#type = match header_type {
            "blob" => GitObjectHeaderType::Blob,
            "tree" => GitObjectHeaderType::Tree,
            "commit" => GitObjectHeaderType::Commit,
            _ => return Err(GitError::InvalidObjectHeader("bad header type")),
        };

        Ok(Self { len, r#type })
    }

    pub fn write<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        write!(
            output,
            "{} {}\0",
            match self.r#type {
                GitObjectHeaderType::Blob => "blob",
                GitObjectHeaderType::Tree => "tree",
                GitObjectHeaderType::Commit => "commit",
            },
            self.len
        )
    }
}
