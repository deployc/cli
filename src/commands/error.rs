use std::convert::From;

use reqwest;

pub struct CommandError {
    pub message: String,
    pub help: String
}

impl CommandError {
    pub fn with_message<S>(msg: S) -> CommandError where S: Into<String> {
        CommandError { message: msg.into(), help: "".to_string() }
    }

    pub fn with_message_and_help<S>(msg: S, help: S) -> CommandError where S: Into<String> {
        CommandError { message: msg.into(), help: help.into() }
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(err: reqwest::Error) -> CommandError {
        CommandError::with_message(format!("{}", err))
    }
}
