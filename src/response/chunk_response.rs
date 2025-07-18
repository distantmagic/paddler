use crate::response_params::GeneratedToken;

#[derive(Debug)]
pub enum ChunkResponse {
    Data(GeneratedToken),
    Error(String),
}
