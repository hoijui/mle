// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use wildmatch::WildMatch;

use crate::{group, ignore_path::IgnorePath, logger::LogLevel, markup, result};

#[derive(Debug, Clone)]
pub struct Config {
    pub log_level: LogLevel,
    pub log_file: Option<PathBuf>,
    pub scan_root: PathBuf,
    pub recursive: bool,
    /// Where to store links to.
    /// None => do not extract links,
    /// Some(None) => extract links and write them to stdout,
    /// Some(Some(path)) => extract links and write them to file `path`.
    pub links: Option<Option<PathBuf>>,
    /// Where to store anchors to.
    /// None => do not extract anchors,
    /// Some(None) => extract anchors and write them to stdout,
    /// Some(Some(path)) => extract anchors and write them to file `path`.
    pub anchors: Option<Option<PathBuf>>,
    // pub match_file_extension: bool,
    pub ignore_paths: Vec<IgnorePath>,
    pub ignore_links: Vec<WildMatch>,
    pub markup_types: Vec<markup::Type>,
    pub resolve_root: Option<PathBuf>,
    // pub dry: bool,
    pub result_format: result::Type,
    /// How to group links together. Default: no grouping -
    /// links appear in the output in the order they were found.
    pub group_by: Option<group::Type>,
}

impl Config {
    #[must_use]
    pub const fn extract_links(&self) -> bool {
        self.links.is_some()
    }

    #[must_use]
    pub const fn extract_anchors(&self) -> bool {
        self.anchors.is_some()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: Default::default(),
            log_file: Default::default(),
            scan_root: Default::default(),
            recursive: Default::default(),
            links: Some(None),
            anchors: None,
            ignore_paths: Default::default(),
            ignore_links: Default::default(),
            markup_types: Default::default(),
            resolve_root: Default::default(),
            result_format: Default::default(),
            group_by: Default::default(),
        }
    }
}
