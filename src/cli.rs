use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Invocation {
    #[command(subcommand)]
    pub command: Commands,
}

const POLYDORO_SOCKET_NAME: &str = "polydoro";

#[derive(Subcommand)]
pub enum Commands {
    /// Run polydomo.
    Run(RunArgs),

    Toggle {
        #[arg(short, long, default_value = POLYDORO_SOCKET_NAME)]
        puid: String,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct RunArgs {
    #[arg(short, long, default_value = POLYDORO_SOCKET_NAME)]
    pub puid: String,

    #[arg(long, default_value = "󰒲 ")]
    pub sleeping_icon: String,

    #[arg(long, default_value = "󰱠 ")]
    pub working_icon: String,

    #[arg(long, default_value = " ")]
    pub paused_icon: String,

    #[arg(long, default_value_t = 60 * 5)]
    pub rest_period_s: u16,

    #[arg(long, default_value_t = 60 * 25)]
    pub work_period_s: u16,

    #[arg(long, default_value_t = 60 * 30)]
    pub break_period_s: u16,

    /// how many cycles before a long break. Defaults to four 
    /// (so breaks are: 5, 5, 5, 5, 25)
    #[arg(long, default_value_t = 4)]
    pub cycles: u16,

    #[arg(long, default_value_t = 750)]
    pub refresh_rate_ms: u64, 
}