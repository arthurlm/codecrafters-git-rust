use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;

use crate::{object::GitObject, path_utils::checksum_to_path, GitError};

pub fn run<P: AsRef<Path>>(path: P, write: bool) -> Result<(), GitError> {
    let content = fs::read(path)?;
    let object = GitObject::Blob(content);

    let mut output = Vec::new();
    let hash_code = object.write(&mut output)?;
    let hash_code: String = hash_code.encode_hex();

    if write {
        let path = checksum_to_path(&hash_code);
        let parent_path = path.parent().expect("Missing object top tree node");
        fs::create_dir_all(parent_path)?;

        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(ZlibEncoder::new(file, Compression::best()));
        writer.write_all(&output)?;
    }

    println!("{hash_code}");
    Ok(())
}
