use crate::error::RustyBotError;
use reqwest::blocking::Client;

pub struct SlackRust {
    client: Client,
    channel_id: String,
    share_link: String,
    stdout: String,
    stderr: String,
}

impl SlackRust {
    pub fn new(channel_id: String, share_link: String, stdout: String, stderr: String) -> Self {
        // TODO: initialize the client with the token header
    }

    pub fn send(&self) -> Result<(), RustyBotError> {
        // TODO: construct payload with format!() and send.
    }
}

// TODO:
// 1. impl a giant string which will be a format! with the payload of the formatted message
// 2. when the event Message occurs (listening for it using the RtmClient from slack library) then
//    on_message() is triggered and eventually the sending will be done with SlackRust which will be a
//    client with send_message method. posting to this endpoint
//    https://slack.com/api/chat.postMessage with the proper bearer token in the header
