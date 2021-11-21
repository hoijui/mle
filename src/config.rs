// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use wildmatch::WildMatch;

use crate::{ignore_path::IgnorePath, logger::LogLevel, markup::Type};

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub log_level: LogLevel,
    pub log_file: Option<PathBuf>,
    pub scan_root: PathBuf,
    pub recursive: bool,
    pub links: bool,
    pub anchors: bool,
    // pub match_file_extension: bool,
    pub ignore_paths: Vec<IgnorePath>,
    pub ignore_links: Vec<WildMatch>,
    pub markup_types: Vec<Type>,
    pub resolve_root: Option<PathBuf>,
    // pub dry: bool,
    pub result_file: Option<PathBuf>,
    pub result_format: String, // TODO Make this an enum
}
