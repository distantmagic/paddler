use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub enum Response {
    Timeout,
    TooManyBufferedRequests,
}
