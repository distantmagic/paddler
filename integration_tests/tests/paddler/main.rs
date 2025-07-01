mod agent_response;
mod agents_collection;
mod assert_balancer_table;
mod balancer_management_client;
mod cleanable;
mod expression;
mod llamacpp_instance;
mod llamacpp_instance_collection;
mod paddler_world;
mod request_builder;
mod request_headers_to_be_set;

use cucumber::World as _;

use self::cleanable::Cleanable as _;
use self::paddler_world::PaddlerWorld;

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
        .run_and_exit("tests/features/supervisor")
        .await;
}
