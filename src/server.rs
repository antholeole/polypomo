use std::{
    thread::{self, JoinHandle}, 
    time::{Duration, Instant}, 
    fmt::Display, 
    sync::{RwLock, Arc}, 
    io::Read
};
use pausable_clock::PausableClock;

use interprocess::local_socket::LocalSocketListener;

use crate::cli::RunArgs;
use crate::client::{OpCode, opcode_from_byte};

#[derive(PartialEq)]
enum PeriodType {
    Rest,
    Work,
    Break
}

pub struct PolydoroServer {
    args: RunArgs,
    cycles: u16,
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

    pub fn run(self) -> JoinHandle<()> {    
        let poll_time = Duration::from_millis(self.args.refresh_rate_ms);
        let local_socket = LocalSocketListener::bind(
            self.args.puid.clone()
        ).unwrap();
        
        let rw_self = Arc::new(RwLock::new(self));

        let listener_lock = rw_self.clone();
        let server_lock = rw_self.clone();

        let should_loop = Arc::new(RwLock::new(true));

        let listener_should_loop = should_loop.clone();
        let display_should_loop = should_loop.clone();


        ctrlc::set_handler(move || *should_loop.write().unwrap() = false
        ).expect("Error setting Ctrl-C handler");
        

        let _listener_thread = thread::spawn(move || {
            for incoming in local_socket.incoming() {
                if !*listener_should_loop.read().unwrap() {
                    break;
                }

                let mut buf: [u8; 1] = [99];

                // We only need one message, so we consume & discard the stream after reading.
                incoming.unwrap().read_exact(&mut buf).unwrap();

                match opcode_from_byte(buf[0]) {
                    OpCode::Skip => listener_lock.write().unwrap().change_state(),
                    OpCode::Toggle => listener_lock.write().unwrap().toggle_pause(),
                }
            }
        });

        let display_thread = thread::spawn(move || {
            loop {
                if !*display_should_loop.read().unwrap() {
                    break;
                }


                let start = Instant::now();


                let runtime = start.elapsed();
                if let Some(remaining) = poll_time.checked_sub(runtime) {
                    thread::sleep(remaining);
                }

                server_lock.write().unwrap().tick();

                println!("{}", rw_self.read().unwrap());
            }
        });

        // Both threads are expected to start / stop in tandem, so no need to return both
        // join handlers. SIGINT will stop both.
        return display_thread;
    }

    pub fn tick(&mut self) {
        if self.clock.is_paused() {
            return;
        }

        let cycle_time_elapsed = Instant::from(self.clock.now()).elapsed();

        if cycle_time_elapsed <= self.get_period_length().into() {
            return;
        }  

        // state change 
        self.change_state();
    }

    fn toggle_pause(&mut self) {
        if self.clock.is_paused() {
            self.clock.resume();
        } else {
            self.clock.pause();
        }
    }

    fn change_state(&mut self) {
        let period_type = if self.current_period != PeriodType::Work {
            self.cycles += 1;
            PeriodType::Work
        } else if self.cycles >= self.args.cycles {
            self.cycles = 0;
            PeriodType::Break
        } else {
            PeriodType::Rest
        };

        // paused = false feels wrong but gets the correct behavior.
        self.clock = PausableClock::new(Duration::ZERO, false);
        self.current_period = period_type;
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

        let seconds = (period_length - time_elapsed).as_secs() + 1;

        write!(f, "{}{} ({})", 
            symbol, 
            format!("{}:{:02}", seconds / 60, seconds % 60),
            self.cycles + 1
        )
    }
}

