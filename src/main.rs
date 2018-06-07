extern crate base64;
extern crate chrono;
extern crate chrono_humanize;
#[macro_use]
extern crate clap;
extern crate colored;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prettytable;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate slug;
extern crate tar;
extern crate termion;
extern crate url;
extern crate url_serde;

mod api;
mod app;
mod cli;
mod commands;
mod config;
mod token_response;

use std::path::PathBuf;

use clap::{AppSettings, Arg, SubCommand};
use colored::*;

use api::API;
use app::App;
use commands::{
    CommandError, CreateCommand, DeleteCommand, DescribeCommand, EnvCommand, ExposeCommand,
    ListCommand, LoginCommand, LogoutCommand, LogsCommand, SecretsCommand, SignupCommand,
    TierCommand, UpCommand,
};
use config::Config;
use token_response::TokenResponse;

fn requires_login(cmd: &str) -> bool {
    match cmd {
        "create" | "list" => true,
        _ => requires_app(cmd),
    }
}

fn requires_app(cmd: &str) -> bool {
    match cmd {
        "delete" | "describe" | "env" | "expose" | "logs" | "secrets" | "tier" | "up" => true,
        _ => false,
    }
}

fn run() -> Result<(), CommandError> {
    let matches = clap::App::new("deployc")
        .version(crate_version!())
        .about("Deploy containers to the cloud")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("app")
                .help("Directory of app. Defaults to current directory.")
                .long("app")
                .short("a")
                .required(false),
        )
        .subcommand(SubCommand::with_name("login").about("Log in to deployc.io"))
        .subcommand(SubCommand::with_name("logout").about("Log out from deployc.io"))
        .subcommand(SubCommand::with_name("signup").about("Sign up for deployc.io"))
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates an app")
                .arg(Arg::with_name("name").required(false).index(1)),
        )
        .subcommand(
            SubCommand::with_name("list")
                .visible_aliases(&["ls", "apps"])
                .about("Lists apps"),
        )
        .subcommand(
            SubCommand::with_name("expose")
                .visible_alias("port")
                .about("Exposes app port.")
                .arg(
                    Arg::with_name("PORT")
                        .help("Port app is listening on. Must be an integer between 1-65535.")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("env")
                .about("Manage environment variables.")
                .subcommand(SubCommand::with_name("list").visible_alias("ls"))
                .subcommand(
                    SubCommand::with_name("set")
                        .arg(
                            Arg::with_name("secret")
                                .long("secret")
                                .short("S")
                                .help("<value> is the name of a secret.")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("key")
                                .help("Variable name.")
                                .required(true)
                                .index(1),
                        )
                        .arg(
                            Arg::with_name("value")
                                .help("Variable value.")
                                .required(true)
                                .index(2),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("secrets")
                .visible_alias("secret")
                .about("Manage secrets")
                .subcommand(
                    SubCommand::with_name("create")
                        .arg(
                            Arg::with_name("bytes")
                                .help("Randomly generate a secret with number of bytes.")
                                .long("generate")
                                .short("g")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("type")
                                .help("Type of secret.")
                                .long("type")
                                .short("t")
                                .required(false)
                                .takes_value(true)
                                .possible_values(&["raw", "certificate", "credentials"])
                                .default_value("raw"),
                        )
                        .arg(
                            Arg::with_name("file")
                                .help("Use value from file.")
                                .long("file")
                                .short("f")
                                .conflicts_with("key")
                                .required(false),
                        )
                        .arg(
                            Arg::with_name("key")
                                .help("Use private key from file. For certificates only.")
                                .long("key")
                                .short("k")
                                .required_if("type", "certificate")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("cert")
                                .help("Use public key cert from file. For certificates only.")
                                .long("cert")
                                .short("c")
                                .required_if("type", "certificate")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("name")
                                .help("Name of secret.")
                                .required(true)
                                .index(1),
                        ),
                )
                .subcommand(SubCommand::with_name("list").visible_alias("ls")),
        )
        .subcommand(
            SubCommand::with_name("tier")
                .about("Manage tier.")
                .subcommand(SubCommand::with_name("upgrade").about("Upgrade tier.")),
        )
        .subcommand(SubCommand::with_name("up").about("Deploy app."))
        .subcommand(
            SubCommand::with_name("delete").about("Delete an app.").arg(
                Arg::with_name("force")
                    .long("force")
                    .short("f")
                    .help("Delete app forcefully, without verification. DANGER!")
                    .required(false),
            ),
        )
        .subcommand(SubCommand::with_name("describe").about("Show the app config."))
        .subcommand(
            SubCommand::with_name("logs")
                .about("View app logs.")
                .arg(
                    Arg::with_name("follow")
                        .help("Follow logs.")
                        .long("follow")
                        .short("f")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("tail")
                        .help("Tail number of lines.")
                        .long("tail")
                        .takes_value(true),
                ),
        )
        .get_matches();

    // get config
    let mut config = Config::get()?;

    // run subcommands
    let (subcmd, maybe_submatches) = matches.subcommand();
    if requires_login(subcmd) {
        if config.token == "" {
            return Err(CommandError::with_message_and_help(
                format!("Not logged in. Log in required for {}", subcmd.bold()),
                format!("{} {}", "Run".dimmed(), "deployc login".bold()),
            ));
        } else if config.token_needs_refresh() {
            // Note: fail silently here as to not scare the user. Might want to change this in the future (?)
            if let Ok(TokenResponse {
                token,
                issued_at,
                expires_at,
            }) = API::new(&config).refresh().get()
            {
                config.token = token;
                config.token_issued_at = issued_at;
                config.token_expires_at = expires_at;
            }
        }
    }

    let app_dir = matches.value_of("app").map_or_else(
        || {
            std::env::current_dir().map_err(|_| {
                CommandError::with_message_and_help(
                    "Could not access current directory.",
                    "Possible permissions problem.",
                )
            })
        },
        |a| Ok(PathBuf::from(a)),
    )?;
    let app = if requires_app(subcmd) {
        Some(App::get(&app_dir)?)
    } else {
        None
    };

    let result = match (subcmd, maybe_submatches) {
        ("login", _) => LoginCommand::run(&mut config),
        ("logout", _) => LogoutCommand::run(&mut config),
        ("signup", _) => SignupCommand::run(&mut config),
        ("list", _) => ListCommand::run(&config),
        ("create", Some(m)) => CreateCommand::run(m, &config),
        ("up", Some(m)) => UpCommand::run(m, &config, &app.unwrap()),
        ("describe", _) => DescribeCommand::run(&app.unwrap()),
        ("env", Some(m)) => EnvCommand::run(m, &config, &app.unwrap()),
        ("secrets", Some(m)) => SecretsCommand::run(m, &config, &app.unwrap()),
        ("expose", Some(m)) => ExposeCommand::run(m, &mut app.unwrap()),
        ("delete", Some(m)) => DeleteCommand::run(m, &config, &app.unwrap()),
        ("logs", Some(m)) => LogsCommand::run(m, &config, &app.unwrap()),
        ("tier", Some(m)) => TierCommand::run(m, &config, &app.unwrap()),
        _ => Ok(()),
    };

    if config.token != "" {
        config.store()?;
    }

    result
}

fn main() {
    if let Err(e) = run() {
        println!("{} {}", "error:".bold().red(), e.message);
        if e.help != "" {
            println!("{} {}", "help:".bold().blue(), e.help);
        }
    }
}
