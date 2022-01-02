use std::process::exit;
use clap::{App,SubCommand,AppSettings};

mod sync;
mod check;
mod common;

fn main() {
    let app = App::new("git-wire")
        .version("1.0")
        .author("msr1k <msr0210@gmail.com>")
        .about("Wiring part of other repository's code.")
        .setting(AppSettings::SubcommandRequired);

    let app = app.subcommand(
        SubCommand::with_name("sync")
            .about("Synchronize code depending on a file '.gitwire' definition.")
            // .arg_from_usage("-v, --verbose 'Verbosely output the command result.'")
    );

    let app = app.subcommand(
        SubCommand::with_name("check")
            .about("Check if the synchronized code identical to the original.")
            // .arg_from_usage("-v, --verbose 'Verbosely output the command result.'")
    );

    let matches = app.get_matches();

    let result = match matches.subcommand() {
        ("sync", Some(_)) => sync::sync(),
        ("check", Some(_)) => check::check(),
        _ => {
            eprintln!("{}", matches.usage());
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
