// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate simplelog;

use std::{fs::File, path::PathBuf};

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum LogLevel {
    Info,
    Warn,
    Debug,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Warn
    }
}

/// Inits the logger with the given log-level.
///
/// # Panics
/// If logger initilaization failed due to it already having been in progress
/// (from a previous call to initialize the logger).
pub fn init(log_level: &LogLevel, log_file: &Option<PathBuf>) {
    let level_filter = match log_level {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Debug => LevelFilter::Debug,
    };
    let logger: Box<dyn SharedLogger> = match log_file {
        None => TermLogger::new(
            level_filter,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        Some(log_file_path) => WriteLogger::new(
            level_filter,
            Config::default(),
            File::create(log_file_path).expect("Failed to open log file for writing"),
        ),
    };
    CombinedLogger::init(vec![logger]).expect("Failed to init logger!");
    debug!("Initialized logging");
}
