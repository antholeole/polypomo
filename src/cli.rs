use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Invocation {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run polydomo.
    Run(RunArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct RunArgs {
    #[arg(short, long, default_value_t = 1)]
    puid: u8,

    #[arg(long, default_value = "󰒲")]
    sleeping_icon: String,

    #[arg(long, default_value = "󰱠")]
    working_icon: String,

    #[arg(long, default_value = "")]
    paused_icon: String,
}