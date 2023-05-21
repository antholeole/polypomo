use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Invocation {
    #[command(subcommand)]
    pub command: Commands,
}

const POLYDORO_SOCKET_NAME: &str = "/tmp/polydoro";

#[derive(Subcommand)]
pub enum Commands {
    /// Runs polydoro.
    /// 
    /// This starts a polydoro server on a local socket, specified by puid. 
    Run(RunArgs),

    /// Pauses / unpauses another polydoro timer that is running.
    Toggle {
        /// Specifies which polydoro timer to toggle. 
        /// 
        /// This default corresponds to the default in "run".
        /// If running on the non-default port, make sure to specify this.
        #[arg(short, long, default_value = POLYDORO_SOCKET_NAME)]
        puid: String,
    },

    Skip {
        /// Entirely skips the current polydoro state. If resting, will go to a work state
        /// and visa-versa. Also increments the counter as normal.
        #[arg(short, long, default_value = POLYDORO_SOCKET_NAME)]
        puid: String,
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct RunArgs {
    /// The socket that the polydoro timer runs on. 
    /// 
    /// By default, runs in a socket caled "polydoro". This socket will appear as a file
    /// in the directory that it points to (ex. if left default, /tmp/polydoro). 
    /// This should be an ABSOLUTE PATH, or else the socket will be created in whatever
    /// dir polydomo is executed from. 
    #[arg(short, long, default_value = POLYDORO_SOCKET_NAME)]
    pub puid: String,

    /// the icon that is displayed when currently resting. 
    /// 
    /// For use with wide unicode characters, prepend and append spaces,
    /// i.e. " 󰒲 ".
    #[arg(long, default_value = " 󰒲 ")]
    pub sleeping_icon: String,

    /// the icon that is displayed when currently working. 
    /// 
    /// For use with wide unicode characters, prepend and append spaces,
    /// i.e. " 󰱠 ".
    #[arg(long, default_value = " 󰱠 ")]
    pub working_icon: String,

    /// the icon that is displayed when currently paused. 
    /// 
    /// For use with wide unicode characters, prepend and append spaces,
    /// i.e. "  ".
    #[arg(long, default_value = "  ")]
    pub paused_icon: String,

    /// The length of time spent resting during a regular rest. 
    /// 
    /// This value is set in seconds. Default is 5 minutes (5 * 60).
    #[arg(long, default_value_t = 60 * 5)]
    pub break_period_s: u16,

    /// The length of time spent during a work period. Set in seconds.
    /// 
    /// By default, this is 25 minutes. 
    #[arg(long, default_value_t = 60 * 25)]
    pub work_period_s: u16,

    /// The length of time spent during a long break.
    #[arg(long, default_value_t = 60 * 30)]
    pub long_break_period_s: u16,

    /// how many cycles before a long break. Defaults to four 
    /// (ex. breaks are: 5, 5, 5, 5, 30, each with a 25 minute work period between.)
    #[arg(long, default_value_t = 4)]
    pub cycles: i8,

    /// How quickly the server writes the time to the polybar. Unit is ms.
    /// 
    ///  Defaults to 500ms. To minimize "time skip" effects, set a number that multiples well into
    /// 1. (500, 250, 100, 50, etc.). This way, there won't be longer periods of non-updating.
    #[arg(long, default_value_t = 500)]
    pub refresh_rate_ms: u64,
}