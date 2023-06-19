use std::{fmt, path::Path, fs::remove_file};

use anyhow::anyhow;
use etcetera::BaseStrategy;
use notify_rust::Notification;

use {
    std::{time::{Duration, Instant}, fmt::Display, sync::Arc},
    futures::{io::BufReader, AsyncReadExt},
    log::debug,
    anyhow::{Result, Error},
    tokio::{time::sleep, sync::RwLock},
    pausable_clock::PausableClock,
    interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream},
    tokio_graceful_shutdown::{SubsystemHandle, Toplevel, errors::GracefulShutdownError},
    crate::cli::RunArgs,
    crate::client::{OpCode, opcode_from_byte}
};

#[derive(PartialEq, Debug, Clone)]
enum PeriodType {
    Break,
    Work,
    LongBreak
}

impl fmt::Display for PeriodType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg_attr(test, derive(derive_builder::Builder))]
#[cfg_attr(test, builder(pattern = "owned", default))]
pub struct PolydoroServer {
    args: RunArgs,
    cycles: i8,
    clock: PausableClock,

    #[cfg_attr(test, builder(private))]
    current_period: PeriodType,
} 

impl Default for PolydoroServer {
    fn default() -> Self { 
        PolydoroServer { 
            args: RunArgs { 
                force: false,
                puid: "/tmp/FAKE_SOCK.sock".to_owned(), 
                sleeping_icon: " sd ".to_string(), 
                working_icon: " sd ".to_string(), 
                paused_icon: " sd ".to_string(), 
                break_period_s: 1, 
                work_period_s: 5, 
                long_break_period_s: 1, 
                cycles: 3, 
                refresh_rate_ms: 10 
            }, 
            cycles: 0, 
            clock: PausableClock::default(), 
            current_period: PeriodType::LongBreak 
        }       
    }
}


impl PolydoroServer {
    pub fn new(args: RunArgs) -> PolydoroServer {
        PolydoroServer {
            args,
            current_period: PeriodType::Work,
            clock: PausableClock::new(Duration::ZERO, false),            
            cycles: 0,
        }
    }

    fn establish_socket(&self) -> Result<LocalSocketListener> {
        let mut path = Path::new(&self.args.puid).to_owned();

        if !path.is_absolute() {
            path = etcetera::choose_base_strategy()
                .map_err(|e| anyhow::anyhow!("Path is non-absolute, and no data dir could be found: {}", e))?
                .data_dir()                
                .join(path);
        };

        if path.try_exists()? {
            debug!("socket on {} already exists", path.display());

            if self.args.force {
                debug!("force deleting socket {}", path.display());
                remove_file(path.clone())?;
            } else {
                return Err(anyhow!("Socket {} already exists, but force flag was not set.", path.display()));
            }
        };

        debug!("established socket on {}", path.display());
        Ok(LocalSocketListener::bind(path)?)
    }

    pub async fn run(self) -> Result<()> {
        let local_socket = self.establish_socket()?;


        let rw_self = Arc::new(RwLock::new(self));

        let rw_self_listener = rw_self.clone();
        let rw_self_tick = rw_self.clone();

        Toplevel::new()
        .start("Ticker", move |subsys| PolydoroServer::do_tick(subsys, rw_self_tick.clone()))
            .start("Event Listener", move |subsys| PolydoroServer::listen_to_requests(subsys, local_socket, rw_self_listener.clone()))
            .catch_signals()
            .handle_shutdown_requests(Duration::from_secs(2))
            .await
            .map_err(|e: GracefulShutdownError| anyhow::anyhow!(e
                    .get_subsystem_errors()
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", "))
                )
    }

    async fn listen_to_requests(
        subsys: SubsystemHandle, 
        local_socket: LocalSocketListener,
        this: Arc<RwLock<PolydoroServer>>,
    ) -> Result<()> {
        let socket_name = &this.read().await.args.puid.clone();


        let drive = async {
                debug!("Beginning event listener loop...");  
                              
                loop {
                    let incoming: LocalSocketStream = local_socket.accept().await?;
                    let (reader, _) = incoming.into_split();
                    let mut buf: [u8; 1] = [99];
                    let mut reader = BufReader::new(reader);
                    
                    // We only need one message, so we consume & discard the stream after reading.
                    reader.read_exact(&mut buf).await?;

                    debug!("Recieved event: {}", match opcode_from_byte(buf[0]) {
                        Err(_) => "Unknown opcode",
                        Ok(OpCode::Skip) => "Skip",
                        Ok(OpCode::Toggle) => "Toggle"
                    });

                    match opcode_from_byte(buf[0]) {
                        Ok(OpCode::Skip) => this.write().await.change_state(true)?,
                        Ok(OpCode::Toggle) => this.write().await.toggle_pause(),
                        Err(e) => return Err::<(), Error>(e),
                };
            }
        };
        

        tokio::select! {
            // cleanup the pipe on shutdown
            _ = subsys.on_shutdown_requested() => {
                debug!("Attempting to remove file socket...");
                tokio::fs::remove_file(socket_name).await?;
                debug!("Deleted file socket.");
            },
            _ = drive => {
                subsys.request_shutdown();
            }
        };


            Ok(())
    }

    async fn do_tick(        
        subsys: SubsystemHandle, 
        this: Arc<RwLock<PolydoroServer>>
    ) -> Result<()> { 
        let poll_time = Duration::from_millis(this.read().await.args.refresh_rate_ms);

        let drive = async { 
            debug!("Beginning tick loop...");

            loop {
                let start = Instant::now();
                let runtime = start.elapsed();

                if let Some(remaining) = poll_time.checked_sub(runtime) {
                    sleep(remaining).await;
                }

                if let Err(e) = this.write().await.tick() {
                    return e;
                }
                
                println!("{}", this.read().await);
            };
        };

        Ok(tokio::select! {
            // nothing to cleanup
            _ = subsys.on_shutdown_requested() => {
                debug!("Tick loop shutting down.");
            },
            _ = drive => {},
        }) 
    }

    pub fn tick(&mut self) -> Result<()> {
        // Again, seems like the pkg we used is broken. This is 
        // the intended behavior yet the boolean seems flipped.
        if !self.clock.is_paused() { 
            return Ok(());
        }

        let cycle_time_elapsed = Instant::from(self.clock.now()).elapsed();

        if cycle_time_elapsed <= self.get_period_length().into() {
            return Ok(());
        }  


        debug!("Changing state...");
        self.change_state(false)?;

        Ok(())
    }

    fn toggle_pause(&mut self) {
        if self.clock.is_paused() {
            self.clock.resume();
        } else {
            self.clock.pause();
        }
    }

    fn change_state(&mut self, forced: bool) -> Result<()> {
        let old_period = self.current_period.clone();

        let period_type = if self.current_period != PeriodType::Work {
            self.cycles += 1;
            PeriodType::Work
        } else if self.cycles >= self.args.cycles {
            self.cycles = -1;
            PeriodType::LongBreak
        } else {
            PeriodType::Break
        };

        // paused = false feels wrong but gets the correct behavior.
        self.clock = PausableClock::new(Duration::ZERO, false);
        self.current_period = period_type;

        if !forced {
            Notification::new()
                .summary(&format!("Polydoro: {} Completed", old_period))
                .body(&format!("Next up: {}", self.current_period))
                .appname("Polypomo")
                .show()?;
        };

        Ok(())
    }

    fn get_period_length(&self) -> Duration {
        Duration::from_secs(match self.current_period {
            PeriodType::Break => self.args.break_period_s,
            PeriodType::Work => self.args.work_period_s,
            PeriodType::LongBreak => self.args.long_break_period_s,
        }.into())
    }
}


