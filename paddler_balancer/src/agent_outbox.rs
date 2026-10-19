use std::collections::VecDeque;

use parking_lot::Mutex;
use tokio::sync::Notify;

use paddler_messaging::management_socket::agent::message::Message as AgentJsonRpcMessage;

#[derive(Default)]
pub struct AgentOutbox {
    message_arrived: Notify,
    messages: Mutex<VecDeque<AgentJsonRpcMessage>>,
}

impl AgentOutbox {
    pub fn push(&self, message: AgentJsonRpcMessage) {
        self.messages.lock().push_back(message);
        self.message_arrived.notify_one();
    }

    pub async fn next(&self) -> AgentJsonRpcMessage {
        loop {
            let next_message = self.messages.lock().pop_front();

            if let Some(message) = next_message {
                return message;
            }

            self.message_arrived.notified().await;
        }
    }
}
