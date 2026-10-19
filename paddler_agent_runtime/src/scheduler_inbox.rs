use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::TryRecvError;

use crate::scheduler_message::SchedulerMessage;

pub struct SchedulerInbox<TCommand> {
    pub scheduler_message_rx: Receiver<SchedulerMessage<TCommand>>,
    pub shutdown_requested: bool,
}

impl<TCommand> SchedulerInbox<TCommand> {
    #[must_use]
    pub const fn new(scheduler_message_rx: Receiver<SchedulerMessage<TCommand>>) -> Self {
        Self {
            scheduler_message_rx,
            shutdown_requested: false,
        }
    }

    pub fn take_next_waiting_command(&mut self) -> Option<TCommand> {
        match self.scheduler_message_rx.try_recv() {
            Ok(SchedulerMessage::Command(command)) => Some(command),
            Ok(SchedulerMessage::Shutdown) | Err(TryRecvError::Disconnected) => {
                self.shutdown_requested = true;

                None
            }
            Err(TryRecvError::Empty) => None,
        }
    }

    pub fn wait_for_next_command(&mut self) -> Option<TCommand> {
        match self.scheduler_message_rx.recv() {
            Ok(SchedulerMessage::Command(command)) => Some(command),
            Ok(SchedulerMessage::Shutdown) | Err(RecvError) => {
                self.shutdown_requested = true;

                None
            }
        }
    }
}
