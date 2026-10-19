use std::sync::Arc;

use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared as _;

use crate::scheduler_awaiting_its_first_message::SchedulerAwaitingItsFirstMessage;

#[tokio::test]
async fn an_unshared_scheduler_handle_shuts_its_scheduler_down() {
    SchedulerAwaitingItsFirstMessage::spawn()
        .assert_shut_down_by(async |scheduler_handle| {
            Arc::new(scheduler_handle).shut_down_unless_shared().await;
        })
        .await;
}
