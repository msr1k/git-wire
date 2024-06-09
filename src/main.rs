use std::process::exit;
use clap::{Parser, Subcommand};

mod sync;
mod check;
mod common;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    /// Synchronizes code depending on a file '.gitwire' definition.
    Sync {
        /// Narrow down the scope of the sync command targets by its name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Checks if the synchronized code identical to the original.
    Check{
        /// Narrow down the scope of the check command targets by its name
        #[arg(short, long)]
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Sync{ name } => sync::sync(name),
        Command::Check{ name } => check::check(name),
    };

    match result.as_ref() {
        Ok(true) => println!("Success"),
        Ok(false) => println!("Failure"),
        Err(e) => eprintln!("{}", e),
    }

    match result {
        Ok(true) => exit(0),
        _ => exit(1),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
