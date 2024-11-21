use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::net::SocketAddr;

use crate::errors::result::Result;

#[derive(Debug)]
pub struct App {
    pub running: bool,
}

impl Default for App {
    fn default() -> Self {
        Self { running: true }
    }
}

impl App {
    pub fn new(management_addr: &SocketAddr) -> Self {
        Self { running: true }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.quit();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn tick(&self) -> Result<()> {
        Ok(())
    }
}
