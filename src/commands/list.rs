use chrono::Local;
use chrono_humanize::HumanTime;
use colored::*;

use api::API;
use app::App;
use cli::{ellipsis, print_table};
use commands::CommandError;
use config::Config;

#[derive(Serialize, Deserialize)]
struct ListResponse {
    apps: Vec<App>,
}

pub struct ListCommand;

impl ListCommand {
    pub fn run(config: &Config) -> Result<(), CommandError> {
        // make request to deployc.io
        let ListResponse { mut apps } = API::new(config).apps().get()?;

        if apps.len() == 0 {
            println!("No apps! Use {} to create an app.", "deployc create".bold());
        } else {
            apps.sort_by(|a1, a2| a2.created_at.cmp(&a1.created_at));
            print_table(
                row![Fbb => "Name", "Tier", "Created"],
                apps.iter()
                    .map(|app| {
                        row![
                            ellipsis(&app.name, 18),
                            app.tier,
                            HumanTime::from(app.created_at.with_timezone(&Local))
                        ]
                    })
                    .collect(),
            );
        }

        Ok(())
    }
}
