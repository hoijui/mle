#[macro_use]
extern crate log;

use mle::State;
use mle::cli;
use mle::logger;
use std::process;

#[macro_use]
extern crate clap;

fn print_header() {
    let width = 60;
    let header = format!("markup link extractor - mle v{:}", crate_version!());
    println!();
    println!("{:+<1$}", "", width);
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    print!("+");
    print!("{}", format!("{: ^1$}", header, width - 2));
    println!("+");
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    println!("{:+<1$}", "", width);
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    let config = cli::parse_args();
    let mut state = State::new(config);
    logger::init(&state.config.log_level);
    info!("Config: {:?}", &state.config);
    if mle::run(&mut state).await.is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
