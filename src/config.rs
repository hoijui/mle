// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use wildmatch::WildMatch;

use crate::{group, ignore_path::IgnorePath, logger::LogLevel, markup::Type, result};

#[derive(Debug, Clone)]
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
    /// Both 'None` and `Some("-")` mean: StdOut;
    /// everything else will be interpreted as a file path.
    pub result_file: Option<&'static str>,
    pub result_format: result::Type,
    /// How to group links together. Default: no grouping -
    /// links appear in the output in the order they were found.
    pub group_by: Option<group::Type>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: Default::default(),
            log_file: Default::default(),
            scan_root: Default::default(),
            recursive: Default::default(),
            links: true,
            anchors: false,
            ignore_paths: Default::default(),
            ignore_links: Default::default(),
            markup_types: Default::default(),
            resolve_root: Default::default(),
            result_file: Default::default(),
            result_format: Default::default(),
            group_by: Default::default(),
        }
    }
}
