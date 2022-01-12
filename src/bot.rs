use crate::error::RustyBotError;
use crate::playground::{PlaygroundAnswer, PlaygroundRequest};
use crate::slack::SlackClient;
use regex::Regex;
use slack::{self, Event, EventHandler, RtmClient};
use std::env;

pub struct RustyBot;

impl RustyBot {
    fn on_message(
        &self,
        client: SlackClient,
        message: slack::Message,
    ) -> Result<(), RustyBotError> {
        match message {
            slack::Message::Standard(msg) => {
                if let Some(code) = has_code(&msg.text) {
                    let response = self.eval_code(code)?;
                    let channel_id = msg.channel.expect("channel should not be None");
                    let _outcome = client.send_code_reply(
                        &channel_id,
                        &response.link,
                        response.playground_answer.stdout,
                        response.playground_answer.stderr,
                    );
                    return Ok(());
                } else if let Some(command) = has_command(&msg.text) {
                    if let Some(output) = self.eval_command(command)? {
                        let channel_id = msg.channel.expect("channel should not be None");
                        let _outcome = client.send_reply(&channel_id, output);
                    }
                    return Ok(());
                } else if let Some(output) = has_bot_mention(&msg.text) {
                    let channel_id = msg.channel.expect("channel should not be None");
                    let _outcome = client.send_reply(&channel_id, output);
                };
                return Ok(());
            }
            _ => Ok(()),
        }
    }

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

    fn eval_code(&self, code: String) -> Result<PlaygroundAnswer, RustyBotError> {
        let request;
        if code.starts_with("!code") {
            request = PlaygroundRequest::new(code);
        } else if code.starts_with("!eval") {
            request = PlaygroundRequest::new_eval(code);
        } else {
            return Err(RustyBotError::InvalidBotCommand {
                command: code
                    .lines()
                    .next()
                    .expect("Should be at least one word")
                    .to_string(),
            });
        };
        match request.execute() {
            Ok(res) => {
                let ans = PlaygroundAnswer {
                    playground_answer: res.playground_response,
                    link: request.create_share_link()?,
                };
                return Ok(ans);
            }
            Err(e) => Err(RustyBotError::InternalServerError(e.into())),
        }
    }
}

impl EventHandler for RustyBot {
    fn on_event(&mut self, _cli: &RtmClient, event: Event) {
        println!("on_event(event: {:?})", event); // TODO: replace with logging print statement
        match event {
            Event::Message(content) => {
                let slack_client = SlackClient::init().expect("SlackClient init failed"); // TODO: should send a PR to allow Result in this trait's methods.
                self.on_message(slack_client, *content)
                    .expect("on_message failed");
            }
            _ => (),
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        return;
    }

    fn on_connect(&mut self, _cli: &RtmClient) {
        return;
    }
}

fn has_code(message: &Option<String>) -> Option<String> {
    match message {
        &Some(ref text) => {
            let re = Regex::new(r"!(code|eval)\n```\n?(?s:(?P<code>.*?))\n```")
                .expect("code regex should not fail");
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
            let re =
                Regex::new(r"!help\n(?P<command>.*?)$").expect("command regex should not fail");
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

fn has_bot_mention(message: &Option<String>) -> Option<String> {
    match message {
        Some(text) => {
            let bot_name =
                env::var("SLACK_BOT_NAME").expect("SLACK_BOT_NAME env var was not found");
            let re = Regex::new(r"@(?P<bot>[\w_]+)").expect("bot mention regex should not fail");
            for caps in re.captures_iter(&text) {
                if bot_name == &caps["bot"] {
                    return Some(String::from("I'm alive, don't worry"));
                };
            }
            None
        }
        None => None,
    }
}
