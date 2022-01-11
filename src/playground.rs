use crate::error::RustyBotError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundRequest {
    backtrace: bool,
    channel: RustChannel,
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

#[derive(Debug, Deserialize, Clone)]
pub struct PlaygroundResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub enum RustChannel {
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "nightly")]
    Nightly,
}

#[derive(Debug, Deserialize)]
struct ShareResponse {
    pub id: String,
    pub url: String,
}

impl fmt::Display for RustChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RustChannel::Stable => "stable",
                RustChannel::Beta => "beta",
                RustChannel::Nightly => "nightly",
            }
        )
    }
}

impl FromStr for RustChannel {
    type Err = RustyBotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "nightly" => Ok(Self::Nightly),
            _ => Err(RustyBotError::InvalidRustChannel),
        }
    }
}

impl PlaygroundRequest {
    pub fn new(code: String, channel: RustChannel) -> Self {
        Self {
            backtrace: false,
            channel,
            code,
            crate_type: "bin",
            edition: "2021",
            mode: "debug",
            tests: false,
        }
    }

    pub fn new_eval(code: String) -> Self {
        let code_to_eval = format!("fn main() {{{}}}", code);
        Self {
            backtrace: false,
            channel: RustChannel::Stable,
            code: code_to_eval,
            crate_type: "bin",
            edition: "2021",
            mode: "debug",
            tests: false,
        }
    }

    pub fn execute(&self) -> Result<Response, RustyBotError> {
        let response = Client::new()
            .post("https://play.rust-lang.org/execute")
            .json(self)
            .send()
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        let status_code = response.status().as_str().to_string();
        let playground_response: PlaygroundResponse = serde_json::from_str(
            &response
                .text()
                .map_err(|e| RustyBotError::InternalServerError(e.into()))?,
        )
        .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        let ans = Response {
            status_code,
            playground_response,
        };
        Ok(ans)
    }

    pub fn create_share_link(&self) -> Result<String, RustyBotError> {
        let share_response: ShareResponse = Client::new()
            .post("https://play.rust-lang.org/meta/gist/")
            .json(&json!({"code": self.code}))
            .send()
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?
            .json()
            .map_err(|e| RustyBotError::InternalServerError(e.into()))?;
        Ok(format!(
            "https://play.rust-lang.org/?version={}&mode=debug&edition={}&gist={}",
            self.channel, self.edition, share_response.id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_working_code() {
        let channel = RustChannel::Stable;
        let code = String::from("fn main() {\n\tprintln!(\"Hello, world!\");\n}");

        let request = PlaygroundRequest::new(code, channel);
        let response = request.execute().unwrap();

        assert_eq!(response.status_code, "200");
        assert!(response
            .playground_response
            .stdout
            .contains("Hello, world!"));
    }

    #[test]
    fn test_execute_not_working_code() {
        let channel = RustChannel::Stable;
        let code = String::from("fn main() {\n\tprintln!(\"Hello, world!\");\n"); // missing "}"

        let request = PlaygroundRequest::new(code, channel);
        let response = request.execute().unwrap();

        assert_eq!(response.status_code, "200");
        assert!(!response.playground_response.success);
        assert_eq!(response.playground_response.stdout, "");
    }

    #[test]
    fn test_create_share_link() {
        let channel = RustChannel::Stable;
        let code = String::from("fn main() {\n\tprintln!(\"Hello, world!\");\n}");

        let request = PlaygroundRequest::new(code, channel);
        request.create_share_link().unwrap();
    }

    #[test]
    fn test_eval() {
        let code = String::from("let v = vec![1,2,3];\n    println!(\"{:?}\", v[1]);");

        let request = PlaygroundRequest::new_eval(code);
        let response = request.execute().unwrap();
        assert!(response.playground_response.success);
        assert_eq!(response.playground_response.stdout, "2\n");
    }

    #[test]
    fn test_from_str_channel() {
        let channel = RustChannel::from_str("stable").unwrap();
        assert_eq!(channel.to_string(), "stable".to_string());
    }
}
