use crate::error::RustyBotError;
use reqwest::header;
use reqwest::Client;
use serde_json::json;
use std::env;

const BASE_URL: &str = "https://slack.com/api/";

#[derive(Debug)]
pub struct SlackSendClient {
    client: Client,
}

impl SlackSendClient {
    pub async fn init() -> Result<Self, RustyBotError> {
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

    pub async fn send_code_reply(
        &self,
        channel_id: &str,
        share_link: &str,
        stdout: String,
        stderr: String,
    ) -> Result<(), RustyBotError> {
        let payload = json!(
                    {
            "channel": channel_id.to_string(),
            "text": "Executing...",
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": "Rust Playground",
                        "emoji": true
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": "Here is the code on Rust Playground"
                    },
                    "accessory": {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "Click Me",
                            "emoji": true
                        },
                        "value": "click_me_123",
                        "url": share_link.to_string(),
                        "action_id": "button-action"
                    }
                },
                {
                    "type": "context",
                    "elements": [
                        {
                            "type": "plain_text",
                            "text": "Stdout",
                            "emoji": true
                        }
                    ]
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("```{}```", stdout)
                    }
                },
                {
                    "type": "divider"
                },
                {
                    "type": "context",
                    "elements": [
                        {
                            "type": "plain_text",
                            "text": "Stderr",
                            "emoji": true
                        }
                    ]
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("```{}```", stderr)
                    }
                }
            ]
        }
                    );
        let _response = self
            .client
            .post(format!("{}/chat.postMessage", BASE_URL))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        Ok(())
    }

    pub async fn send_reply(&self, channel_id: &str, reply: String) -> Result<(), RustyBotError> {
        let payload = json!(
        {
        "channel": channel_id.to_string(),
        "text": reply
        });
        let _response = self
            .client
            .post(format!("{}/chat.postMessage", BASE_URL))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        let txt = _response
            .text()
            .await
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        println!("{}", txt);
        Ok(())
    }
}

// TODO:
// 3. create tests using a mock server (httpmock crate)
