pub mod agent;
pub mod balancer;
pub mod supervisor;
pub mod utils;

#[cfg(test)]
mod tests {
    use super::agent::tests::run_cucumber_tests as run_agent_tests;
    use super::balancer::tests::run_cucumber_tests as run_balancer_tests;
    use super::supervisor::tests::run_cucumber_tests as run_supervisor_tests;
    use super::utils::utils::PaddlerWorld;

    #[tokio::test]
    async fn test_all() {
        PaddlerWorld::setup().expect("Failed to setup");
        run_agent_tests().await;
        run_balancer_tests().await;
        run_supervisor_tests().await;
    }
}
