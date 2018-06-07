use std::path::PathBuf;
use std::{env, fs};

use chrono::{DateTime, Duration, Utc};
use colored::*;
use reqwest::header::{Authorization, Bearer};
use serde_json;
use url::Url;
use url_serde;

use commands::CommandError;

const DEFAULT_ENDPOINT: &'static str = "https://deployc.io/";
// Anytime after 8 hours, the token will try to be refreshed
const REFRESH_THRESHOLD_HOURS: i64 = 8;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(with = "url_serde")]
    pub endpoint: Url,
    pub token: String,
    pub token_issued_at: Option<DateTime<Utc>>,
    pub token_expires_at: Option<DateTime<Utc>>,
}

impl Config {
    pub fn get() -> Result<Config, CommandError> {
        if let Some(dir) = Config::dir() {
            let filepath = dir.join("config.json");
            if filepath.exists() {
                if let Ok(f) = fs::File::open(filepath) {
                    return serde_json::from_reader(f)
                        .map_err(|_| CommandError::with_message("Failed to read config file."));
                }
            }
        }

        let url = if let Ok(u) =
            Url::parse(&env::var("DEPLOYC_ENDPOINT").unwrap_or(DEFAULT_ENDPOINT.to_string()))
        {
            u
        } else {
            return Err(CommandError::with_message("Invalid endpoint."));
        };

        let endpoint = url.join("/api/").unwrap();
        Ok(Config {
            endpoint,
            token: "".to_string(),
            token_expires_at: None,
            token_issued_at: None,
        })
    }

    pub fn dir() -> Option<PathBuf> {
        env::home_dir()
            .or_else(|| env::current_dir().ok())
            .and_then(|dir| {
                if dir.exists() {
                    Some(dir.join(".deployc"))
                } else {
                    None
                }
            })
    }

    fn get_dir(&self) -> Result<PathBuf, CommandError> {
        let dir = match env::home_dir() {
            Some(d) => d,
            None => {
                let d = env::current_dir().map_err(|_| {
                    CommandError::with_message(
                        "Cannot access home or current directory. Config not stored.",
                    )
                })?;
                println!(
                    "{} {}",
                    "warning:".yellow().bold(),
                    "Could not get home directory. Using local directory for user config."
                );
                d
            }
        }.join(".deployc");

        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(&dir) {
                return Err(CommandError::with_message(format!(
                    "Could not create user config directory.\n{}",
                    e
                )));
            }
        }

        Ok(dir)
    }

    fn filepath(&self) -> Result<PathBuf, CommandError> {
        Ok(self.get_dir()?.join("config.json"))
    }

    pub fn store(&self) -> Result<(), CommandError> {
        let filepath = self.filepath()?;
        let f = fs::File::create(filepath)
            .map_err(|_| CommandError::with_message("Could not open config file."))?;
        serde_json::to_writer_pretty(f, self)
            .map_err(|_| CommandError::with_message("Could not write to config file."))
    }

    pub fn clear(&mut self) -> Result<(), CommandError> {
        self.token = "".to_string();

        let filepath = self.filepath()?;
        if filepath.exists() {
            fs::remove_file(filepath)
                .map_err(|_| CommandError::with_message("Failed to delete token."))?;
        }

        Ok(())
    }

    pub fn get_auth_header(&self) -> Authorization<Bearer> {
        Authorization(Bearer {
            token: self.token.to_owned(),
        })
    }

    pub fn token_needs_refresh(&self) -> bool {
        self.token != "" && self.token_issued_at.map_or(false, |iat| {
            iat + Duration::hours(REFRESH_THRESHOLD_HOURS) <= Utc::now()
        })
    }
}
