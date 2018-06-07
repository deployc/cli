use clap::ArgMatches;
use colored::*;
use slug::slugify;
use std::env;

use api::API;
use app::App;
use commands::common::check_card;
use commands::CommandError;
use config::Config;

#[derive(Serialize, Deserialize)]
struct CreateResponse {
    name: String,
    tier: String,
}

pub struct CreateCommand;

impl CreateCommand {
    fn get_name(matches: &ArgMatches) -> Result<String, CommandError> {
        let name = if let Some(n) = matches.value_of("name") {
            n.to_string()
        } else {
            // get name of directory
            env::current_dir()
                .map_err(|_| CommandError::with_message("Cannot access current directory."))?
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        };

        Ok(slugify(name))
    }

    pub fn run(matches: &ArgMatches, config: &Config) -> Result<(), CommandError> {
        check_card(config)?;

        let name = CreateCommand::get_name(matches)?;
        let body = json!({ "name": name });

        // make request to deployc.io
        let CreateResponse { tier, .. } = API::new(config).apps().post(&body)?;
        println!("{} {}!", "Created app".green().bold(), name.bold());
        println!(
            "Your pricing tier is {}. Run {} to view your current tier or {} upgrade.",
            tier.bold(),
            "deployc tier".blue().bold(),
            "deployc tier upgrade".blue().bold()
        );

        App::create(&name)?;
        Ok(())
    }
}
