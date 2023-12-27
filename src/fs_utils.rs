use std::{
    fs,
    io::{self, BufWriter, Write},
};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;

use crate::{path_utils::checksum_to_path, HashCode};

pub fn write_compressed(hash_code: HashCode, content: &[u8]) -> io::Result<()> {
    let path = checksum_to_path(&hash_code.encode_hex::<String>());
    let parent_path = path.parent().expect("Missing object top tree node");
    fs::create_dir_all(parent_path)?;

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(ZlibEncoder::new(file, Compression::best()));
    writer.write_all(content)?;

    Ok(())
}
