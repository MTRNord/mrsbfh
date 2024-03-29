use crate::config::Config;
use clap::Parser;
use mrsbfh::config::Loader;
use std::error::Error;
use tracing::*;

pub mod commands;
mod config;
mod errors;
mod matrix;

#[derive(Parser, Debug)]
#[clap(version = "0.1.0", author = "MTRNord")]
struct Opts {
    #[clap(short, long, default_value = "config.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting...");
    let opts: Opts = Opts::parse();

    info!("Loading Configs...");
    let config = Config::load(opts.config)?;
    info!("Setting up Client...");
    let client = &mut matrix::setup(config.clone()).await?;
    info!("Starting Sync...");
    matrix::start_sync(client, config).await?;

    Ok(())
}
