use std::io::{stdout, Write};

use crate::{fs_utils::read_compressed, object::GitObject, GitError};

pub fn run(cs: &str) -> Result<(), GitError> {
    let mut reader = read_compressed(cs)?;
    let object = GitObject::read(&mut reader)?;

    if let GitObject::Blob(content) = object {
        stdout().write_all(&content)?;
    }

    Ok(())
}
