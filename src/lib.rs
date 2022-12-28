// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::default_trait_access)]
// #![warn(clippy::restriction)]

// #![warn(clippy::wildcard_enum_match_arm)]
// #![warn(clippy::string_slice)]
// #![warn(clippy::indexing_slicing)]
// #![warn(clippy::clone_on_ref_ptr)]
// #![warn(clippy::try_err)]
#![warn(clippy::shadow_reuse)]
// #![warn(clippy::single_char_lifetime_names)]
// #![warn(clippy::empty_structs_with_brackets)]
// #![warn(clippy::else_if_without_else)]
#![warn(clippy::use_debug)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_format;

pub mod anchor;
pub mod cli;
pub mod config;
pub mod extractors;
pub mod file_traversal;
pub mod ignore_link;
pub mod ignore_path;
pub mod link;
pub mod logger;
pub mod markup;
pub mod result;
pub mod state;

use crate::anchor::Anchor;
use crate::link::Link;
use crate::markup::File;
pub use colored::*;
use config::Config;
use state::State;
pub use wildmatch::WildMatch;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxResult<T> = Result<T, BoxError>;

fn find_all_links(conf: &Config) -> (Vec<Link>, Vec<Anchor>, Vec<BoxError>) {
    let mut files: Vec<File> = Vec::new();
    let mut links = vec![];
    let mut anchor_targets = vec![];
    let mut errors: Vec<_> = vec![];
    if let Err(err) = file_traversal::find(conf, &mut files) {
        errors.push(err.into());
        return (links, anchor_targets, errors);
    }
    for file in files {
        match extractors::find_links(&file, conf) {
            Ok((mut file_links, mut file_anchor_targets)) => {
                links.append(&mut file_links);
                anchor_targets.append(&mut file_anchor_targets);
            }
            Err(err) => {
                errors.push(err.into());
            }
        }
    }
    (links, anchor_targets, errors)
}

/// Runs the markup link extractor.
/// This is the main entry point of this library.
///
/// # Errors
///
/// If reading of any input or writing of the log or result-file failed.
pub fn run(state: &mut State) -> BoxResult<()> {
    let (links, anchors, errors) = find_all_links(&state.config);
    // TODO make this more stream-like, where each found link is directly sent to all output streams/files
    result::sink(&state.config, &links, &anchors, &errors).map_err(Into::into)
}
