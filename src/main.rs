pub mod bot;
pub mod error;
pub mod playground;
pub mod slack_conn;

use crate::bot::RustyBot;
use slack::RtmClient;
use std::env;

fn main() {
    let token = env::var("SLACK_TOKEN").expect("SLACK_TOKEN env var was not found");

    let mut bot_handler = RustyBot {};
    let response = RtmClient::login_and_run(&token, &mut bot_handler);

    println!("{:?}", response);
    match response {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}
