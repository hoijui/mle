// SPDX-FileCopyrightText: 2022 - 2023 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod anchor;
pub mod config;
pub mod extractors;
pub mod file_traversal;
pub mod ignore_link;
pub mod ignore_path;
pub mod link;
pub mod markup;
pub mod result;
pub mod state;

use crate::anchor::Anchor;
use crate::link::Link;
use crate::markup::File;
pub use colored::*;
pub use config::Config;
use git_version::git_version;
use state::State;
pub use wildmatch::WildMatch;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxResult<T> = Result<T, BoxError>;

pub const VERSION: &str = git_version!();

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
            Ok(mut parsed) => {
                links.append(&mut parsed.links);
                anchor_targets.append(&mut parsed.anchors);
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
