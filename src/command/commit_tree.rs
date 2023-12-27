use crate::{fs_utils::write_compressed, object::GitObject, GitError, HashCode};

pub fn run(tree: HashCode, parent: HashCode, message: &str) -> Result<[u8; 20], GitError> {
    // Build git object
    let object = GitObject::Commit {
        tree,
        parent: Some(parent),
        message: message.to_string(),
    };

    // Save to disk
    let (hash_code, bytes) = object.to_bytes_vec()?;
    write_compressed(hash_code, &bytes)?;
    Ok(hash_code)
}
