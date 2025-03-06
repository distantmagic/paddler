#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("Unable to communicate with actor: {0}")]
    ActixActorMailboxError(#[from] actix::MailboxError),

    #[cfg(feature = "statsd_reporter")]
    #[error("Cadence error: {0}")]
    CadenceMetrixError(#[from] cadence::MetricError),

    #[error("Invalid request header: {0}")]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tokio Join Error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Pingora error: {0}")]
    BoxedPingoraError(#[from] Box<pingora::Error>),

    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Tokio broadcast receive error: {0}")]
    TokioBroadcastSendBytesError(
        #[from] tokio::sync::broadcast::error::SendError<actix_web::web::Bytes>,
    ),

    #[error("Tokio broadcast receive error: {0}")]
    TokioBroadcastSendConfigError(#[from] tokio::sync::broadcast::error::SendError<Vec<String>>),

    #[error("Tokio broadcast receive error: {0}")]
    MapToVecParseError(#[from] mavec::error::MavecError),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("URL parse error: {0}")]
    URLParseError(#[from] url::ParseError),

    #[error("Time parse error: {0}")]
    TimeParseError(#[from] std::time::SystemTimeError),

    #[error("RwLock poison error: {0}")]
    RwLockPoisonError(String),

    #[error("Invalid file error: {0}")]
    InvalidFileError(String),

    #[cfg(feature = "etcd")]
    #[error("Invalid file error: {0}")]
    ConfigurationServerError(#[from] etcd_client::Error),

    #[error("Invalid config error: {0}")]
    ConfigurationFileParseError(#[from] toml_edit::TomlError),
}

impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::UnexpectedError(error.to_string())
    }
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError()
            .body(format!("Internal Server Error: {}", self))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        AppError::RwLockPoisonError(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::InvalidFileError(err)
    }
}
