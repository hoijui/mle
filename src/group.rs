// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::link::Target;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
pub use wildmatch::WildMatch;

const T_TR_AS_IS: &str = "none";
const T_TR_IGNORE_ANCHOR: &str = "ignore-anchor";

type Grouping<'a> = Vec<(Cow<'a, Target>, Vec<&'a Link>)>;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    AsIs,
    IgnoreAnchor,
}

const fn group_as_is(link: &Link) -> Cow<'_, Target> {
    Cow::Borrowed(&link.target)
}

fn group_without_anchor(link: &Link) -> Cow<'_, Target> {
    link.target.remove_anchor()
}

impl FromStr for Type {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            T_TR_AS_IS => Ok(Self::AsIs),
            T_TR_IGNORE_ANCHOR => Ok(Self::IgnoreAnchor),
            _ => Err("Unknown group type"),
        }
    }
}

impl Type {
    /// Returns a function that extracts the group from a [`Link`],
    /// as specified in the configuration.
    #[must_use]
    pub const fn get(config: &Config) -> Self {
        match config.group_by {
            None => Self::AsIs,
            Some(r#type) => r#type,
        }
    }

    /// Returns a function that extracts the group from a [`Link`],
    /// as specified in the configuration.
    #[must_use]
    pub const fn get_grouper(self) -> fn(&Link) -> Cow<'_, Target> {
        match self {
            Self::AsIs => group_as_is,
            Self::IgnoreAnchor => group_without_anchor,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AsIs => T_TR_AS_IS,
            Self::IgnoreAnchor => T_TR_IGNORE_ANCHOR,
        }
    }
}

/// Runs the markup link extractor.
/// This is the main entry point of this library.
///
/// # Errors
///
/// If reading of any input or writing of the log or result-file failed.
pub fn group<'a>(
    links: &'a [Link],
    _anchors: &[Anchor],
    _errors: &[Box<dyn std::error::Error>],
    _grouper: fn(&Link) -> Cow<'_, Target>,
) -> Result<Grouping<'a>, Box<dyn std::error::Error>> {
    let mut groups: HashMap<Cow<'_, Target>, Vec<&Link>> = HashMap::new();
    // let extract_group = group_as_is;
    let extract_group = group_without_anchor;
    for link in links {
        let group = extract_group(link);
        groups.entry(group).or_insert(vec![]).push(link);
    }
    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_by(|x, y| x.0.cmp(&y.0));
    Ok(sorted_groups)
}
