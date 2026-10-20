use std::sync::mpsc::RecvError;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;

use tokio::task::JoinHandle;
use tokio::task::spawn_blocking;

pub struct OccupiedBlockingThread {
    occupant: JoinHandle<Result<(), RecvError>>,
    release_tx: Sender<()>,
}

impl OccupiedBlockingThread {
    pub fn occupy() -> Self {
        let (release_tx, release_rx) = channel::<()>();

        Self {
            occupant: spawn_blocking(move || release_rx.recv()),
            release_tx,
        }
    }

    pub async fn release(self) {
        self.release_tx
            .send(())
            .expect("the blocking thread occupant must be waiting");
        self.occupant
            .await
            .expect("the blocking thread occupant must not panic")
            .expect("the blocking thread occupant must be released");
    }
}
