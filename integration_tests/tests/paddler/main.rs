mod agent_instance_collection;
mod agent_response;
mod assert_balancer_table;
mod balancer_instance;
mod balancer_management_client;
mod cleanable;
mod expression;
mod metrics;
mod paddler_world;
mod request_builder;
mod request_headers_to_be_set;
mod retry_until_success;
mod state_database_configuration;
mod statsd_instance;

use cucumber::World as _;

use self::cleanable::Cleanable as _;
use self::paddler_world::PaddlerWorld;

pub const BALANCER_PORT: u16 = 8095;
pub const MOCK_LLAMACPP_SERVER_PATH: &str = "./tests/fixtures/llamacpp-server-mock.mjs";
pub const REVERSE_PROXY_PORT: u16 = 8096;

#[tokio::main]
async fn main() {
    PaddlerWorld::cucumber()
        .after(|_feature, _rule, _scenario, _ev, world| {
            Box::pin(async move {
                world
                    .unwrap()
                    .cleanup()
                    .await
                    .expect("Failed to clean up world");
            })
        })
        .fail_fast()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
