use std::sync::Arc;
use std::sync::mpsc::TryRecvError;
use std::thread::spawn;

use paddler_agent_runtime::scheduler_handle::SchedulerHandle;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared as _;

use crate::number_request_preparer::NumberRequestPreparer;

#[tokio::test]
async fn a_shared_scheduler_handle_keeps_its_scheduler_running() {
    let NumberRequestPreparer {
        request_preparer,
        scheduler_message_rx,
        ..
    } = NumberRequestPreparer::new();
    let scheduler_handle = Arc::new(SchedulerHandle {
        request_preparer,
        scheduler_thread_handle: spawn(|| {}),
    });

    scheduler_handle.clone().shut_down_unless_shared().await;

    assert_eq!(
        scheduler_message_rx.try_recv().err(),
        Some(TryRecvError::Empty)
    );

    drop(scheduler_handle);
}
