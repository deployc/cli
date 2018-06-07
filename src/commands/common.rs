use colored::*;

use api::API;
use commands::CommandError;
use config::Config;

#[derive(Serialize, Deserialize)]
struct HasCardResponse {
    exists: bool,
}

pub fn check_card(config: &Config) -> Result<(), CommandError> {
    // check that user has a valid credit card
    let HasCardResponse { exists } = API::new(config).has_card().get()?;
    if !exists {
        return Err(CommandError::with_message_and_help(
            "Missing payment method.",
            &format!(
                "Visit {} to enter your payment information.",
                "https://deployc.io".underline()
            ),
        ));
    }

    Ok(())
}
