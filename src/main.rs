use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};
use git_starter_rust::{command, hash_code_text_to_array};

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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Init => {
            command::init::run()?;
            println!("Initialized git directory");
            Ok(())
        }
        SubCommand::CatFile { pretty, name } => {
            if pretty {
                command::cat_file::run(&name)?;
            }
            Ok(())
        }
        SubCommand::HashObject { write, path } => {
            let hash_code = command::hash_object::run(path, write)?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
        SubCommand::LsTree { name, .. } => {
            command::ls_tree::run(&name)?;
            Ok(())
        }
        SubCommand::WriteTree => {
            let hash_code =
                command::write_tree::run(env::current_dir().expect("Missing current dir"))?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
        SubCommand::CommitTree {
            tree,
            parent,
            message,
        } => {
            let hash_code = command::commit_tree::run(
                hash_code_text_to_array(&tree),
                hash_code_text_to_array(&parent),
                &message,
            )?;

            println!("{}", hex::encode(hash_code));
            Ok(())
        }
    }
}
