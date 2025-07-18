use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub enum ResponseEnvelope<TResponse> {
    Error {
        request_id: String,
        error: String,
    },
    // OneShot {
    //     request_id: String,
    //     response: TResponse,
    // },
    StreamChunk {
        request_id: String,
        chunk: TResponse,
    },
    StreamDone {
        request_id: String,
    },
}
