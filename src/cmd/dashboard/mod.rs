use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, net::SocketAddr, time::Duration};
use tokio::{
    runtime::Runtime,
    time::{interval, MissedTickBehavior},
};

use crate::{
    cmd::dashboard::{app::App, render::render, tui::Tui},
    errors::result::Result,
};

pub mod app;
pub mod render;
pub mod tui;

async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    let mut app = App::new(management_addr);
    let mut ticker = interval(Duration::from_millis(100));

    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut reader = EventStream::new();

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.init()?;

    while app.running {
        tui.draw(|frame| render(&mut app, frame))?;

        tokio::select! {
          _ = ticker.tick() => app.tick().await?,
          Some(Ok(evt)) = reader.next().fuse() => {
            match evt {
              Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key)?;
                }
              },
              _ => {}
            }
          }
        };
    }

    tui.exit()
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;

    Ok(())
}
