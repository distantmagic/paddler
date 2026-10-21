use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use parking_lot::Mutex;
use tokio::sync::watch;

use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;

use crate::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;

pub struct ClusterInferenceModeBroadcaster {
    latest_connection_generation: AtomicU64,
    reported_cluster_inference_mode_tx: watch::Sender<ReportedClusterInferenceMode>,
    reporting_connection_generation: Mutex<u64>,
}

impl ClusterInferenceModeBroadcaster {
    pub fn publish(&self, report: ClusterInferenceModeReport, connection_generation: u64) {
        let mut reporting_connection_generation = self.reporting_connection_generation.lock();

        self.reported_cluster_inference_mode_tx.send_if_modified(
            |last_reported| match last_reported {
                ReportedClusterInferenceMode::Reported(last_report)
                    if report.is_a_later_version_of(last_report)
                        || (report.balancer_instance != last_report.balancer_instance
                            && connection_generation > *reporting_connection_generation) =>
                {
                    let inference_mode_changed =
                        last_report.inference_mode != report.inference_mode;

                    *last_report = report;
                    *reporting_connection_generation = connection_generation;

                    inference_mode_changed
                }
                ReportedClusterInferenceMode::Reported(_) => false,
                ReportedClusterInferenceMode::NotYetReported => {
                    *last_reported = ReportedClusterInferenceMode::Reported(report);
                    *reporting_connection_generation = connection_generation;

                    true
                }
            },
        );
    }

    pub fn register_connection(&self) -> u64 {
        self.latest_connection_generation
            .fetch_add(1, Ordering::Relaxed)
            + 1
    }

    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<ReportedClusterInferenceMode> {
        self.reported_cluster_inference_mode_tx.subscribe()
    }
}

impl Default for ClusterInferenceModeBroadcaster {
    fn default() -> Self {
        let (reported_cluster_inference_mode_tx, _initial_reported_cluster_inference_mode_rx) =
            watch::channel(ReportedClusterInferenceMode::NotYetReported);

        Self {
            latest_connection_generation: AtomicU64::new(0),
            reported_cluster_inference_mode_tx,
            reporting_connection_generation: Mutex::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;
    use paddler_messaging::inference_mode::InferenceMode;

    use super::ClusterInferenceModeBroadcaster;
    use crate::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;

    fn report(
        balancer_instance: &str,
        version: u64,
        inference_mode: InferenceMode,
    ) -> ClusterInferenceModeReport {
        ClusterInferenceModeReport {
            balancer_instance: balancer_instance.to_owned(),
            inference_mode,
            version,
        }
    }

    #[test]
    fn a_late_report_of_an_earlier_state_does_not_replace_a_newer_one() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let connection_generation = broadcaster.register_connection();
        let latest_report = report("first", 3, InferenceMode::TextGenerationAndEmbeddings);

        broadcaster.publish(
            report(
                "first",
                2,
                InferenceMode::TextGenerationEmbeddingsAndDecisions,
            ),
            connection_generation,
        );
        broadcaster.publish(latest_report.clone(), connection_generation);
        broadcaster.publish(
            report(
                "first",
                2,
                InferenceMode::TextGenerationEmbeddingsAndDecisions,
            ),
            connection_generation,
        );

        assert_eq!(
            *broadcaster.subscribe().borrow(),
            ReportedClusterInferenceMode::Reported(latest_report)
        );
    }

    #[test]
    fn a_newer_report_of_the_same_mode_does_not_notify_subscribers() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let connection_generation = broadcaster.register_connection();

        broadcaster.publish(
            report("first", 1, InferenceMode::TextGenerationAndEmbeddings),
            connection_generation,
        );

        let subscriber = broadcaster.subscribe();

        broadcaster.publish(
            report("first", 2, InferenceMode::TextGenerationAndEmbeddings),
            connection_generation,
        );

        assert_eq!(subscriber.has_changed().ok(), Some(false));
    }

    #[test]
    fn a_report_from_an_older_connection_to_a_replaced_balancer_does_not_undo_the_newer_instance() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let older_connection_generation = broadcaster.register_connection();
        let newer_connection_generation = broadcaster.register_connection();
        let restarted_balancer_report = report(
            "restarted",
            0,
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
        );

        broadcaster.publish(
            report("first", 5, InferenceMode::TextGenerationAndEmbeddings),
            older_connection_generation,
        );
        broadcaster.publish(
            restarted_balancer_report.clone(),
            newer_connection_generation,
        );
        broadcaster.publish(
            report("first", 6, InferenceMode::TextGenerationAndEmbeddings),
            older_connection_generation,
        );

        assert_eq!(
            *broadcaster.subscribe().borrow(),
            ReportedClusterInferenceMode::Reported(restarted_balancer_report)
        );
    }

    #[test]
    fn a_restarted_balancer_reported_over_a_newer_connection_replaces_the_earlier_instance() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let older_connection_generation = broadcaster.register_connection();
        let restarted_balancer_report = report(
            "restarted",
            0,
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
        );

        broadcaster.publish(
            report("first", 5, InferenceMode::TextGenerationAndEmbeddings),
            older_connection_generation,
        );
        broadcaster.publish(
            restarted_balancer_report.clone(),
            broadcaster.register_connection(),
        );

        assert_eq!(
            *broadcaster.subscribe().borrow(),
            ReportedClusterInferenceMode::Reported(restarted_balancer_report)
        );
    }
}
