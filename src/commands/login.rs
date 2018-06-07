use colored::*;

use api::API;
use cli::prompt_credentials;
use commands::CommandError;
use config::Config;
use token_response::TokenResponse;

pub struct LoginCommand;

impl LoginCommand {
    pub fn run(config: &mut Config) -> Result<(), CommandError> {
        // get username and password
        println!("{}", "Log in to deployc.io".blue().bold());
        let (username, password) = prompt_credentials()?;

        let body = json!({
            "username": username,
            "password": password
        });

        // make request to deployc.io
        let TokenResponse {
            token,
            issued_at,
            expires_at,
        } = API::new(config).login().post(&body)?;
        config.token = token;
        config.token_issued_at = issued_at;
        config.token_expires_at = expires_at;
        println!("{}", "Logged in!".green().bold());
        Ok(())
    }
}
