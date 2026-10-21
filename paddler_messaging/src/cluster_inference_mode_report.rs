use serde::Deserialize;
use serde::Serialize;

use crate::inference_mode::InferenceMode;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterInferenceModeReport {
    pub balancer_instance: String,
    pub inference_mode: InferenceMode,
    pub version: u64,
}

impl ClusterInferenceModeReport {
    #[must_use]
    pub fn is_a_later_version_of(&self, earlier_report: &Self) -> bool {
        self.balancer_instance == earlier_report.balancer_instance
            && self.version > earlier_report.version
    }
}

#[cfg(test)]
mod tests {
    use super::ClusterInferenceModeReport;
    use crate::inference_mode::InferenceMode;

    fn report(balancer_instance: &str, version: u64) -> ClusterInferenceModeReport {
        ClusterInferenceModeReport {
            balancer_instance: balancer_instance.to_owned(),
            inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            version,
        }
    }

    #[test]
    fn a_later_version_of_the_same_balancer_is_a_later_version() {
        assert!(report("first", 3).is_a_later_version_of(&report("first", 2)));
    }

    #[test]
    fn an_earlier_or_equal_version_of_the_same_balancer_is_not_a_later_version() {
        assert!(!report("first", 2).is_a_later_version_of(&report("first", 3)));
        assert!(!report("first", 3).is_a_later_version_of(&report("first", 3)));
    }

    #[test]
    fn a_report_of_another_balancer_instance_is_not_a_later_version_whatever_its_version() {
        assert!(!report("restarted", 9).is_a_later_version_of(&report("first", 5)));
    }
}
