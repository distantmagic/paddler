use std::net::SocketAddr;
use tokio::runtime::Runtime;

use crate::{cmd::dashboard::app::App, errors::result::Result};

pub mod app;
pub mod ui;

pub async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    // let agents = get_registered_agents(management_addr).await?;

    let terminal = ratatui::init();
    let mut app = App::new()?;
    let app_result = app.clone().run(terminal).await?;
    app.update_registered_agents(management_addr).await?;
    ratatui::restore();
    Ok(app_result)
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;
    Ok(())
}
