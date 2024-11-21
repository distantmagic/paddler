use std::net::SocketAddr;

use crate::errors::result::Result;

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    println!("Dashboard command");

    Ok(())
}
