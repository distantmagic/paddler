use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportedClusterInferenceMode {
    NotYetReported,
    Reported(ClusterInferenceModeReport),
}
