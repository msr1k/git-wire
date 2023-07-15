use std::process::exit;
use clap::Command;

mod sync;
mod check;
mod common;

fn main() {
    let app = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg_required_else_help(true)
        .subcommand_required(true);

    let app = app.subcommand(
        Command::new("sync")
            .about("Synchronizes code depending on a file '.gitwire' definition.")
            // .arg_from_usage("-v, --verbose 'Verbosely output the command result.'")
    );

    let app = app.subcommand(
        Command::new("check")
            .about("Checks if the synchronized code identical to the original.")
            // .arg_from_usage("-v, --verbose 'Verbosely output the command result.'")
    );

    let matches = app.get_matches();

    let result = match matches.subcommand() {
        Some(("sync", _)) => sync::sync(),
        Some(("check", _)) => check::check(),
        _ => {
            std::process::exit(1);
        }
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
