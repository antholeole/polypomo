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
    pub puid: u8,

    #[arg(long, default_value = "󰒲")]
    pub sleeping_icon: String,

    #[arg(long, default_value = "󰱠")]
    pub working_icon: String,

    #[arg(long, default_value = "")]
    pub paused_icon: String,

    #[arg(long, default_value_t = 60 * 5)]
    pub rest_period: u16,

    #[arg(long, default_value_t = 60 * 25)]
    pub work_period: u16,

    #[arg(long, default_value_t = 60 * 30)]
    pub break_period: u16,

    #[arg(long, default_value_t = 4)]
    pub cycles: u16,

    #[arg(long, default_value_t = 0.75)]
    pub refresh_rate: f32, 
}