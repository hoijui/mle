// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    logger::init(&state.config.log_level, &state.config.log_file);
    info!("Config: {:?}", &state.config);
    if mle::run(&mut state).is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
