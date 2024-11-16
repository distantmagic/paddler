#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("Unable to communicate with actor: {0}")]
    ActixActorMailboxError(#[from] actix::MailboxError),

    #[error("Invalid request header: {0}")]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tokio Join Error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Pingora error: {0}")]
    BoxedPingoraError(#[from] Box<pingora::Error>),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Tokio broadcast receive error: {0}")]
    TokioBroadcastSendBytesError(
        #[from] tokio::sync::broadcast::error::SendError<actix_web::web::Bytes>,
    ),

    #[error("URL parse error: {0}")]
    URLParseError(#[from] url::ParseError),
}
