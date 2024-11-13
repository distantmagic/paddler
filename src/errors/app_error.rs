#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unable to communicate with actor: {0}")]
    ActixActorMailboxError(#[from] actix::MailboxError),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    URLParseError(#[from] url::ParseError),

    #[error("Invalid request header: {0}")]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),
}

