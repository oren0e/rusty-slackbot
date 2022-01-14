pub mod bot;
pub mod error;
pub mod playground;
pub mod slack_conn;

use crate::bot::on_message;
use crate::error::RustyBotError;
use slack_morphism::prelude::*;
use slack_morphism_hyper::*;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), RustyBotError> {
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

    let socket_mode_callbacks =
        SlackSocketModeListenerCallbacks::new().with_push_events(on_message);
    let listener_environment = Arc::new(SlackClientEventsListenerEnvironment::new(client.clone()));
    let socket_mode_listener = SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listener_environment.clone(),
        socket_mode_callbacks,
    );

    let app_token_value =
        SlackApiTokenValue(env::var("SLACK_APP_TOKEN").expect("SLACK_APP_TOKEN env var not found"));
    let app_token = SlackApiToken::new(app_token_value);

    socket_mode_listener.listen_for(&app_token).await?;

    socket_mode_listener.serve().await;

    Ok(())
}
