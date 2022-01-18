use crate::error::RustyBotError;
use crate::playground::{PlaygroundAnswer, PlaygroundRequest};
use crate::slack_conn::CodeReplyTemplate;
use html_escape::decode_html_entities;
use regex::Regex;
use slack_morphism::prelude::*;
use slack_morphism_hyper::*;
use std::env;
use std::sync::Arc;

pub async fn on_message(
    event: SlackPushEventCallback,
    client: Arc<SlackHyperClient>,
    _states: Arc<SlackClientEventsUserState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::spawn(async move { process_message(client, event).await });
    Ok(())
}

async fn process_message(
    client: Arc<SlackHyperClient>,
    event: SlackPushEventCallback,
) -> Result<(), RustyBotError> {
    let token_value =
        SlackApiTokenValue(env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN env var not found"));
    let token = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    match event.event {
        SlackEventCallbackBody::Message(msg_event) => {
            println!("Matched message");
            let channel = msg_event.origin.channel;
            let content = msg_event.content;
            if let Some(channel_id) = channel {
                if let Some(msg_content) = content {
                    let text = msg_content.text;
                    // start matching the has_ functions
                    // code
                    if let Some(code) = has_code(&text) {
                        println!("Has code: {:?}", code);
                        let response = eval_code(code)
                            .await
                            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
                        println!("Evaled code: {:?}", response);
                        let reply_content = CodeReplyTemplate::new(
                            &response.link,
                            response.playground_answer.stdout,
                            response.playground_answer.stderr,
                        );
                        let reply_request = SlackApiChatPostMessageRequest::new(
                            channel_id,
                            reply_content.render_template(),
                        );
                        let _response = session.chat_post_message(&reply_request).await;
                        println!("{:?}", _response);
                        return Ok(());
                    }
                    // command
                    else if let Some(command) = has_command(&text) {
                        if let Some(output) = eval_command(command)? {
                            let reply_content = SlackMessageContent::new().with_text(output);
                            let reply_request =
                                SlackApiChatPostMessageRequest::new(channel_id, reply_content);
                            let _response = session.chat_post_message(&reply_request).await;
                        }
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                }
                return Ok(());
            }
            Ok(())
        }
        SlackEventCallbackBody::AppMention(mention_event) => {
            let channel_id = mention_event.channel;
            let reply_content =
                SlackMessageContent::new().with_text("I'm alive, don't worry".to_string());
            let reply_request = SlackApiChatPostMessageRequest::new(channel_id, reply_content);
            let _response = session.chat_post_message(&reply_request).await;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn eval_command(command: String) -> Result<Option<String>, RustyBotError> {
    match command.to_lowercase().as_str() {
        "docs" => Ok(Some("https://doc.rust-lang.org/".to_owned())),
        "book" => Ok(Some("https://doc.rust-lang.org/book/".to_owned())),
        _ => Ok(Some("*Available commands*\n!code - for complete code blocks\n!eval - for evaluating chunks that can fit in main function\n!help [docs, book] - links to classic rust material\n_Yours truely, Ferris_".to_owned())),
    }
}

async fn eval_code(code: Code) -> Result<PlaygroundAnswer, RustyBotError> {
    let request;
    if code.kind == "code".to_string() {
        request = PlaygroundRequest::new(decode_html_entities(&code.text).as_ref().to_string());
    } else if code.kind == "eval".to_string() {
        request =
            PlaygroundRequest::new_eval(decode_html_entities(&code.text).as_ref().to_string());
    } else {
        return Err(RustyBotError::InvalidBotCommand {
            command: code.kind.to_string(),
        });
    };
    let result = request.execute().await;
    match result {
        Ok(res) => {
            let ans = PlaygroundAnswer {
                playground_answer: res.playground_response,
                link: request.create_share_link().await?,
            };
            Ok(ans)
        }
        Err(e) => Err(RustyBotError::InternalServerError(e.into())),
    }
}

#[derive(Debug)]
struct Code {
    kind: String,
    text: String,
}

fn has_code(message: &Option<String>) -> Option<Code> {
    match *message {
        Some(ref text) => {
            let re = Regex::new(r"!(?P<kind>code|eval)\n```?(?s:(?P<code>.*?))```")
                .expect("code regex should not fail");
            let code_result = match re.captures(text) {
                Some(capture) => Some(Code {
                    kind: String::from(&capture["kind"]),
                    text: String::from(&capture["code"]),
                }),
                _ => None,
            };
            code_result
        }
        _ => None,
    }
}

fn has_command(message: &Option<String>) -> Option<String> {
    match *message {
        Some(ref text) => {
            let re =
                Regex::new(r"!help\s(?P<command>.*?)$").expect("command regex should not fail");
            let command_result = match re.captures(text) {
                Some(capture) => Some(String::from(&capture["command"])),
                _ => None,
            };
            if command_result == Some("".to_string()) {
                None
            } else {
                command_result
            }
        }
        _ => None,
    }
}
