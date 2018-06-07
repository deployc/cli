mod env_var;
mod restart_policy;
mod secret;
mod service;

pub use self::env_var::EnvVar;
pub use self::restart_policy::RestartPolicy;
pub use self::secret::{Secret, SecretType};
pub use self::service::Service;

use std::env;
use std::fs::File;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_json;
use slug::slugify;

use commands::CommandError;

fn is_zero(u: &u16) -> bool {
    u == &0
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    #[serde(default)]
    pub restart: RestartPolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<Service>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub port: u16,
    #[serde(default, skip_serializing)]
    pub tier: String,
}

impl App {
    pub fn get(dir: &PathBuf) -> Result<App, CommandError> {
        let filepath = dir.join("deployc.json");
        if filepath.exists() {
            if let Ok(f) = File::open(filepath) {
                return serde_json::from_reader(f).map_err(|err| {
                    CommandError::with_message(format!("Failed to read config file: {}", err))
                });
            }
        }

        let mut app = App::default();
        app.name = slugify(dir.file_name().unwrap().to_str().unwrap());
        Ok(app)
    }

    pub fn create(name: &String) -> Result<App, CommandError> {
        let mut app = App::default();
        app.name = name.clone();
        app.create_file(true)?;
        Ok(app)
    }

    fn create_file(&self, fail_if_exists: bool) -> Result<(), CommandError> {
        let dir = env::current_dir()
            .map_err(|_| CommandError::with_message("Cannot access current directory."))?;

        let filepath = dir.join("deployc.json");
        if filepath.exists() && fail_if_exists {
            return Err(CommandError::with_message(
                "App config file already exists.",
            ));
        }

        let f = File::create(filepath)
            .map_err(|_| CommandError::with_message("Failed to create config file."))?;

        serde_json::to_writer_pretty(f, self)
            .map_err(|_| CommandError::with_message("Failed writing config to file."))?;

        Ok(())
    }

    pub fn store(&self) -> Result<(), CommandError> {
        self.create_file(false)
    }
}

impl Default for App {
    fn default() -> App {
        App {
            name: "".to_string(),
            created_at: Utc::now(),
            command: vec![],
            restart: RestartPolicy::default(),
            services: vec![],
            port: 0,
            tier: "".to_string(),
        }
    }
}
