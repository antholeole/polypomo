mod client;
mod server;
mod cli;

use clap::Parser;

use cli::{Invocation, Commands};
use server::PolypomoServer;


#[tokio::main]
async fn main() {
    let args = Invocation::parse();

    match args.command {
        Commands::Run(runArgs) => PolypomoServer::new(runArgs),
    };

    println!("Hello, world!");
}
