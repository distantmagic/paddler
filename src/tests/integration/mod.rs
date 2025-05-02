pub mod agent;
pub mod balancer;
pub mod supervisor;
pub mod utils;

#[cfg(test)]
mod tests {
    // use super::agent::tests::run_agent_tests;
    // use super::balancer::tests::run_balancer_tests;
    use super::supervisor::tests::run_supervisor_tests;
    // use super::utils::utils::PaddlerWorld;

    #[tokio::test]
    async fn test_all() {
        // PaddlerWorld::setup().await.expect("Failed to setup");
        run_supervisor_tests().await;
        // run_agent_tests().await;
        // run_balancer_tests().await;
    }
}
