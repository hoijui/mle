// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[macro_use]
extern crate log;

use mle::cli;
use mle::state::State;
use mle::BoxResult;

#[tokio::main]
async fn main() -> BoxResult<()> {
    env_logger::init();
    let config = cli::parse_args()?;
    let mut state = State::new(config);
    info!("Config: {:?}", &state.config);
    mle::run(&mut state)
}
