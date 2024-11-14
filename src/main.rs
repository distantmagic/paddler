use clap::{value_parser, Parser, Subcommand};
use std::net::SocketAddr;

use crate::errors::result::Result;

mod agent;
mod balancer;
mod cmd;
mod errors;
mod llamacpp;

fn parse_url(s: &str) -> Result<url::Url> {
    Ok(url::Url::parse(s)?)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Agent {
        #[arg(long, value_parser = parse_url)]
        external_llamacpp_addr: url::Url,

        #[arg(long, value_parser = parse_url)]
        local_llamacpp_addr: url::Url,

        #[arg(long)]
        local_llamacpp_api_key: Option<String>,

        #[arg(long, value_parser = parse_url)]
        management_addr: url::Url,

        #[arg(long)]
        name: Option<String>,
    },
    Balancer {
        #[arg(long, value_parser = value_parser!(SocketAddr))]
        management_socket_addr: SocketAddr,

        #[arg(long, value_parser = value_parser!(SocketAddr))]
        reverseproxy_socket_addr: SocketAddr,
    },
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Agent {
            external_llamacpp_addr,
            local_llamacpp_addr,
            local_llamacpp_api_key,
            management_addr,
            name,
        }) => {
            cmd::agent::handle(
                external_llamacpp_addr,
                local_llamacpp_addr,
                local_llamacpp_api_key,
                management_addr,
                name,
            )
            .await?;
        }
        Some(Commands::Balancer {
            management_socket_addr,
            reverseproxy_socket_addr,
        }) => {
            cmd::balancer::handle(management_socket_addr, reverseproxy_socket_addr).await?;
        }
        None => {}
    }

    Ok(())
}