impl Display for PolydoroServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // paused seems to be inverted in this package.
        let symbol = if !self.clock.is_paused() {
            &self.args.paused_icon
        } else if self.current_period == PeriodType::Work {
            &self.args.working_icon
        } else {
            &self.args.sleeping_icon
        }.to_owned();

        let time_elapsed = Instant::from(self.clock.now()).elapsed();
        let period_length = self.get_period_length();

        let seconds = match period_length.checked_sub(time_elapsed) {
            Some(t) => t.as_secs() + 1,
            None => 0,
        };

         write!(f, "{}{} ({})", 
            symbol, 
            format!("{}:{:02}", seconds / 60, seconds % 60),
            self.cycles + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use std::env::{set_var, remove_var};

    use rand::distributions::{Alphanumeric, DistString};
    use pausable_clock::PausableClock;
    use tokio::fs::try_exists;

    use crate::cli::RunArgsBuilder;

    use super::*;


    struct FileCleanup {
        file: String
    }

    impl Drop for FileCleanup {
        fn drop(&mut self) {
            remove_file(self.file.clone()).unwrap();
        }
    }

    #[test]
    fn tick_should_ok_if_paused() {
        let mut server_with_paused_clock = PolydoroServerBuilder::default()
            .clock(PausableClock::new(Duration::ZERO, false))
            .build()
            .unwrap();
        
        assert!(server_with_paused_clock.tick().is_ok());
    }

    #[tokio::test]
    // 
    async fn should_not_override_existing_socket_without_force_flag() {
        let random_sock: String = format!("/tmp/{}.sock", 
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        let _cleaup = FileCleanup {
            file: random_sock.clone()
        };

        let server = PolydoroServerBuilder::default()
            .args(
                RunArgsBuilder::default()
                    .puid(random_sock.clone())
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        server.establish_socket().expect("should establish connection");
        server.establish_socket().expect_err("should not be able to establish connection (socket held)");
    }

    #[tokio::test]
    async fn should_override_existing_socket_with_force_flag() {
        let random_sock: String = format!("/tmp/{}.sock", 
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        let _cleaup = FileCleanup {
            file: random_sock.clone()
        };

        let server = PolydoroServerBuilder::default()
            .args(
                RunArgsBuilder::default()
                    .puid(random_sock.clone())
                    .force(true)
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        server.establish_socket().expect("should establish connection");
        server.establish_socket().expect("should override previous connection");
    }

    #[tokio::test]
    async fn should_use_xdg_data_home() {
        let random_sock: String = format!("{}.sock", 
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        set_var("XDG_DATA_HOME", "/tmp/");
    
        let _cleaup = FileCleanup {
            file: format!("/tmp/{}", random_sock.clone())
        };

        let server = PolydoroServerBuilder::default()
            .args(
                RunArgsBuilder::default()
                    .puid(random_sock.clone())
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        server.establish_socket().expect("should establish connection");
        try_exists(format!("/tmp/{}", random_sock)).await.expect("should have used XDG_DATA_DIR");
    }

    #[tokio::test]
    async fn should_use_absolute_path() {
        let random_sock: String = format!("/tmp/{}.sock", 
            Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        remove_var("XDG_DATA_HOME");

        let _cleaup = FileCleanup {
            file: random_sock.clone()
        };

        let server = PolydoroServerBuilder::default()
            .args(
                RunArgsBuilder::default()
                    .puid(random_sock.clone())
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        server.establish_socket().expect("should establish connection");
        try_exists(random_sock).await.expect("should have used XDG_DATA_DIR");
    }
}

