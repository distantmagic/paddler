use std::mem::discriminant;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread::spawn;

use paddler_agent_runtime::scheduler_handle::SchedulerHandle;
use paddler_agent_runtime::scheduler_message::SchedulerMessage;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared as _;

use crate::number_command::NumberCommand;
use crate::number_request_preparer::NumberRequestPreparer;

#[tokio::test]
async fn an_unshared_scheduler_handle_shuts_its_scheduler_down() {
    let NumberRequestPreparer {
        request_preparer,
        scheduler_message_rx,
        ..
    } = NumberRequestPreparer::new();
    let (shutdown_observed_tx, shutdown_observed_rx) = channel();
    let scheduler_thread_handle = spawn(move || {
        shutdown_observed_tx
            .send(
                scheduler_message_rx
                    .recv()
                    .map(|scheduler_message| discriminant(&scheduler_message)),
            )
            .expect("the test must wait for the scheduler thread");
    });

    Arc::new(SchedulerHandle {
        request_preparer,
        scheduler_thread_handle,
    })
    .shut_down_unless_shared()
    .await;

    assert_eq!(
        shutdown_observed_rx.try_recv(),
        Ok(Ok(discriminant(
            &SchedulerMessage::<NumberCommand>::Shutdown
        )))
    );
}
