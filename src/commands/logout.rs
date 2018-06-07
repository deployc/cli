use colored::*;

use commands::CommandError;
use config::Config;

pub struct LogoutCommand;

impl LogoutCommand {
    pub fn run(config: &mut Config) -> Result<(), CommandError> {
        config.clear()?;
        println!("{}", "Logged out.".green().bold());
        Ok(())
    }
}
