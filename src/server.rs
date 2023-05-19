use std::fmt;

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
    Rest,
    Work,
    Break
}

impl fmt::Display for PeriodType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct PolydoroServer {
    args: RunArgs,
    cycles: i8,
    clock: PausableClock,
    current_period: PeriodType,
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

    pub fn build_socket_path(puid: &str) -> String {
        format!("/tmp/{}", puid)
    }

    pub async fn run(self) -> Result<()> {
        let local_socket = LocalSocketListener::bind(
            PolydoroServer::build_socket_path(&self.args.puid)
        )?;

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
        let socket_name = PolydoroServer::build_socket_path(&this.read().await.args.puid);


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
            PeriodType::Break
        } else {
            PeriodType::Rest
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
            PeriodType::Rest => self.args.rest_period_s,
            PeriodType::Work => self.args.work_period_s,
            PeriodType::Break => self.args.break_period_s,
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

