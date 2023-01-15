// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;
use std::{fs::File, path::Path};

use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt,
    prelude::*,
    reload::{self, Handle},
    Registry,
};

use crate::BoxResult;

const fn level_to_filter(level: log::Level) -> LevelFilter {
    match level {
        log::Level::Error => LevelFilter::ERROR,
        log::Level::Warn => LevelFilter::WARN,
        log::Level::Info => LevelFilter::INFO,
        log::Level::Debug => LevelFilter::DEBUG,
        log::Level::Trace => LevelFilter::TRACE,
    }
}

const fn level_opt_to_filter(level_opt: Option<log::Level>) -> LevelFilter {
    if let Some(level) = level_opt {
        level_to_filter(level)
    } else {
        LevelFilter::OFF
    }
}

const fn default_level() -> log::Level {
    if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    }
}

/// Sets up logging, with a way to change the log level later on,
/// and with all output going to stderr,
/// as suggested by <https://clig.dev/>.
///
/// # Errors
///
/// If initializing the registry (logger) failed.
pub fn setup<P: AsRef<Path>>(
    level_opt: &Option<log::Level>,
    file_opt: &Option<P>,
// ) -> BoxResult<(Handle<LevelFilter, Registry>, Handle<reload::Layer<LevelFilter, Registry>, Registry>)> {
) -> BoxResult<Handle<LevelFilter, Registry>> {
    let level = level_opt.unwrap_or_else(default_level);
    let level_filter = level_to_filter(level);
    let (lr_filter, rh_filter) = reload::Layer::new(level_filter);

    let stderr = io::stderr.with_max_level();
    let l_stderr = fmt::layer().map_writer(move |w| io::stderr.or_else(w));

    let l_file = fmt::layer().map_writer(move |_| io::sink);
    let (lr_file, rh_file) = reload::Layer::new(l_file);
    // let (lr_file, rh_file) = reload::Layer::new:<Sink>(io::sink);

    let registry = tracing_subscriber::registry().with(lr_filter).with(l_file).with(l_stderr);
    if let Some(file) = file_opt {
        let file_strm = File::open(file.as_ref())?;
        let l_file = fmt::layer().map_writer(move |_| file_strm);
        registry.with(l_file).try_init()?;
    } else {
        registry.try_init()?;
    }

    Ok(rh_filter)
    // Ok((rh_filter, rh_file))
}

/// Changes the minimum log level.
///
/// # Errors
///
/// If modifying the log level filter failes.
pub fn set_level(
    reload_handle: &Handle<LevelFilter, Registry>,
    level: Option<log::Level>,
) -> BoxResult<()> {
    let level_filter = level_opt_to_filter(level);
    reload_handle.modify(|filter| *filter = level_filter)?;
    Ok(())
}

pub fn set_file(
    reload_handle: &Handle<LevelFilter, Registry>,
    file: &Path,
) -> BoxResult<()> {
    // let level_filter = verbosity_to_level(verbosity);
    // reload_handle.modify(|filter| *filter = level_filter)?;
    todo!();

    // let level_filter = level_opt_to_filter(level);
    // reload_handle.modify(|filter| *filter = level_filter)?;

    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // subscriber.wi
    // tracing::subscriber::s
    Ok(())
}
