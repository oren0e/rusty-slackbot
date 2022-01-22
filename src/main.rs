use rusty_slackbot::bot::{error_handler, on_message};
use rusty_slackbot::error::RustyBotError;
use slack_morphism::prelude::*;
use slack_morphism_hyper::*;
use std::env;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), RustyBotError> {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(EnvFilter::new(
            env::var("RUSTY_LOG_LEVEL").expect("RUSTY_LOG_LEVEL env var was not found"),
        ))
        .init();

    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

    let socket_mode_callbacks =
        SlackSocketModeListenerCallbacks::new().with_push_events(on_message);
    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone()).with_error_handler(error_handler),
    );
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
