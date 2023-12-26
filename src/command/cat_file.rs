use std::{
    fs,
    io::{stdout, BufReader, Read, Write},
    path::PathBuf,
};

use flate2::read::ZlibDecoder;

use crate::{header::ObjectHeader, path_utils::checksum_to_path, GitError};

pub fn run(cs: &str) -> Result<(), GitError> {
    let path = PathBuf::from(".git/objects").join(checksum_to_path(cs));
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(ZlibDecoder::new(file));

    let header = ObjectHeader::read(&mut reader)?;

    let mut content = match header {
        ObjectHeader::Blob { len } => vec![0; len],
    };
    reader.read_exact(&mut content)?;
    stdout().write_all(&content)?;

    Ok(())
}
