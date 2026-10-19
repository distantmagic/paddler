use crate::scheduler_awaiting_its_first_message::SchedulerAwaitingItsFirstMessage;

#[tokio::test]
async fn shutting_down_stops_the_scheduler_and_waits_for_its_thread() {
    SchedulerAwaitingItsFirstMessage::spawn()
        .assert_shut_down_by(async |scheduler_handle| scheduler_handle.shut_down().await)
        .await;
}
