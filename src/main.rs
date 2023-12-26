use clap::{Parser, Subcommand};
use git_starter_rust::command;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    Init,
    CatFile {
        #[arg(short, long)]
        pretty: bool,
        name: String,
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
    }
}
