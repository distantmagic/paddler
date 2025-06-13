mod balancer_world;
mod expression;
mod llamacpp_instance;

use cucumber::World as _;

use self::balancer_world::BalancerWorld;

#[tokio::main]
async fn main() {
    BalancerWorld::cucumber()
        .after(|_feature, _rule, _scenario, _ev, world| {
            Box::pin(async move {
                world.unwrap().cleanup().await;
            })
        })
        .fail_fast()
        .fail_on_skipped()
        .run("tests/features/balancer")
        .await;
}
