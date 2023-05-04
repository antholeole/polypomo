mod client;
mod server;
mod cli;

use clap::Parser;

use cli::{Invocation, Commands};
use server::PolypomoServer;
use client::{send_polydoro_message, OpCode};

fn main() {
    let args = Invocation::parse();

    match args.command {
        Commands::Toggle { puid } => send_polydoro_message(puid, OpCode::Toggle),
        Commands::Run(runArgs) => PolypomoServer::new(runArgs)
        .run()
        .join()
        .expect("Polypomo server crashed."),
    };
}
