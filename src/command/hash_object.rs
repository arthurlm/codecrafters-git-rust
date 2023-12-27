use std::{fs, path::Path};

use crate::{fs_utils::write_compressed, object::GitObject, GitError, HashCode};

pub fn run<P: AsRef<Path>>(path: P, write: bool) -> Result<HashCode, GitError> {
    let content = fs::read(path)?;
    let object = GitObject::Blob(content);

    let (hash_code, bytes) = object.to_bytes_vec()?;
    if write {
        write_compressed(hash_code, &bytes)?;
    }

    Ok(hash_code)
}
