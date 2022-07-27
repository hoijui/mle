#[macro_use]
extern crate log;

use mle::cli;
use mle::logger;
use mle::state::State;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = cli::parse_args()?;
    let mut state = State::new(config);
    logger::init(&state.config.log_level);
    info!("Config: {:?}", &state.config);
    if mle::run(&mut state).is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
