use clap::{Parser, Subcommand};

use crate::errors::result::Result;

mod agent;
mod balancer;
mod cmd;
mod llamacpp;
mod errors;

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
        #[arg(long, value_parser = parse_url)]
        management_addr: url::Url,

        #[arg(long, value_parser = parse_url)]
        reverseproxy_addr: url::Url,
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Agent{
            external_llamacpp_addr: _,
            local_llamacpp_addr,
            local_llamacpp_api_key,
            management_addr,
            name,
        }) => {
            cmd::agent::handle(
                local_llamacpp_addr,
                local_llamacpp_api_key,
                management_addr,
                name,
            ).await?;
        }
        Some(Commands::Balancer{
            management_addr,
            reverseproxy_addr,
        }) => {
            cmd::balancer::handle(
                management_addr,
                reverseproxy_addr,
            ).await?;
        }
        None => {}
    }

    Ok(())
}
