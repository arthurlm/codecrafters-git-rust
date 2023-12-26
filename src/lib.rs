use std::io;

mod error;
pub mod path_utils;
pub mod command {
    pub mod cat_file;
    pub mod init;
}

pub use error::*;

pub enum ObjectHeader {
    Blob { len: usize },
}

impl ObjectHeader {
    pub fn read<R: io::BufRead>(input: &mut R) -> Result<Self, GitError> {
        let mut output = Vec::with_capacity(50);
        input.read_until(0, &mut output)?;

        let header_raw = String::from_utf8(output)?;
        let mut header_raw_iter = header_raw.trim_end_matches('\0').split_whitespace();
        let header_type = header_raw_iter
            .next()
            .ok_or(GitError::InvalidObjectHeader("missing header type"))?;

        match header_type {
            "blob" => {
                let len = header_raw_iter
                    .next()
                    .and_then(|x| x.parse().ok())
                    .ok_or(GitError::InvalidObjectHeader("bad header len"))?;

                Ok(Self::Blob { len })
            }
            _ => Err(GitError::InvalidObjectHeader("bad header type")),
        }
    }
}
