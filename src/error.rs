use slack_morphism::errors::SlackClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustyBotError {
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
    #[error(
        "The command {command} is not a valid command for the bot. Use one of !code, !eval, !help (docs, book)"
    )]
    InvalidBotCommand { command: String },
}

impl From<SlackClientError> for RustyBotError {
    fn from(err: SlackClientError) -> Self {
        RustyBotError::InternalServerError(anyhow::anyhow!(err))
    }
}
