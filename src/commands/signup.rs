use chrono::{DateTime, Utc};
use colored::*;
use regex::Regex;

use api::API;
use cli::{prompt, prompt_password};
use commands::CommandError;
use config::Config;

lazy_static! {
    static ref USERNAME_RE: Regex = Regex::new(r"^[\-_a-zA-Z0-9]+$").unwrap();
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignupResponse {
    id: String,
    username: String,
    email: String,
    token: String,
    token_issued_at: Option<DateTime<Utc>>,
    token_expires_at: Option<DateTime<Utc>>,
}

pub struct SignupCommand;

impl SignupCommand {
    fn get_username() -> Result<String, CommandError> {
        let username = prompt("  Username: ".bold())
            .ok_or(CommandError::with_message("Invalid username."))?
            .to_lowercase()
            .trim()
            .to_string();

        // username must be valid
        if !USERNAME_RE.is_match(&username) {
            Err(CommandError::with_message_and_help(
                "Invalid username.",
                "Username may only contain 'a-z', 'A-Z', '0-9', '-' or '_'.",
            ))
        } else {
            Ok(username)
        }
    }

    fn get_email() -> Result<String, CommandError> {
        let email = prompt("     Email: ".bold())
            .ok_or(CommandError::with_message("Invalid email."))?
            .to_lowercase()
            .trim()
            .to_string();

        match email.rfind('@') {
            Some(n) if n != email.len() - 1 && !email.contains(char::is_whitespace) => Ok(email),
            _ => Err(CommandError::with_message("Invalid email.")),
        }
    }

    fn get_password() -> Result<String, CommandError> {
        let password = prompt_password("  Password: ".bold())
            .ok_or(CommandError::with_message("Invalid password."))?
            .trim()
            .to_string();

        Ok(password)
    }

    pub fn run(config: &mut Config) -> Result<(), CommandError> {
        // get username and password
        println!("{}", "Sign up for deployc.io".blue().bold());
        let username = SignupCommand::get_username()?;
        let email = SignupCommand::get_email()?;
        let password = SignupCommand::get_password()?;

        let body = json!({
            "username": username,
            "email": email,
            "password": password
        });

        // make request to deployc.io
        let SignupResponse {
            token,
            token_issued_at,
            token_expires_at,
            ..
        } = API::new(config).signup().post(&body)?;
        config.token = token;
        config.token_issued_at = token_issued_at;
        config.token_expires_at = token_expires_at;

        println!(
            "\n{}\n",
            "You are now signed up for deployc.io!".green().bold()
        );
        println!(
            "Before deploying an app, visit {} to enter your payment information.",
            "https://deployc.io".underline()
        );
        Ok(())
    }
}
