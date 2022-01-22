use crate::error::RustyBotError;
use crate::playground::{PlaygroundAnswer, PlaygroundRequest};
use crate::slack_conn::CodeReplyTemplate;
use regex::Regex;
use slack_morphism::prelude::*;
use slack_morphism_hyper::*;
use std::env;
use std::sync::Arc;
use tracing::{debug, error, instrument};
use uuid::Uuid;

pub async fn on_message(
    event: SlackPushEventCallback,
    client: Arc<SlackHyperClient>,
    _states: Arc<SlackClientEventsUserState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::spawn(async move { process_message(client, event).await });
    Ok(())
}

pub fn error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: Arc<SlackClientEventsUserState>,
) -> http::StatusCode {
    error!("{:#?}", err);
    http::StatusCode::OK
}

#[instrument(
    skip(client, event),
    fields(
        request_id = %Uuid::new_v4()
        )
    )]
async fn process_message(
    client: Arc<SlackHyperClient>,
    event: SlackPushEventCallback,
) -> Result<(), RustyBotError> {
    let token_value =
        SlackApiTokenValue(env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN env var not found"));
    let token = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    let playground_url = env::var("PLAYGROUND_URL").expect("PLAYGROUND_URL env var not found");

    match event.event {
        SlackEventCallbackBody::Message(msg_event) => {
            debug!("Matched message");
            let channel = msg_event.origin.channel;
            let content = msg_event.content;
            if let Some(channel_id) = channel {
                if let Some(msg_content) = content {
                    debug!(
                        "Found channel {} and content {:?}",
                        channel_id, msg_content.text
                    );
                    let text = msg_content.text;
                    debug!("Start matching has_ functions");
                    // start matching the has_ functions
                    // code
                    if let Some(code) = has_code(&text) {
                        debug!("Found code: {:?}", code);
                        // print "executing"
                        let reply_content =
                            SlackMessageContent::new().with_text("Executing...".to_owned());
                        let reply_request =
                            SlackApiChatPostMessageRequest::new(channel_id.clone(), reply_content);
                        let _response = session.chat_post_message(&reply_request).await;
                        let response = eval_code(&code, &playground_url)
                            .await
                            .map_err(|e| {
                                error!("Error: {}\n when executing eval_code with code type: {}\ncode text: {}\nbase URL {}", e, code.kind, code.text, playground_url);
                                RustyBotError::InternalServerError(e.into())})?;
                        let reply_content = CodeReplyTemplate::new(
                            &response.link,
                            response.playground_answer.stdout.clone(),
                            response.playground_answer.stderr.clone(),
                        );
                        debug!(
                            "Reply produced\nlink: {}\nstdout: {}\n stderr: {}",
                            &response.link,
                            response.playground_answer.stdout,
                            response.playground_answer.stderr
                        );
                        let reply_request = SlackApiChatPostMessageRequest::new(
                            channel_id,
                            reply_content.render_template(),
                        );
                        let _response = session.chat_post_message(&reply_request).await;
                        debug!(
                            "Response from session.chat_post_message of code: {:?}",
                            _response
                        );
                        return Ok(());
                    }
                    // command
                    else if let Some(command) = has_command(&text) {
                        debug!("Found command: {}", command);
                        if let Some(output) = eval_command(command.clone()) {
                            debug!("command {} produced output {}", command, output);
                            let reply_content = SlackMessageContent::new().with_text(output);
                            let reply_request =
                                SlackApiChatPostMessageRequest::new(channel_id, reply_content);
                            let _response = session.chat_post_message(&reply_request).await;
                            debug!(
                                "Response from session.chat_post_message of command: {:?}",
                                _response
                            );
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
            debug!("Matched mention");
            let channel_id = mention_event.channel;
            debug!("channel_id: {}", channel_id);
            let reply_content =
                SlackMessageContent::new().with_text("I'm alive, don't worry".to_owned());
            let reply_request = SlackApiChatPostMessageRequest::new(channel_id, reply_content);
            let _response = session.chat_post_message(&reply_request).await;
            debug!(
                "Response from session.chat_post_message of mention: {:?}",
                _response
            );
            Ok(())
        }
        _ => Ok(()),
    }
}

fn eval_command(command: String) -> Option<String> {
    match command.to_lowercase().as_str() {
        "docs" => Some("https://doc.rust-lang.org/".to_owned()),
        "book" => Some("https://doc.rust-lang.org/book/".to_owned()),
        _ => Some("*Available commands*\n!code - for complete code blocks\n!eval - for evaluating chunks that can fit in main function\n!help [docs, book] - links to classic rust material\n_Yours truely, Ferris_".to_owned()),
    }
}

async fn eval_code(code: &Code, playground_url: &str) -> Result<PlaygroundAnswer, RustyBotError> {
    let request;
    if code.kind == *"code" {
        request = PlaygroundRequest::new(code.text.clone()).escape_html();
    } else if code.kind == *"eval" {
        request = PlaygroundRequest::new_eval(code.text.clone()).escape_html();
    } else {
        error!("Error: InvalidBotCommand reached! code kind: {}", code.kind);
        return Err(RustyBotError::InvalidBotCommand {
            // never really reached because of has_code matching
            command: code.kind.to_owned(),
        });
    };
    let result = request.execute(playground_url).await;
    match result {
        Ok(res) => {
            let ans = PlaygroundAnswer {
                playground_answer: res.playground_response,
                link: request.create_share_link(playground_url).await?,
            };
            Ok(ans)
        }
        Err(e) => {
            error!(
                "Error: {}\nerror during eval_code when trying to run execute function on request to base URL: {}",
                e, playground_url
            );
            Err(RustyBotError::InternalServerError(e.into()))
        }
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
            let code_result = re.captures(text).map(|capture| Code {
                kind: String::from(&capture["kind"]),
                text: String::from(&capture["code"]),
            });
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
            let command_result = re
                .captures(text)
                .map(|capture| String::from(&capture["command"]));
            if command_result == Some("".to_owned()) {
                None
            } else {
                command_result
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_command() {
        let message_with_command = &Some("!help book".to_owned());
        let message_without_command = &Some("nothing here".to_owned());
        assert_eq!(has_command(message_with_command), Some("book".to_owned()));
        assert!(has_command(message_without_command).is_none());
    }

    #[test]
    fn test_has_code() {
        let message_with_code = &Some("!eval\n```this is code```".to_owned());
        let message_without_code = &Some("!bla\n```this is not code```".to_owned());

        let ans_with_code = has_code(message_with_code).unwrap();
        assert_eq!(ans_with_code.kind, "eval".to_owned());
        assert_eq!(ans_with_code.text, "this is code".to_owned());

        let ans_without_code = has_code(message_without_code);
        assert!(ans_without_code.is_none());
    }

    #[test]
    fn test_eval_command() {
        let command_docs = "docs".to_owned();
        let command_book = "book".to_owned();

        let expected_reply_docs = "https://doc.rust-lang.org/".to_owned();
        let expected_reply_book = "https://doc.rust-lang.org/book/".to_owned();
        let expected_reply_other = "*Available commands*\n!code - for complete code blocks\n!eval - for evaluating chunks that can fit in main function\n!help [docs, book] - links to classic rust material\n_Yours truely, Ferris_".to_owned();

        let reply_docs = eval_command(command_docs).unwrap();
        let reply_book = eval_command(command_book).unwrap();
        let reply_other = eval_command("something".to_owned()).unwrap();

        assert_eq!(expected_reply_docs, reply_docs);
        assert_eq!(expected_reply_book, reply_book);
        assert_eq!(expected_reply_other, reply_other);
    }
}
