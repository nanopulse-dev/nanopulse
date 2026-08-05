#![allow(dead_code)]

use std::path::Path;
use std::process;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::{filter, fmt, prelude::*};

mod api;
mod cmd;
mod config;
mod errors;
mod flow;
mod gateway;
mod integration;
mod keys;
mod logging;
mod lua;
mod profile;
mod region;
mod storage;

#[cfg(test)]
mod test;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "DIR")]
    config_dir: Option<String>,

    #[arg(short, long, value_name = "DIR")]
    profiles_dir: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate key-pair
    GenerateKeyPair {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(config) = &cli.config_dir {
        config::load(Path::new(config).join("nanopulse-server.toml"))?;
    }

    if let Some(Commands::GenerateKeyPair {}) = &cli.command {
        cmd::generate_key_pair::run();
        process::exit(0);
    }

    if cli.config_dir.is_none() {
        return Err(anyhow!(
            "you must specify the path to the configuration directory using -c"
        ));
    }
    if cli.profiles_dir.is_empty() {
        return Err(anyhow!(
            "you must specify at least one profiles directory using -p"
        ));
    }

    {
        let conf = config::get();
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(logging::LoggingLayer::new())
            .with(filter::Targets::new().with_targets([(
                "nanopulse_server",
                Level::from_str(&conf.logging.level).unwrap(),
            )]))
            .init();
    }

    lua::setup(&cli.profiles_dir)?;
    storage::setup().await?;
    region::load_all()?;
    gateway::setup().await?;
    integration::setup().await?;
    api::setup().await?;

    Ok(())
}
