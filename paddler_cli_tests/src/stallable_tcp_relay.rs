use std::future::pending;
use std::io;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

use anyhow::Result;
use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::spawn;
use tokio::sync::watch;
use tokio::sync::watch::error::RecvError;
use tokio::task::JoinHandle;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

async fn wait_for_a_later_stall(
    mut stall_generation_rx: watch::Receiver<u32>,
    accepted_during_stall_generation: u32,
) -> Result<(), RecvError> {
    stall_generation_rx
        .wait_for(|stall_generation| *stall_generation > accepted_during_stall_generation)
        .await
        .map(drop)
}

async fn relay_until_stalled(
    mut downstream: TcpStream,
    upstream_addr: SocketAddr,
    stall_generation_rx: watch::Receiver<u32>,
    accepted_during_stall_generation: u32,
) -> io::Result<()> {
    let mut upstream = TcpStream::connect(upstream_addr).await?;

    select! {
        relay_outcome = copy_bidirectional(&mut downstream, &mut upstream) => relay_outcome.map(drop),
        stall_outcome = wait_for_a_later_stall(stall_generation_rx, accepted_during_stall_generation) => {
            stall_outcome.map_err(io::Error::other)?;

            pending().await
        }
    }
}

async fn relay_accepted_connections(
    listener: TcpListener,
    upstream_addr: SocketAddr,
    accepted_connection_count_tx: watch::Sender<u32>,
    stall_generation_rx: watch::Receiver<u32>,
    stop: CancellationToken,
) -> io::Result<()> {
    let mut relayed_connections = JoinSet::new();

    loop {
        let (downstream, _downstream_addr) = select! {
            () = stop.cancelled() => break Ok(()),
            accepted_connection = listener.accept() => accepted_connection?,
        };
        let accepted_during_stall_generation = *stall_generation_rx.borrow();

        accepted_connection_count_tx.send_modify(|accepted_connection_count| {
            *accepted_connection_count += 1;
        });
        relayed_connections.spawn(relay_until_stalled(
            downstream,
            upstream_addr,
            stall_generation_rx.clone(),
            accepted_during_stall_generation,
        ));
    }
}

pub struct StallableTcpRelay {
    pub addr: SocketAddr,
    accepted_connection_count_rx: watch::Receiver<u32>,
    relaying_task: JoinHandle<io::Result<()>>,
    stall_generation_tx: watch::Sender<u32>,
    stop: CancellationToken,
}

impl StallableTcpRelay {
    pub async fn start(upstream_addr: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await?;
        let addr = listener.local_addr()?;
        let (accepted_connection_count_tx, accepted_connection_count_rx) = watch::channel(0);
        let (stall_generation_tx, stall_generation_rx) = watch::channel(0);
        let stop = CancellationToken::new();
        let relaying_task = spawn(relay_accepted_connections(
            listener,
            upstream_addr,
            accepted_connection_count_tx,
            stall_generation_rx,
            stop.clone(),
        ));

        Ok(Self {
            addr,
            accepted_connection_count_rx,
            relaying_task,
            stall_generation_tx,
            stop,
        })
    }

    pub fn stall_established_connections(&self) {
        self.stall_generation_tx
            .send_modify(|stall_generation| *stall_generation += 1);
    }

    pub async fn wait_for_accepted_connections(
        &self,
        expected_accepted_connection_count: u32,
    ) -> Result<(), RecvError> {
        self.accepted_connection_count_rx
            .clone()
            .wait_for(|accepted_connection_count| {
                *accepted_connection_count >= expected_accepted_connection_count
            })
            .await
            .map(drop)
    }

    pub async fn stop(self) -> Result<()> {
        self.stop.cancel();
        self.relaying_task.await??;

        Ok(())
    }
}
