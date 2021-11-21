// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;

use crate::{config::Config, link::MarkupAnchorTarget};

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
pub type AnchorTargets = Option<Vec<MarkupAnchorTarget>>;

/// If a URL is not stored in the map (the URL does not appear as a key),
/// it means that URL has not yet been checked.
/// If the Result is Err, it means the URL has been checked,
/// but was not available, or anchor parsing has failed.
/// If the Option is None, it means the URL was checked and evaluated as for available,
/// but no parsing of anchors was tried.
/// If the Vec is empty, it means that the document was parsed, but no anchors were found.
pub type RemoteCache = HashMap<reqwest::Url, reqwest::Result<AnchorTargets>>;

#[derive(Default, Debug)]
pub struct State {
    pub config: Config,
    pub remote_cache: RemoteCache,
}

impl State {
    #[must_use]
    pub fn new(config: Config) -> State {
        State {
            remote_cache: RemoteCache::new(),
            config,
        }
    }
}
