// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate simplelog;

use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum LogLevel {
    Info,
    Warn,
    Debug,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Warn
    }
}

/// Inits the logger with the given log-level
///
/// # Panics
/// If logger initilaization failed due to it already having been in progress
/// (from a previous call to initilaize the logger).
pub fn init(log_level: &LogLevel) {
    let level_filter = match log_level {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Debug => LevelFilter::Debug,
    };

    let err = CombinedLogger::init(vec![TermLogger::new(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);
    assert!(err.is_ok(), "Failed to init logger! Error: {:?}", err);
    debug!("Initialized logging");
}
