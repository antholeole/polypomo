mod client;
mod server;
mod cli;

use clap::Parser;

use cli::{Invocation, Commands};


fn main() {
    let args = Invocation::parse();

    match args.command {
        Commands::Run(runArgs) => println!("hi"),
    };

    println!("Hello, world!");
}
