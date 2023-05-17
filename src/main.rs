mod client;
mod server;
mod cli;

use clap::Parser;

use cli::{Invocation, Commands};
use server::PolydoroServer;
use client::{send_polydoro_message, OpCode};

fn main() {
    let args = Invocation::parse();

    match args.command {
        Commands::Skip { puid } => send_polydoro_message(puid, OpCode::Skip),
        Commands::Toggle { puid } => send_polydoro_message(puid, OpCode::Toggle),
        Commands::Run(run_args) => PolydoroServer::new(run_args)
        .run()
        .join()
        .expect("polydoro server crashed."),
    };
}
