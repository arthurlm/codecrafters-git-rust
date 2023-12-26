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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Init => {
            command::init::run()?;
            println!("Initialized git directory");
            Ok(())
        }
    }
}
