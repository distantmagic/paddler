use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    pub code: i32,
    pub description: Option<String>,
}

impl Error {
    pub fn bad_request(err: Option<serde_json::Error>) -> Self {
        Self {
            code: 400,
            description: err.map(|err| err.to_string()),
        }
    }

    pub fn forbidden() -> Self {
        Self {
            code: 403,
            description: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: 404,
            description: None,
        }
    }

    pub fn parse() -> Self {
        Self {
            code: 600,
            description: None,
        }
    }

    pub fn server_error(_error: anyhow::Error) -> Self {
        Self {
            code: 500,
            description: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "jsonrpc_error(code={})", self.code)
    }
}
