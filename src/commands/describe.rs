use serde_json;

use app::App;
use commands::CommandError;

pub struct DescribeCommand;

impl DescribeCommand {
    pub fn run(app: &App) -> Result<(), CommandError> {
        let s = serde_json::to_string_pretty(&app).map_err(|err| {
            CommandError::with_message(format!("Could not read app config: {}", err))
        })?;
        println!("{}", s);
        Ok(())
    }
}
