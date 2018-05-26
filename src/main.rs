extern crate clap;

use clap::{App, AppSettings, SubCommand};

fn main() {
    let matches = App::new("deployc")
        .version("0.1.0")
        .about("Deploy containers to the cloud")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("login")
            .about("Login to deployc.io")
        )
        .get_matches();

    match matches.subcommand() {
        ("login", Some(_)) => println!("Logging in..."),
        _ => {}
    }
}
