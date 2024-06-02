// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod html;
mod markdown;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::markup::{self, File};

pub struct ParseRes {
    pub links: Vec<Link>,
    pub anchors: Vec<Anchor>,
}

impl ParseRes {
    #[must_use]
    pub fn invalid_links(&self) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|&link| link.is_invalid())
            .collect()
    }
}

pub fn remove_anchor(link: &mut String) -> Option<String> {
    link.find('#').map(|anchor_pos| {
        // let anchor = link.rsplit(pat: P)(suffix: P)(new_len: usize)
        let anchor: String = link.drain(anchor_pos..).skip(1).collect();
        // link.truncate(anchor_pos);
        anchor
    })
}

/// Finds links (and optionally anchors),
/// using the markup file specific link extractor internally.
///
/// # Errors
///
/// If fetching the markup file content failed.
pub fn find_links(file: &File, conf: &Config) -> std::io::Result<ParseRes> {
    let link_extractor = link_extractor_factory(file.markup_type);

    log::debug!(
        "Scannig file at location '{:#?}' for links ...",
        file.locator
    );
    link_extractor.find_links_and_anchors(file, conf)
    // match file.content.fetch() {
    //     Ok(text) => {
    //         // let (mut links, anchor_targets) =
    //             link_extractor.find_links_and_anchors(&file, anchors_only)//;
    //         // for l in &mut links {
    //         //     l.source = file.locator.to_string();
    //         //     l.anchor = remove_anchor(&mut l.target);
    //         //     //println!("XXX {:?}", l);
    //         // }
    //         // (links, anchor_targets)
    //     }
    //     Err(e) => {
    //         warn!(
    //             "File '{:#?}'. IO Error: '{}'. Check your file encoding.",
    //             file.locator, e
    //         );
    //         // (vec![], vec![])
    //         Err(...)
    //     }
    // }
}

fn link_extractor_factory(markup_type: markup::Type) -> Box<dyn LinkExtractor> {
    match markup_type {
        markup::Type::Markdown => Box::new(markdown::LinkExtractor()),
        markup::Type::Html => Box::new(html::LinkExtractor()),
    }
}

pub trait LinkExtractor {
    /// Finds links (and optionally anchors),
    /// using the markup file specific link extractor internally.
    ///
    /// # Errors
    /// If fetching the markup file content failed.
    fn find_links_and_anchors(&self, file: &File, conf: &Config) -> std::io::Result<ParseRes>;
}
