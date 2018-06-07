use std::collections::HashMap;

use clap::ArgMatches;
use colored::*;

use api::API;
use app::{App, EnvVar};
use cli::print_table;
use commands::CommandError;
use config::Config;

pub struct EnvCommand;

#[derive(Serialize, Deserialize)]
struct EnvResponse {
    env: HashMap<String, EnvVar>,
}

impl EnvCommand {
    fn print_env(env: &HashMap<String, EnvVar>) {
        print_table(
            row![Fbb => "Name", "Value"],
            env.iter()
                .map(|(k, v)| {
                    row![
                        k,
                        match v {
                            EnvVar::Value(val) => format!("{}", val),
                            EnvVar::Secret(s) => format!("secret: {}", s),
                        }
                    ]
                })
                .collect(),
        );
    }

    fn list(config: &Config, app: &App) -> Result<(), CommandError> {
        let EnvResponse { env } = API::new(config).app(&app.name).param("env").get()?;
        if env.is_empty() {
            println!(
                "No environment variables. Create one using {}.",
                "deployc env set".blue().bold()
            );
            return Ok(());
        }

        EnvCommand::print_env(&env);
        Ok(())
    }

    fn set(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        let from_secret = matches.is_present("secret");
        let key = value_t!(matches, "key", String).unwrap();
        let value = value_t!(matches, "value", String).unwrap();
        let var = if from_secret {
            EnvVar::Secret(value.clone())
        } else {
            EnvVar::Value(value.clone())
        };

        let body = json!({ "key": key, "value": var });
        let EnvResponse { .. } = API::new(config).app(&app.name).param("env").post(&body)?;

        let redeploy_text = format!(
            "Run {} to re-deploy with new environment.",
            "deployc up".blue().bold()
        );
        if from_secret {
            println!(
                "Set {} to value of secret {}. {}",
                key.blue().bold(),
                value.blue().bold(),
                redeploy_text
            );
        } else {
            println!(
                "Set {} to {}. {}",
                key.blue().bold(),
                value.blue().bold(),
                redeploy_text
            );
        }
        Ok(())
    }

    pub fn run(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        match matches.subcommand() {
            ("set", Some(m)) => EnvCommand::set(m, config, app),
            ("list", _) | _ => EnvCommand::list(config, app),
        }
    }
}
