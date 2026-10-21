use serde::Deserialize;
use serde::Serialize;

use crate::inference_mode::InferenceMode;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterInferenceModeReport {
    pub balancer_instance: u32,
    pub inference_mode: InferenceMode,
    pub version: u64,
}

impl ClusterInferenceModeReport {
    #[must_use]
    pub const fn supersedes(&self, earlier_report: &Self) -> bool {
        self.balancer_instance != earlier_report.balancer_instance
            || self.version > earlier_report.version
    }
}

#[cfg(test)]
mod tests {
    use super::ClusterInferenceModeReport;
    use crate::inference_mode::InferenceMode;

    const fn report(balancer_instance: u32, version: u64) -> ClusterInferenceModeReport {
        ClusterInferenceModeReport {
            balancer_instance,
            inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            version,
        }
    }

    #[test]
    fn a_later_version_of_the_same_balancer_supersedes_an_earlier_one() {
        assert!(report(7, 3).supersedes(&report(7, 2)));
    }

    #[test]
    fn an_earlier_or_equal_version_of_the_same_balancer_does_not_supersede() {
        assert!(!report(7, 2).supersedes(&report(7, 3)));
        assert!(!report(7, 3).supersedes(&report(7, 3)));
    }

    #[test]
    fn a_report_from_a_restarted_balancer_supersedes_whatever_its_version() {
        assert!(report(8, 0).supersedes(&report(7, 5)));
    }
}
