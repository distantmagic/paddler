use std::collections::VecDeque;
use std::num::NonZeroU16;
use std::sync::Arc;

use tokio::sync::mpsc;

use paddler_agent_embeddings::active_embedding_request::ActiveEmbeddingRequest;
use paddler_agent_embeddings::embedding_input_tokenized::EmbeddingInputTokenized;
use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_messaging::embedding_normalization_method::EmbeddingNormalizationMethod;
use paddler_messaging::embedding_result::EmbeddingResult;

pub struct ActiveEmbeddingRequestFixture {
    pub active_embedding_request: ActiveEmbeddingRequest,
    pub generate_embedding_stop_tx: mpsc::UnboundedSender<()>,
    pub generated_embedding_rx: mpsc::UnboundedReceiver<EmbeddingResult>,
}

impl ActiveEmbeddingRequestFixture {
    #[must_use]
    pub fn new(
        pending_inputs: Vec<EmbeddingInputTokenized>,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
        reserved_slots: NonZeroU16,
    ) -> Self {
        let (generated_embedding_tx, generated_embedding_rx) = mpsc::unbounded_channel();
        let (generate_embedding_stop_tx, generate_embedding_stop_rx) = mpsc::unbounded_channel();

        Self {
            active_embedding_request: ActiveEmbeddingRequest {
                embeddings_emitted: 0,
                generate_embedding_stop_rx,
                generated_embedding_tx,
                normalization_method: EmbeddingNormalizationMethod::None,
                pending_inputs: VecDeque::from(pending_inputs),
                slot_guard: SlotGuard::new(slot_aggregated_status, reserved_slots),
            },
            generate_embedding_stop_tx,
            generated_embedding_rx,
        }
    }
}
