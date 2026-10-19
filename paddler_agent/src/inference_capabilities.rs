use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCapabilities<TDecisions, TEmbeddings, TTextGeneration> {
    pub decisions: TDecisions,
    pub embeddings: TEmbeddings,
    pub text_generation: TTextGeneration,
}

impl<TDecisions, TEmbeddings, TTextGeneration> ShutsDownUnlessShared
    for InferenceCapabilities<TDecisions, TEmbeddings, TTextGeneration>
where
    TDecisions: ShutsDownUnlessShared + Send,
    TEmbeddings: ShutsDownUnlessShared + Send,
    TTextGeneration: ShutsDownUnlessShared + Send,
{
    async fn shut_down_unless_shared(self) {
        let Self {
            decisions,
            embeddings,
            text_generation,
        } = self;

        decisions.shut_down_unless_shared().await;
        embeddings.shut_down_unless_shared().await;
        text_generation.shut_down_unless_shared().await;
    }
}
