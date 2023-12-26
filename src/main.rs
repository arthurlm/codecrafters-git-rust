use std::path::PathBuf;

use clap::{Parser, Subcommand};
use git_starter_rust::command;

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
            command::hash_object::run(path, write)?;
            Ok(())
        }
    }
}
