use slack_morphism::errors::SlackClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustyBotError {
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
    #[error("No code provided")]
    MissingCode,
    #[error("Provided rust channel does not exist, please use Stable, Beta, or Nightly")]
    InvalidRustChannel,
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
