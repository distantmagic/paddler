#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Unable to communicate with actor: {0}")]
    ActixActorMailbox(#[from] actix::MailboxError),

    #[cfg(feature = "statsd_reporter")]
    #[error("Cadence error: {0}")]
    CadenceMetrix(#[from] cadence::MetricError),

    #[error("Invalid request header: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tokio Join Error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Pingora error: {0}")]
    BoxedPingora(#[from] Box<pingora::Error>),

    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Tokio broadcast receive error: {0}")]
    TokioBroadcastSendBytesError(
        #[from] tokio::sync::broadcast::error::SendError<actix_web::web::Bytes>,
    ),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("URL parse error: {0}")]
    URLParse(#[from] url::ParseError),

    #[error("Time parse error: {0}")]
    TimeParse(#[from] std::time::SystemTimeError),

    #[error("RwLock poison error: {0}")]
    RwLockPoisonError(String),
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
