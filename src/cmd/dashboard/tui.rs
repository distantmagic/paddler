use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::Backend, Frame, Terminal};
use std::io;

use crate::errors::result::Result;

#[derive(Debug)]
pub struct Tui<TBackend: Backend> {
    terminal: Terminal<TBackend>,
}

impl<TBackend: Backend> Tui<TBackend> {
    pub fn new(terminal: Terminal<TBackend>) -> Self {
        Self { terminal }
    }

    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    #[inline]
    pub fn draw<TRender>(&mut self, render_fn: TRender) -> Result<()>
    where
        TRender: FnOnce(&mut Frame),
    {
        self.terminal.draw(|frame| render_fn(frame))?;

        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
