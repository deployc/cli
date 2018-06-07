use std::fs;
use std::iter;

use base64;
use chrono::Local;
use chrono_humanize::HumanTime;
use clap::ArgMatches;
use colored::*;
use rand::distributions;
use rand::{thread_rng, Rng};
use serde_json;
use slug::slugify;

use api::API;
use app::{App, Secret, SecretType};
use cli::{print_table, prompt_credentials, prompt_password};
use commands::CommandError;
use config::Config;

pub struct SecretsCommand;

#[derive(Serialize, Deserialize)]
struct ListResponse {
    secrets: Vec<Secret>,
}

impl SecretsCommand {
    fn list(config: &Config, app: &App) -> Result<(), CommandError> {
        let ListResponse { mut secrets } = API::new(config).app(&app.name).param("secrets").get()?;
        if secrets.is_empty() {
            println!(
                "No secrets. Add a secret using {}.",
                "deployc secret create".blue().bold()
            );
            return Ok(());
        }

        secrets.sort_by(|s1, s2| s2.created_at.cmp(&s1.created_at));
        print_table(
            row![Fbb=>"Name", "Type", "Created"],
            secrets
                .iter()
                .map(|s| {
                    row![
                        s.name,
                        s.ty.as_str(),
                        HumanTime::from(s.created_at.with_timezone(&Local))
                    ]
                })
                .collect(),
        );
        Ok(())
    }

    fn generate(num_bytes: usize) -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|_| rng.sample(distributions::Alphanumeric))
            .take(num_bytes)
            .collect()
    }

    fn read_from_file(filename: String) -> Result<String, CommandError> {
        let filepath = fs::canonicalize(filename)
            .map_err(|err| CommandError::with_message(format!("Could not get filepath: {}", err)))?;
        let bytes = fs::read(filepath)
            .map_err(|err| CommandError::with_message(format!("Cannot read file: {}", err)))?;
        String::from_utf8_lossy(&bytes).parse().map_err(|err| {
            CommandError::with_message(format!("Cannot parse bytes from file: {}", err))
        })
    }

    fn encode(s: &String) -> Result<serde_json::Value, CommandError> {
        let b64_value = base64::encode(s);
        serde_json::to_value(&b64_value).map_err(|err| {
            CommandError::with_message(format!("Could not serialize value: {}", err))
        })
    }

    fn create(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        let name = slugify(value_t!(matches, "name", String).unwrap());
        let ty = value_t!(matches, "type", SecretType).unwrap();
        if matches.is_present("bytes") && !ty.is_raw() {
            return Err(CommandError::with_message(format!(
                "{} cannot be generated randomly.",
                ty.as_str()
            )));
        }

        let value = match ty {
            SecretType::Raw => {
                let raw = if matches.is_present("bytes") {
                    let num_bytes = value_t!(matches, "bytes", usize)
                        .map_err(|_| CommandError::with_message("Invalid number of bytes."))?;
                    SecretsCommand::generate(num_bytes)
                } else if let Ok(filename) = value_t!(matches, "file", String) {
                    SecretsCommand::read_from_file(filename)?
                } else {
                    prompt_password(format!("  {}{} ", name.blue().bold(), ":".blue().bold()))
                        .ok_or_else(|| CommandError::with_message("Invalid value."))?
                };

                SecretsCommand::encode(&raw)?
            }
            SecretType::Certificate => {
                // get the key
                let key_filename = value_t!(matches, "key", String).unwrap();
                let key = SecretsCommand::read_from_file(key_filename)?;

                // get the cert
                let cert_filename = value_t!(matches, "cert", String).unwrap();
                let cert = SecretsCommand::read_from_file(cert_filename)?;

                json!({
                    "tls.key": SecretsCommand::encode(&key)?,
                    "tls.cert": SecretsCommand::encode(&cert)?
                })
            }
            SecretType::Credentials => {
                let (username, password) = prompt_credentials()?;
                json!({
                    "username": SecretsCommand::encode(&username)?,
                    "password": SecretsCommand::encode(&password)?
                })
            }
        };

        let body = json!({
            "name": name,
            "type": ty,
            "value": value
        });

        let Secret { .. } = API::new(config)
            .app(&app.name)
            .param("secrets")
            .post(&body)?;

        println!(
            "{} {}",
            format!("Secret {} created!", name).green().bold(),
            format!(
                "Use secret as env var with {}",
                format!("deployc env set -S MY_SECRET {}", name)
                    .blue()
                    .bold()
            )
        );
        Ok(())
    }

    pub fn run(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        match matches.subcommand() {
            ("create", Some(m)) => SecretsCommand::create(m, config, app),
            ("list", _) | _ => SecretsCommand::list(config, app),
        }
    }
}
