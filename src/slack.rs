use crate::error::RustyBotError;
use reqwest::blocking::Client;
use reqwest::header;
use std::env;

pub struct SlackClient {
    client: Client,
}

impl SlackClient {
    pub fn init() -> Result<Self, RustyBotError> {
        let token = env::var("SLACK_TOKEN").expect("SLACK_TOKEN env var was not found");
        let bearer = format!("Bearer {}", token);
        let mut auth_value = header::HeaderValue::from_str(&bearer)
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        auth_value.set_sensitive(true);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        Ok(Self { client })
    }

    pub fn send_code_reply(
        &self,
        channel_id: String,
        share_link: String,
        stdout: String,
        stderr: String,
    ) -> Result<(), RustyBotError> {
        let payload = format!(
            r#"{{
    "channel": \"{}\",
    "text": "Executing...",
	"blocks": [
		{{
			"type": "header",
			"text": {{
				"type": "plain_text",
				"text": "Rust Playground",
				"emoji": true
			}}
		}},
		{{
			"type": "section",
			"text": {{
				"type": "mrkdwn",
				"text": "Here is the code on Rust Playground"
			}},
			"accessory": {{
				"type": "button",
				"text": {{
					"type": "plain_text",
					"text": "Click Me",
					"emoji": true
				}},
				"value": "click_me_123",
				"url": \"{}\",
				"action_id": "button-action"
			}}
		}},
		{{
			"type": "context",
			"elements": [
				{{
					"type": "plain_text",
					"text": "Stdout",
					"emoji": true
				}}
			]
		}},
		{{
			"type": "section",
			"text": {{
				"type": "mrkdwn",
				"text": "```{}```"
			}}
		}},
		{{
			"type": "divider"
		}},
		{{
			"type": "context",
			"elements": [
				{{
					"type": "plain_text",
					"text": "Stderr",
					"emoji": true
				}}
			]
		}},
		{{
			"type": "section",
			"text": {{
				"type": "mrkdwn",
				"text": "```{}``"
			}}
		}}
	]
}}"#,
            channel_id, share_link, stdout, stderr
        );
        self.client
            .post("https://slack.com/api/chat.postMessage")
            .body(payload)
            .send()
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        Ok(())
    }
}

// TODO:
// 2. when the event Message occurs (listening for it using the RtmClient from slack library) then
//    on_message() is triggered and eventually the sending will be done with SlackRust which will be a
//    client with send_message method. posting to this endpoint
//    https://slack.com/api/chat.postMessage with the proper bearer token in the header
