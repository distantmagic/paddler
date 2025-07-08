use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;

pub struct LlamaCppProcess {
    applicable_state: LlamaCppApplicableState,
    child_process: Mutex<Option<Child>>,
    is_dead: AtomicBool,
    is_healthy: AtomicBool,
    llamacpp_listen_addr: SocketAddr,
    llamacpp_server_bin_path: PathBuf,
}

impl LlamaCppProcess {
    pub fn new(
        applicable_state: LlamaCppApplicableState,
        llamacpp_listen_addr: SocketAddr,
        llamacpp_server_bin_path: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            applicable_state,
            child_process: Mutex::new(None),
            is_dead: AtomicBool::new(false),
            is_healthy: AtomicBool::new(false),
            llamacpp_listen_addr,
            llamacpp_server_bin_path,
        })
    }

    pub async fn check_health(&self) -> Result<()> {
        if self.is_dead() {
            return Ok(());
        }

        let mut child_process = self.child_process.lock().await;

        if let Some(ref mut child) = *child_process {
            match child.try_wait()? {
                Some(_) => {
                    self.is_dead.store(true, Ordering::Relaxed);
                    self.is_healthy.store(false, Ordering::Relaxed);
                }
                None => {
                    self.is_dead.store(false, Ordering::Relaxed);
                    self.is_healthy.store(true, Ordering::Relaxed);
                }
            }
        } else {
            self.is_dead.store(false, Ordering::Relaxed);
            self.is_healthy.store(false, Ordering::Relaxed);
        }

        Ok(())
    }

    pub fn is_dead(&self) -> bool {
        self.is_dead.load(Ordering::Relaxed)
    }

    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }

    pub async fn shutdown(&self, signal: Signal) -> Result<()> {
        let mut child_process = self.child_process.lock().await;

        if let Some(mut child) = child_process.take() {
            if let Some(pid) = child.id() {
                let nix_pid = Pid::from_raw(pid as i32);

                signal::kill(nix_pid, signal)?;

                let _ = child.wait().await?;
            }
        }

        Ok(())
    }

    pub async fn spawn(&self) -> Result<()> {
        let child = Command::new(self.llamacpp_server_bin_path.clone())
            .kill_on_drop(true)
            .arg("--host")
            .arg(self.llamacpp_listen_addr.ip().to_string())
            .arg("--port")
            .arg(self.llamacpp_listen_addr.port().to_string())
            .arg("--model")
            .arg(self.applicable_state.model_path.clone())
            .spawn()?;

        {
            let mut child_process = self.child_process.lock().await;

            *child_process = Some(child);
        }

        Ok(())
    }
}
