use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityPlan<TServed> {
    Keep(TServed),
    Start,
}

impl<TServed: Clone> CapabilityPlan<TServed> {
    #[must_use]
    pub fn between<TSettings: PartialEq>(
        served: &TServed,
        served_settings: &TSettings,
        desired_settings: &TSettings,
    ) -> Self {
        if served_settings == desired_settings {
            Self::Keep(served.clone())
        } else {
            Self::Start
        }
    }
}

impl<TServed> ShutsDownUnlessShared for CapabilityPlan<TServed>
where
    TServed: ShutsDownUnlessShared + Send,
{
    async fn shut_down_unless_shared(self) {
        match self {
            Self::Keep(served) => served.shut_down_unless_shared().await,
            Self::Start => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CapabilityPlan;

    #[test]
    fn keeps_a_capability_whose_settings_did_not_change() {
        assert_eq!(
            CapabilityPlan::between(&"served", &1, &1),
            CapabilityPlan::Keep("served")
        );
    }

    #[test]
    fn restarts_a_capability_whose_settings_changed() {
        assert_eq!(
            CapabilityPlan::between(&"served", &1, &2),
            CapabilityPlan::Start
        );
    }
}
