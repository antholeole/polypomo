mod client;
mod server;
mod cli;

use tokio;

use clap::Parser;

use cli::{Invocation, Commands};
use server::PolydoroServer;
use client::{send_polydoro_message, OpCode};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Invocation::parse();

    Ok(match args.command {
        Commands::Skip { puid } => send_polydoro_message(puid, OpCode::Skip),
        Commands::Toggle { puid } => send_polydoro_message(puid, OpCode::Toggle),
        Commands::Run(run_args) => PolydoroServer::new(run_args).run().await?,
    })
}
