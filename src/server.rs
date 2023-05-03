use std::future::Future;

use tokio::time::error::Error;

use crate::cli::RunArgs;

enum PeriodType {
    Rest,
    Work
}

pub struct PolypomoServer {
    args: RunArgs,
    cycles: u16
} 


impl PolypomoServer {
    pub fn new(args: RunArgs) -> PolypomoServer {
        PolypomoServer {
            args,
            cycles: 0
        }
    }

    async fn run(&self) {

    } 
}