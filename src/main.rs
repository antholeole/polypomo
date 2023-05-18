mod client;
mod server;
mod cli;

use {
    tokio,
    clap::Parser,
    cli::{Invocation, Commands},
    server::PolydoroServer,
    client::{send_polydoro_message, OpCode},
    anyhow::Result
};


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Invocation::parse();

    match args.command {
        Commands::Skip { puid } => send_polydoro_message(puid, OpCode::Skip),
        Commands::Toggle { puid } => send_polydoro_message(puid, OpCode::Toggle),
        Commands::Run(run_args) => Ok(PolydoroServer::new(run_args).run().await?),
    }
}
