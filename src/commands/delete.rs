use clap::ArgMatches;
use colored::*;

use api::API;
use app::App;
use cli::prompt;
use commands::CommandError;
use config::Config;

pub struct DeleteCommand;

#[derive(Serialize, Deserialize)]
struct DeleteResponse {
    message: String,
}

impl DeleteCommand {
    pub fn run(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        if !matches.is_present("force") {
            println!(
                "{}",
                format!("Are you sure you want to delete {}?", app.name)
                    .red()
                    .bold()
            );
            match prompt(format!(
                "Type the name of the app to confirm {}: ",
                format!("({})", app.name).dimmed().bold()
            )) {
                Some(ref app_name) if app_name.trim() == &app.name => println!("Deleting app..."),
                _ => return Err(CommandError::with_message("Not deleting app.")),
            }
        }

        let DeleteResponse { .. } = API::new(config).app(&app.name).delete()?;
        println!("App deleted.");
        Ok(())
    }
}
