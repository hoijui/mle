// SPDX-FileCopyrightText: 2022 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::path_buf::PathBuf;

use serde::{Deserialize, Serialize};
use wildmatch::WildMatch;

use crate::result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Markup files to extract links and/or anchors from.
    ///
    /// Use other commands to construct this list,
    /// like `ls` or `git ls-files`.
    pub markup_files: Vec<PathBuf>,
    // pub recursive: bool,
    /// Where to store links to.
    ///
    /// - `None` => do not extract links,
    /// - `Some(None)` => extract links and write them to stdout,
    /// - `Some(Some(path))` => extract links and write them to file `path`.
    pub links: Option<Option<PathBuf>>,
    /// Where to store anchors to.
    ///
    /// - `None` => do not extract anchors,
    /// - `Some(None)` => extract anchors and write them to stdout,
    /// - `Some(Some(path))` => extract anchors and write them to file `path`.
    pub anchors: Option<Option<PathBuf>>,
    pub ignore_links: Vec<WildMatch>,
    pub result_format: result::Type,
    /// Whether to include non-essential information in the resulting report.
    /// Non-essential are things like:
    ///
    /// - is the link local
    /// - is the link a URL or to the file-system
    /// - which type of anchor it is (e.g. from a title or anchor tag)
    pub result_extended: bool,
    /// Whether to flush output streams after each item (link, anchor, error),
    /// for the result formats that support it.
    pub result_flush: bool,
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
            markup_files: Vec::default(),
            links: Some(None),
            anchors: None,
            ignore_links: Vec::default(),
            result_format: result::Type::default(),
            result_extended: false,
            result_flush: false,
        }
    }
}
