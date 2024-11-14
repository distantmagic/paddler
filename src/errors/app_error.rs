#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unable to communicate with actor: {0}")]
    ActixActorMailboxError(#[from] actix::MailboxError),

    #[error("Invalid request header: {0}")]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tokio Join Error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    URLParseError(#[from] url::ParseError),
}
