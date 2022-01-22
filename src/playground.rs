use crate::error::RustyBotError;
use html_escape::decode_html_entities;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundRequest {
    backtrace: bool,
    channel: &'static str,
    code: String,
    crate_type: &'static str,
    edition: &'static str,
    mode: &'static str,
    tests: bool,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub status_code: String,
    pub playground_response: PlaygroundResponse,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlaygroundResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ShareResponse {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaygroundAnswer {
    pub playground_answer: PlaygroundResponse,
    pub link: String,
}

impl PlaygroundRequest {
    pub fn new(code: String) -> Self {
        Self {
            backtrace: false,
            channel: "stable",
            code,
            crate_type: "bin",
            edition: "2021",
            mode: "debug",
            tests: false,
        }
    }

    pub fn get_code(&self) -> String {
        self.code.clone()
    }

    pub fn get_channel(&self) -> String {
        self.channel.to_owned()
    }

    pub fn get_edition(&self) -> String {
        self.edition.to_owned()
    }

    pub fn new_eval(code: String) -> Self {
        let code_to_eval = format!("fn main() {{{}}}", code);
        Self {
            backtrace: false,
            channel: "stable",
            code: code_to_eval,
            crate_type: "bin",
            edition: "2021",
            mode: "debug",
            tests: false,
        }
    }

    pub fn escape_html(&self) -> Self {
        Self {
            backtrace: self.backtrace,
            channel: self.channel,
            code: decode_html_entities(&self.code).as_ref().to_owned(),
            crate_type: self.crate_type,
            edition: self.edition,
            mode: self.mode,
            tests: self.tests,
        }
    }

    pub async fn execute(&self, playground_url: &str) -> Result<Response, RustyBotError> {
        debug!("execute function start with base URL: {}", playground_url);
        let response = Client::new()
            .post(format!("{}/execute", playground_url))
            .json(&self)
            .send()
            .await
            .map_err(|e| {
                error!(
                    "Error: {}\nwhen sending request to base URL {} with payload {:?}",
                    e, playground_url, &self
                );
                RustyBotError::InternalServerError(e.into())
            })?;
        let status_code = response.status().as_str().to_owned();
        let playground_response: PlaygroundResponse = serde_json::from_str(
            &response
                .text()
                .await
                .map_err(|e| {
                    error!("Error: {}\n when trying to retreive body text of response in execute function to base URL: {}", e, playground_url);
                    RustyBotError::InternalServerError(e.into())})?,
        )
        .map_err(|e| {
            error!("Error: {}\n when trying to convert response body text to JSON format in execute function to base URL: {}", e, playground_url);
            RustyBotError::InternalServerError(e.into())})?;
        let ans = Response {
            status_code,
            playground_response,
        };
        Ok(ans)
    }

    pub async fn create_share_link(&self, playground_url: &str) -> Result<String, RustyBotError> {
        debug!(
            "create_share_link function start with base URL: {}",
            playground_url
        );
        let share_response: ShareResponse = Client::new()
            .post(format!("{}/meta/gist/", playground_url))
            .json(&json!({"code": self.code}))
            .send()
            .await
            .map_err(|e| {
                error!(
                    "Error: {}\n when sending request to create share link to base URL {} with payload {:?}",
                    e, playground_url,
                    json!({"code": self.code})
                );
                RustyBotError::InternalServerError(e.into())
            })?
            .json()
            .await
            .map_err(|e| {
                error!("Error: {}\n when trying to deserialize response of create_share_link to ShareResponse. base URL of request: {}", e, playground_url);
                RustyBotError::InternalServerError(e.into())})?;
        debug!(
            "Share link produced: {}",
            format!(
                "https://play.rust-lang.org/?version={}&mode=debug&edition={}&gist={}",
                self.channel, self.edition, share_response.id
            )
        );
        Ok(format!(
            "https://play.rust-lang.org/?version={}&mode=debug&edition={}&gist={}",
            self.channel, self.edition, share_response.id
        ))
    }
}
