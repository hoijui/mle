// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use cli_utils::{StreamIdent, path_buf::PathBuf};
use serde::{Deserialize, Serialize};
use wildmatch::WildMatch;

use crate::result;

/// The core config for the link extractor.
///
/// This part is used by the library/programmatic invocation
/// of the extractor.
/// See [`Tool`] for the extended configuration,
/// used by the CLI invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extractor {
    /// Markup files to extract links and/or anchors from.
    ///
    /// Use other commands to construct this list,
    /// like `ls` or `git ls-files`.
    pub markup_files: Vec<PathBuf>,
    // pub recursive: bool,
    /// Where to store links to.
    ///
    /// - `None` => do not extract links,
    /// - `Some(StreamIdent::StdOut)`
    ///   => extract links and write them to stdout,
    /// - `Some(StreamIdent::Path("/path/to/extracted_links_file.txt".into(), false))`
    ///   => extract links and write them to file `path`.
    pub links: bool,
    /// Where to store anchors to.
    ///
    /// - `None` => do not extract anchors,
    /// - `Some(StreamIdent::StdOut)`
    ///   => extract anchors and write them to stdout,
    /// - `Some(StreamIdent::Path("/path/to/extracted_anchors_file.txt".into(), false))`
    ///   => extract anchors and write them to file `path`.
    pub anchors: bool,
    pub ignore_links: Vec<WildMatch>,
    // /// Where absolute links are resolve to..
    // ///
    // /// If `None`, the file-system root is used.
    // pub resolve_root: Option<PathBuf>,
}

/// The extended config for the link extractor.
///
/// This part is used by the CLI invocation
/// of the extractor.
/// See [`Extractor`] for the library/programmatic invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The core part of the configuration.
    pub extractor: Extractor,
    /// Where to store links to.
    ///
    /// - `None` => do not extract links,
    /// - `Some(StreamIdent::StdOut)`
    ///   => extract links and write them to stdout,
    /// - `Some(StreamIdent::Path("/path/to/extracted_links_file.txt".into(), false))`
    ///   => extract links and write them to file `path`.
    pub links: Option<StreamIdent>,
    /// Where to store anchors to.
    ///
    /// - `None` => do not extract anchors,
    /// - `Some(StreamIdent::StdOut)`
    ///   => extract anchors and write them to stdout,
    /// - `Some(StreamIdent::Path("/path/to/extracted_anchors_file.txt".into(), false))`
    ///   => extract anchors and write them to file `path`.
    pub anchors: Option<StreamIdent>,
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

impl Default for Extractor {
    fn default() -> Self {
        Self {
            markup_files: Vec::default(),
            links: true,
            anchors: false,
            ignore_links: Vec::default(),
        }
    }
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            extractor: Extractor::default(),
            links: Some(StreamIdent::StdOut),
            anchors: None,
            result_format: result::Type::default(),
            result_extended: false,
            result_flush: false,
        }
    }
}
