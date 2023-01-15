// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[macro_use]
extern crate log;

use std::path::PathBuf;

use mle::cli;
use mle::logger;
use mle::state::State;
use mle::BoxResult;

#[tokio::main]
async fn main() -> BoxResult<()> {
    let log_rh_filter = logger::setup::<PathBuf>(&None, &None)?;
    let config = cli::parse_args()?;
    let mut state = State::new(config);
    logger::set_level(&log_rh_filter, state.config.log_level_wrap())?;
    logger::setup(&state.config.log_level_wrap(), &state.config.log_file)?;
    info!("Config: {:?}", &state.config);
    mle::run(&mut state)
}
