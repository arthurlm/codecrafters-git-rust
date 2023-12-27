use std::{fs, path::Path};

use crate::{
    command::hash_object,
    fs_utils::write_compressed,
    object::{GitObject, GitTreeItem},
    GitError, HashCode,
};

pub fn run<P: AsRef<Path>>(path: P) -> Result<HashCode, GitError> {
    let mut items = Vec::new();

    // Build tree items
    let mut dir_entries: Vec<_> = fs::read_dir(path)?.filter_map(|x| x.ok()).collect();
    dir_entries.sort_by_key(|x| x.file_name());

    for dir_entry in dir_entries {
        if dir_entry.file_name() == ".git" {
            continue;
        }

        let file_type = dir_entry.file_type()?;

        if file_type.is_file() {
            let hash_code = hash_object::run(dir_entry.path(), true)?;

            items.push(GitTreeItem {
                mode: 100644,
                name: dir_entry.file_name().to_string_lossy().to_string(),
                hash_code,
            });
        }

        if file_type.is_dir() {
            let hash_code = run(dir_entry.path())?;

            items.push(GitTreeItem {
                mode: 40000,
                name: dir_entry.file_name().to_string_lossy().to_string(),
                hash_code,
            });
        }
    }

    // Build git object
    let object = GitObject::Tree(items);

    // Save to disk
    let (hash_code, bytes) = object.to_bytes_vec()?;
    write_compressed(hash_code, &bytes)?;
    Ok(hash_code)
}
