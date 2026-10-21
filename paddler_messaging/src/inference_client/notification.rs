use serde::Deserialize;
use serde::Serialize;

use crate::cluster_inference_mode_report::ClusterInferenceModeReport;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum Notification {
    ClusterInferenceMode(ClusterInferenceModeReport),
}
