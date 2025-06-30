use serde::Serialize;

use super::error::Error;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response<TResult>
where
    TResult: Serialize,
{
    Success {
        request_id: String,
        result: TResult,
    },
    Error {
        request_id: String,
        error: Error,
    },
}

impl<TResult> Response<TResult>
where
    TResult: Serialize,
{
    pub fn forbidden(request_id: String) -> Self {
        Self::Error {
            request_id,
            error: Error::forbidden(),
        }
    }

    pub fn not_found(request_id: String) -> Self {
        Self::Error {
            request_id,
            error: Error::not_found(),
        }
    }

    pub fn server_error(request_id: String, error: anyhow::Error) -> Self {
        Self::Error {
            request_id,
            error: Error::server_error(error),
        }
    }
}
