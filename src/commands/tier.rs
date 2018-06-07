use std::fmt;

use clap::ArgMatches;
use colored::*;

use api::API;
use app::App;
use cli;
use commands::common::check_card;
use commands::CommandError;
use config::Config;

#[derive(Serialize, Deserialize)]
struct Tier {
    name: String,
    pricing: String,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.pricing)
    }
}

impl PartialEq for Tier {
    fn eq(&self, other: &Tier) -> bool {
        self.name == other.name
    }
}

#[derive(Serialize, Deserialize)]
struct ListTiersResponse {
    tiers: Vec<Tier>,
}

pub struct TierCommand;

impl TierCommand {
    pub fn run(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        let current_tier = API::new(config).app(&app.name).param("tier").get()?;
        if let Some(_) = matches.subcommand_matches("upgrade") {
            check_card(config)?;

            let ListTiersResponse { tiers } = API::new(config).tiers().get()?;
            println!(
                "Choose a tier. {}",
                "(ESC or Ctrl-C to cancel)".dimmed().bold()
            );
            if let Some(choice) = cli::get_option(&tiers, Some(current_tier))? {
                println!("Chose: {}", choice.name);
            } else {
                println!("{} Tier unchanged.", "note:".cyan().bold());
            }
        } else {
            let Tier { name, pricing } = current_tier;
            println!("{} {}", "Tier:".bold(), name);
            println!("{} {}", "Pricing:".bold(), pricing);
        }

        Ok(())
    }
}
