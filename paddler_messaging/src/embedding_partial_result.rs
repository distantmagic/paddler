use crate::embedding::Embedding;
use crate::embedding_document_exceeding_sequence_context_details::EmbeddingDocumentExceedingSequenceContextDetails;
use crate::oversized_embedding_document_details::OversizedEmbeddingDocumentDetails;

#[derive(Debug, PartialEq)]
pub enum EmbeddingPartialResult {
    DocumentExceedsBatchSize(OversizedEmbeddingDocumentDetails),
    DocumentExceedsSequenceContext(EmbeddingDocumentExceedingSequenceContextDetails),
    Embedding(Embedding),
}
