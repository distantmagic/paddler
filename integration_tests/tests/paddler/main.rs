mod agent_response;
mod agents_collection;
mod expression;
mod llamacpp_instance;
mod llamacpp_instance_collection;
mod paddler_world;

use cucumber::World as _;

use self::paddler_world::PaddlerWorld;

#[tokio::main]
async fn main() {
    PaddlerWorld::cucumber()
        .after(|_feature, _rule, _scenario, _ev, world| {
            Box::pin(async move {
                world.unwrap().cleanup().await;
            })
        })
        .fail_fast()
        .fail_on_skipped()
        .run_and_exit("tests/features/agent")
        .await;
}
