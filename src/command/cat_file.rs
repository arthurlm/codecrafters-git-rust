use std::{
    fs,
    io::{stdout, BufReader, Write},
};

use flate2::read::ZlibDecoder;

use crate::{object::GitObject, path_utils::checksum_to_path, GitError};

pub fn run(cs: &str) -> Result<(), GitError> {
    let path = checksum_to_path(cs);
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(ZlibDecoder::new(file));

    let object = GitObject::read(&mut reader)?;

    match object {
        GitObject::Blob(content) => {
            stdout().write_all(&content)?;
        }
    }

    Ok(())
}
