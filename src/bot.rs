use crate::error::RustyBotError;
use crate::playground::{PlaygroundRequest, PlaygroundResponse, Response};
use regex::Regex;
use slack::{Event, EventHandler, RtmClient};

pub struct RustyBot;

impl RustyBot {
    fn eval_command(&self, command: String) -> Result<Option<String>, RustyBotError> {
        if command.starts_with("!help") {
            match command.to_lowercase().as_str() {
                "docs" => Ok(Some("https://doc.rust-lang.org/".to_owned())),
                "book" => Ok(Some("https://doc.rust-lang.org/book/".to_owned())),
                _ => Ok(None),
            }
        } else {
            Err(RustyBotError::InvalidBotCommand {
                command: command
                    .lines()
                    .next()
                    .expect("Should be at least one word")
                    .to_string(),
            })
        }
    }

    fn eval_code(&self, code: String) -> Result<String, RustyBotError> {
        if code.starts_with("!code") {
            todo!();
        } else if code.starts_with("!eval") {
            todo!();
        } else {
            Err(RustyBotError::InvalidBotCommand {
                command: code
                    .lines()
                    .next()
                    .expect("Should be at least one word")
                    .to_string(),
            })
        }
    }
}

fn has_code(message: &Option<String>) -> Option<String> {
    match message {
        &Some(ref text) => {
            let re = Regex::new(r"!(code|eval)\n```\n?(?s:(?P<code>.*?))\n```")
                .expect("regex should not fail");
            let code_result = match re.captures(&text) {
                Some(capture) => Some(String::from(&capture["code"])),
                _ => None,
            };
            if code_result == Some("".to_string()) {
                return None;
            } else {
                return code_result;
            }
        }
        _ => None,
    }
}

fn has_command(message: &Option<String>) -> Option<String> {
    match message {
        &Some(ref text) => {
            let re = Regex::new(r"!help\n(?P<command>.*?)$").expect("regex should not fail");
            let command_result = match re.captures(&text) {
                Some(capture) => Some(String::from(&capture["command"])),
                _ => None,
            };
            if command_result == Some("".to_string()) {
                return None;
            } else {
                return command_result;
            }
        }
        _ => None,
    }
}
