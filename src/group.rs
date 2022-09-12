// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::link::Target;
use crate::BoxResult;
use clap::{PossibleValue, ValueEnum};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
pub use wildmatch::WildMatch;

const T_TR_AS_IS: &str = "none";
const T_TR_IGNORE_ANCHOR: &str = "ignore-anchor";

pub type Id<'a> = Cow<'a, Target>;
pub type Items<'a> = Vec<&'a Link>;

pub type Grouping<'a> = Vec<(Cow<'a, Target>, Vec<&'a Link>)>;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    AsIs,
    IgnoreAnchor,
}

impl ValueEnum for Type {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::AsIs, Self::IgnoreAnchor]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        Some(self.as_str().into())
    }
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
    pub const fn get_grouper(self) -> Option<fn(&Link) -> Cow<'_, Target>> {
        match self {
            Self::AsIs => None,
            Self::IgnoreAnchor => Some(group_without_anchor),
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
pub fn group(
    links: &'_ [Link],
    grouper: Option<fn(&Link) -> Cow<'_, Target>>,
) -> BoxResult<Grouping<'_>> {
    if let Some(grouping_fn) = grouper {
        let mut groups: HashMap<Id<'_>, Items<'_>> = HashMap::new();
        for link in links {
            let group = grouping_fn(link);
            groups.entry(group).or_insert(vec![]).push(link);
        }
        let mut sorted_groups = groups.into_iter().collect::<Grouping>();
        sorted_groups.sort_by(|x, y| x.0.cmp(&y.0));
        Ok(sorted_groups)
    } else {
        Ok(vec![(
            Cow::Owned(Target::Invalid(String::from(""))),
            links.iter().collect(),
        )])
    }
}
