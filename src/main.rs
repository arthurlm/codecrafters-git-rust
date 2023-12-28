use std::{
    env, fs,
    io::{self, stdout, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use git_starter_rust::{
    clone::clone,
    fs_utils::{read_compressed, write_compressed},
    hash_code_text_to_array,
    object::{GitObject, GitTreeItem},
    GitError, HashCode,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Init empty repository.
    Init,
    /// Read a file from object store.
    CatFile {
        /// Print blob as pretty text.
        #[arg(short, long)]
        pretty: bool,

        /// Blob name.
        name: String,
    },
    /// Hash a file and save it to object store.
    HashObject {
        /// Save Blob to object store.
        #[arg(short, long)]
        write: bool,

        /// Input file name.
        path: PathBuf,
    },
    /// List tree object content.
    LsTree {
        /// Show only file name.
        #[arg(long)]
        name_only: bool,

        /// Blob name.
        name: String,
    },
    /// Write current dir as a object tree.
    WriteTree,
    /// Create new commit from scratch.
    CommitTree {
        /// Tree object ID.
        tree: String,

        /// Parent object ID.
        #[arg(short, long)]
        parent: String,

        /// Commit message.
        #[arg(short, long)]
        message: String,
    },
    /// Clone a repository.
    Clone {
        /// Source URL.
        url: String,

        /// Repo path
        dst: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Init => {
            command_init()?;
            println!("Initialized git directory");
            Ok(())
        }
        SubCommand::CatFile { pretty, name } => {
            if pretty {
                command_cat_file(&name)?;
            }
            Ok(())
        }
        SubCommand::HashObject { write, path } => {
            let hash_code = command_hash_object(path, write)?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
        SubCommand::LsTree { name, .. } => {
            command_ls_tree(&name)?;
            Ok(())
        }
        SubCommand::WriteTree => {
            let hash_code = command_write_tree(env::current_dir().expect("Missing current dir"))?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
        SubCommand::CommitTree {
            tree,
            parent,
            message,
        } => {
            let hash_code = command_commit_tree(
                hash_code_text_to_array(&tree),
                hash_code_text_to_array(&parent),
                &message,
            )?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
        SubCommand::Clone { url, .. } => {
            clone(&url).await?;
            Ok(())
        }
    }
}

pub fn command_init() -> io::Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    Ok(())
}

pub fn command_cat_file(cs: &str) -> Result<(), GitError> {
    let mut reader = read_compressed(cs)?;
    let object = GitObject::read(&mut reader)?;

    if let GitObject::Blob(content) = object {
        stdout().write_all(&content)?;
    }

    Ok(())
}

pub fn command_hash_object<P: AsRef<Path>>(path: P, write: bool) -> Result<HashCode, GitError> {
    let content = fs::read(path)?;
    let object = GitObject::Blob(content);

    let (hash_code, bytes) = object.to_bytes_vec()?;
    if write {
        write_compressed(hash_code, &bytes)?;
    }

    Ok(hash_code)
}

pub fn command_ls_tree(cs: &str) -> Result<(), GitError> {
    let mut reader = read_compressed(cs)?;
    let object = GitObject::read(&mut reader)?;

    if let GitObject::Tree(items) = object {
        for item in items {
            println!("{}", item.name);
        }
    }

    Ok(())
}

pub fn command_write_tree<P: AsRef<Path>>(path: P) -> Result<HashCode, GitError> {
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
            let hash_code = command_hash_object(dir_entry.path(), true)?;

            items.push(GitTreeItem {
                mode: 100644,
                name: dir_entry.file_name().to_string_lossy().to_string(),
                hash_code,
            });
        }

        if file_type.is_dir() {
            let hash_code = command_write_tree(dir_entry.path())?;

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

pub fn command_commit_tree(
    tree: HashCode,
    parent: HashCode,
    message: &str,
) -> Result<[u8; 20], GitError> {
    // Build git object
    let object = GitObject::Commit {
        tree,
        parent: Some(parent),
        author: None,
        committer: None,
        message: message.to_string(),
    };

    // Save to disk
    let (hash_code, bytes) = object.to_bytes_vec()?;
    write_compressed(hash_code, &bytes)?;
    Ok(hash_code)
}
