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
    let path = dst.as_ref().join(checksum_to_path(hash_code));
    let parent_path = path.parent().expect("Missing object top tree node");
    fs::create_dir_all(parent_path)?;

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(ZlibEncoder::new(file, Compression::best()));
    writer.write_all(content)?;

    Ok(())
}

pub fn read_compressed(hash_code: HashCode) -> io::Result<impl BufRead> {
    read_compressed_at(hash_code, ".")
}

pub fn read_compressed_at<P: AsRef<Path>>(hash_code: HashCode, src: P) -> io::Result<impl BufRead> {
    let path = src.as_ref().join(checksum_to_path(hash_code));
    let file = fs::File::open(path)?;
    Ok(BufReader::new(ZlibDecoder::new(file)))
}

fn checksum_to_path(hash_code: HashCode) -> PathBuf {
    let cs = hex::encode(hash_code);
    assert_eq!(cs.len(), 40, "Invalid checksum size");
    PathBuf::from(".git/objects").join(&cs[..2]).join(&cs[2..])
}

#[cfg(test)]
mod tests {
    use crate::hash_code_text_to_array;

    use super::*;

    #[test]
    fn test_checksum_to_path() {
        assert_eq!(
            checksum_to_path(hash_code_text_to_array(
                "e547aac8945402134e4c0b9bb85ad82361eed68a"
            )),
            PathBuf::from(".git/objects/e5/47aac8945402134e4c0b9bb85ad82361eed68a"),
        );
    }
}
