use std::mem::Discriminant;
use std::mem::discriminant;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::channel;
use std::thread::spawn;

use paddler_agent_runtime::scheduler_handle::SchedulerHandle;
use paddler_agent_runtime::scheduler_message::SchedulerMessage;

use crate::even_number_preparation::EvenNumberPreparation;
use crate::number_command::NumberCommand;
use crate::number_request_preparer::NumberRequestPreparer;

pub struct SchedulerAwaitingItsFirstMessage {
    pub first_message_rx:
        Receiver<Result<Discriminant<SchedulerMessage<NumberCommand>>, RecvError>>,
    pub scheduler_handle: SchedulerHandle<EvenNumberPreparation>,
}

impl SchedulerAwaitingItsFirstMessage {
    pub fn spawn() -> Self {
        let NumberRequestPreparer {
            request_preparer,
            scheduler_message_rx,
            ..
        } = NumberRequestPreparer::new();
        let (first_message_tx, first_message_rx) = channel();
        let scheduler_thread_handle = spawn(move || {
            first_message_tx
                .send(
                    scheduler_message_rx
                        .recv()
                        .map(|scheduler_message| discriminant(&scheduler_message)),
                )
                .expect("the test must wait for the scheduler thread");
        });

        Self {
            first_message_rx,
            scheduler_handle: SchedulerHandle {
                request_preparer,
                scheduler_thread_handle,
            },
        }
    }

    pub async fn assert_shut_down_by(
        self,
        shut_down: impl AsyncFnOnce(SchedulerHandle<EvenNumberPreparation>),
    ) {
        shut_down(self.scheduler_handle).await;

        assert_eq!(
            self.first_message_rx.try_recv(),
            Ok(Ok(discriminant(
                &SchedulerMessage::<NumberCommand>::Shutdown
            )))
        );
    }
}
