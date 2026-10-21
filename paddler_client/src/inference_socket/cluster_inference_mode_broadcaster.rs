use tokio::sync::watch;

use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;

use crate::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;

pub struct ClusterInferenceModeBroadcaster {
    reported_cluster_inference_mode_tx: watch::Sender<ReportedClusterInferenceMode>,
}

impl ClusterInferenceModeBroadcaster {
    pub fn publish(&self, report: ClusterInferenceModeReport) {
        self.reported_cluster_inference_mode_tx.send_if_modified(
            |last_reported| match last_reported {
                ReportedClusterInferenceMode::Reported(last_report)
                    if !report.supersedes(last_report) =>
                {
                    false
                }
                ReportedClusterInferenceMode::Reported(last_report) => {
                    let inference_mode_changed =
                        last_report.inference_mode != report.inference_mode;

                    *last_report = report;

                    inference_mode_changed
                }
                ReportedClusterInferenceMode::NotYetReported => {
                    *last_reported = ReportedClusterInferenceMode::Reported(report);

                    true
                }
            },
        );
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
            reported_cluster_inference_mode_tx,
        }
    }
}

#[cfg(test)]
mod tests {
    use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;
    use paddler_messaging::inference_mode::InferenceMode;

    use super::ClusterInferenceModeBroadcaster;
    use crate::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;

    const fn report(
        balancer_instance: u32,
        version: u64,
        inference_mode: InferenceMode,
    ) -> ClusterInferenceModeReport {
        ClusterInferenceModeReport {
            balancer_instance,
            inference_mode,
            version,
        }
    }

    #[test]
    fn a_late_report_of_an_earlier_state_does_not_replace_a_newer_one() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let latest_report = report(1, 3, InferenceMode::TextGenerationAndEmbeddings);

        broadcaster.publish(report(
            1,
            2,
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
        ));
        broadcaster.publish(latest_report);
        broadcaster.publish(report(
            1,
            2,
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
        ));

        assert_eq!(
            *broadcaster.subscribe().borrow(),
            ReportedClusterInferenceMode::Reported(latest_report)
        );
    }

    #[test]
    fn a_newer_report_of_the_same_mode_does_not_notify_subscribers() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();

        broadcaster.publish(report(1, 1, InferenceMode::TextGenerationAndEmbeddings));

        let subscriber = broadcaster.subscribe();

        broadcaster.publish(report(1, 2, InferenceMode::TextGenerationAndEmbeddings));

        assert_eq!(subscriber.has_changed().ok(), Some(false));
    }

    #[test]
    fn a_report_from_a_restarted_balancer_replaces_the_previous_one() {
        let broadcaster = ClusterInferenceModeBroadcaster::default();
        let restarted_balancer_report =
            report(2, 0, InferenceMode::TextGenerationEmbeddingsAndDecisions);

        broadcaster.publish(report(1, 5, InferenceMode::TextGenerationAndEmbeddings));
        broadcaster.publish(restarted_balancer_report);

        assert_eq!(
            *broadcaster.subscribe().borrow(),
            ReportedClusterInferenceMode::Reported(restarted_balancer_report)
        );
    }
}
