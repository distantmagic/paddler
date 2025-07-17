use std::sync::Arc;

use crate::agent::slot_aggregated_metrics::SlotAggregatedMetrics;
use crate::agent::slot_metrics::SlotMetrics;

pub struct SlotAggregatedMetricsManager {
    pub slot_aggregated_metrics: Arc<SlotAggregatedMetrics>,
}

impl SlotAggregatedMetricsManager {
    pub fn new(slots_total: i32) -> Self {
        SlotAggregatedMetricsManager {
            slot_aggregated_metrics: Arc::new(SlotAggregatedMetrics::new(slots_total)),
        }
    }

    pub fn bind_slot_metrics(&self) -> Arc<SlotMetrics> {
        Arc::new(SlotMetrics::new(self.slot_aggregated_metrics.clone()))
    }

    pub fn reset(&self) {
        self.slot_aggregated_metrics.reset();
    }
}
