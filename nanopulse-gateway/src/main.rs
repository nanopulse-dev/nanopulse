use std::env;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use tracing::info;

mod config;
mod gateway;
mod radio;
mod reset;
mod traits;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting NanoPulse Gateway"
    );

    let cli = Cli::parse();
    let config_path = Path::new(&cli.config);
    let conf = config::get(config_path)?;

    let mut radio = radio::get(
        &conf.gateway.chipset_vendor,
        &conf.gateway.chipset,
        &conf.gateway.board_mapping,
        &conf.get_board_config_file_path(config_path),
    )
    .await?;

    gateway::start(&conf.gateway.name, radio, &conf.mqtt).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
