use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use crate::{path_utils::checksum_to_path, HashCode};

pub fn write_compressed(hash_code: HashCode, content: &[u8]) -> io::Result<()> {
    let path = checksum_to_path(&hex::encode(hash_code));
    let parent_path = path.parent().expect("Missing object top tree node");
    fs::create_dir_all(parent_path)?;

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(ZlibEncoder::new(file, Compression::best()));
    writer.write_all(content)?;

    Ok(())
}

pub fn read_compressed(cs: &str) -> io::Result<impl BufRead> {
    let path = checksum_to_path(cs);
    let file = fs::File::open(path)?;
    Ok(BufReader::new(ZlibDecoder::new(file)))
}
