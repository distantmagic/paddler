use crate::active_embedding_request::ActiveEmbeddingRequest;

pub enum EmbeddingRequestContinuation {
    Concluded,
    Continues(ActiveEmbeddingRequest),
}
