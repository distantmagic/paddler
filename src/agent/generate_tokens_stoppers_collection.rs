use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use dashmap::DashMap;

use crate::agent::generate_tokens_stop_result::GenerateTokensStopResult;

pub struct GenerateTokensStoppersCollection {
    stoppers: DashMap<String, Arc<AtomicBool>>,
}

impl GenerateTokensStoppersCollection {
    pub fn new() -> Self {
        GenerateTokensStoppersCollection {
            stoppers: DashMap::new(),
        }
    }

    pub fn clear(&self, request_id: String) {
        self.stoppers.remove(&request_id);
    }

    pub fn register_for(&self, request_id: String) -> Arc<AtomicBool> {
        let stopper = Arc::new(AtomicBool::new(false));
        self.stoppers.insert(request_id, stopper.clone());

        stopper
    }

    pub fn stop(&self, request_id: String) -> GenerateTokensStopResult {
        if let Some(stopper) = self.stoppers.get(&request_id) {
            stopper.store(true, Relaxed);
            self.clear(request_id.clone());

            GenerateTokensStopResult::Stopped
        } else {
            GenerateTokensStopResult::RequestNotFound(request_id)
        }
    }

    pub fn stop_all(&self) {
        for ref_multi in self.stoppers.iter() {
            ref_multi.value().store(true, Relaxed);
        }

        self.stoppers.clear();
    }
}
