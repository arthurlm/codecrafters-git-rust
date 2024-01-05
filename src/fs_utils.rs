use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::HashCode;

pub fn write_compressed(hash_code: HashCode, content: &[u8]) -> io::Result<()> {
    write_compressed_at(hash_code, content, ".")
}

pub fn write_compressed_at<P: AsRef<Path>>(
    hash_code: HashCode,
    content: &[u8],
    dst: P,
) -> io::Result<()> {
    let path = dst.as_ref().join(checksum_to_path(&hex::encode(hash_code)));
    let parent_path = path.parent().expect("Missing object top tree node");
    fs::create_dir_all(parent_path)?;

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(ZlibEncoder::new(file, Compression::best()));
    writer.write_all(content)?;

    Ok(())
}

pub fn read_compressed(cs: &str) -> io::Result<impl BufRead> {
    read_compressed_at(cs, ".")
}

pub fn read_compressed_at<P: AsRef<Path>>(cs: &str, src: P) -> io::Result<impl BufRead> {
    let path = src.as_ref().join(checksum_to_path(cs));
    let file = fs::File::open(path)?;
    Ok(BufReader::new(ZlibDecoder::new(file)))
}

fn checksum_to_path(cs: &str) -> PathBuf {
    assert_eq!(cs.len(), 40, "Invalid checksum size");
    PathBuf::from(".git/objects").join(&cs[..2]).join(&cs[2..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_to_path() {
        assert_eq!(
            checksum_to_path("e547aac8945402134e4c0b9bb85ad82361eed68a"),
            PathBuf::from(".git/objects/e5/47aac8945402134e4c0b9bb85ad82361eed68a"),
        );
    }
}
