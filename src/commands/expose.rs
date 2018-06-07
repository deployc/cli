use clap::ArgMatches;
use colored::*;

use app::App;
use commands::CommandError;

pub struct ExposeCommand;

impl ExposeCommand {
    pub fn run(matches: &ArgMatches, app: &mut App) -> Result<(), CommandError> {
        let port = value_t!(matches, "PORT", u16).map_err(|_| {
            CommandError::with_message_and_help(
                "Invalid port.".to_string(),
                "Must be an integer between 1-65535.".to_string(),
            )
        })?;

        if port == 0 {
            return Err(CommandError::with_message_and_help(
                "Invalid port.",
                "Port must be non-zero.",
            ));
        }

        app.port = port;
        app.store()?;

        println!(
            "App port {} exposed. Run {} to publish your app.",
            port,
            "deployc up".blue().bold()
        );
        Ok(())
    }
}
