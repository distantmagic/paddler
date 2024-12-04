use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::{io::stdout, net::SocketAddr};
use tokio::{
    runtime::Runtime,
    sync::{
        broadcast,
        mpsc::{self},
    },
    task::JoinHandle,
    time::{interval, Duration, MissedTickBehavior},
};

use crate::{
    balancer::upstream_peer_pool::UpstreamPeerPool, cmd::dashboard::app::App,
    errors::result::Result,
};

pub mod app;
pub mod ui;

async fn fetch_registered_agents(management_addr: SocketAddr) -> Result<UpstreamPeerPool> {
    let response_string = reqwest::get(format!(
        "http://{}/api/v1/agents",
        management_addr.to_string().as_str()
    ))
    .await?
    .text()
    .await?;

    Ok(serde_json::from_str(response_string.as_str())?)
}

pub async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    let mut terminal = ratatui::init();

    let management_clone = management_addr.clone();

    let (app_needs_to_stop_tx, mut app_needs_to_stop_rx_update) = broadcast::channel::<bool>(1);
    let (upstream_peer_pool_tx, mut upstream_peer_pool_rx) = mpsc::channel::<UpstreamPeerPool>(1);
    let (app_needs_to_render_app_error_tx, mut app_needs_to_render_error_rx) =
        mpsc::channel::<String>(1);

    let update_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(500));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = app_needs_to_stop_rx_update.recv() => {
                    break Ok(())
                },
                _ = ticker.tick() => {
                    let upstream_peer_pool = fetch_registered_agents(management_clone).await;

                    match upstream_peer_pool {
                        Ok(upstream_peer_pool) => {
                            if let Err(err) = upstream_peer_pool_tx.send(upstream_peer_pool).await {
                                app_needs_to_render_app_error_tx.send(format!("Error sending upstream peer pool - {}", err.to_string())).await.ok();
                            }
                        },
                        Err(err) => {
                            app_needs_to_render_app_error_tx.send(format!("Error fetching agents - {}", err.to_string())).await.ok();
                        }
                    }
                }
            }
        }
    });

    let mut app_needs_to_stop_rx_render = app_needs_to_stop_tx.subscribe();

    let render_handle = tokio::spawn(async move {
        let mut app = App::new()?;
        let mut reader = EventStream::new();

        loop {
            terminal.try_draw(|frame| app.draw(frame))?;

            tokio::select! {
                _ = app_needs_to_stop_rx_render.recv() => {
                    stop_rendering(&mut terminal)?;

                    return Ok(())
                },
                Some(app_error) = app_needs_to_render_error_rx.recv() => {
                    app.error = Some(app_error);
                },
                Some(upstream_peer_pool) = upstream_peer_pool_rx.recv() => {
                    app.set_registered_agents(upstream_peer_pool)?;
                },
                Some(Ok(evt)) = reader.next().fuse() => {
                    match evt {
                        Event::Resize(_, _) => {},
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        app_needs_to_stop_tx.send(true).ok();
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => app.next_row(),
                                    KeyCode::Char('k') | KeyCode::Up => app.previous_row(),
                                    _ => {}
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    });

    let join = tokio::try_join!(update_handle, render_handle)?;

    ratatui::restore();

    join.1
}

fn stop_rendering(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.clear()?;
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;
    Ok(())
}
