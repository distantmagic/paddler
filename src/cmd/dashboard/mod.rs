use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::{io::stdout, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{runtime::Runtime, sync::Mutex, time::interval};


use crate::{
    cmd::dashboard::app::App,
    errors::{app_error::AppError, result::Result},
};

pub mod app;
pub mod ui;

pub async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    let mut terminal = ratatui::init();
    let app = App::new()?;

    let app = Arc::new(Mutex::new(app));

    let update_app_state: Arc<Mutex<App>> = Arc::clone(&app);
    let management_clone = management_addr.clone();
    let update_handle = tokio::spawn(async move {
        let update_interval = Duration::from_millis(100);
        loop {
            interval(update_interval.clone());
            let mut app = update_app_state.lock().await;
            if app.needs_to_stop {
                break Ok::<(), AppError>(());
            }
            app.update_registered_agents(management_clone).await.ok();
            app.set_needs_rendering(true);
        }
    });

    let render_app_state: Arc<Mutex<App>> = Arc::clone(&app);
    let render_handle = tokio::spawn(async move {
        let render_interval = Duration::from_millis(100);
        loop {
            interval(render_interval.clone());

            let mut app = render_app_state.lock().await;

            if app.needs_to_stop {
                stops_rendering(&mut terminal)?;
                break Ok::<(), AppError>(());
            }

            if app.needs_rendering() {
                terminal.try_draw(|frame| app.draw(frame))?;
                app.set_needs_rendering(false);
            }
        }
    });

    let render_keyboard_app: Arc<Mutex<App>> = Arc::clone(&app);
    let events_handle = tokio::spawn(async move {
        loop {
            if let Event::Key(key) = event::read()? {
                let mut app = render_keyboard_app.lock().await;
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.needs_to_stop = true;
                            return Ok::<(), AppError>(());
                        }
                        KeyCode::Char('j') | KeyCode::Down => {app.next_row();},
                        KeyCode::Char('k') | KeyCode::Up => {app.previous_row();},
                        _ => {}
                    }
                }
            }
        }
    });

    let increase_ticks_app: Arc<Mutex<App>> = Arc::clone(&app);
    let ticks_handle = tokio::spawn(async move {
        let time_duration = Duration::from_millis(500);
        loop {
            interval(time_duration);
            let mut app = increase_ticks_app.lock().await;

            if app.needs_to_stop {
                break Ok::<(), AppError>(());
            }

            app.increase_ticks();
        }
    });

    let join = tokio::try_join!(update_handle, render_handle, events_handle, ticks_handle)?;

    ratatui::restore();

    join.1
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;
    Ok(())
}

fn stops_rendering(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.clear()?;
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
