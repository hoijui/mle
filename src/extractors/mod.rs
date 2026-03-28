// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod html;
mod markdown;

use crate::anchor::Anchor;
use crate::config::Extractor as Config;
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
pub async fn scan_for_links<LR: AsyncFnMut(Link), AR: AsyncFnMut(Anchor)>(
    file: &File<'_>,
    conf: &Config,
    links_receiver: &mut LR,
    anchors_receiver: &mut AR,
) -> std::io::Result<()> {
    let link_extractor = link_extractor_factory(file.markup_type);

    log::debug!(
        "Scanning file at location '{:#?}' for links ...",
        file.locator
    );
    link_extractor
        .find_links_and_anchors(file, conf, links_receiver, anchors_receiver)
        .await
}

/// Finds links (and optionally anchors),
/// using the markup file specific link extractor internally.
///
/// # Errors
///
/// If fetching the markup file content failed.
pub async fn gather_links(file: &File<'_>, conf: &Config) -> std::io::Result<ParseRes> {
    let mut links = vec![];
    let mut anchors = vec![];
    let links_receiver = &mut async |link: Link| {
        for link_ignorer in &conf.ignore_links {
            let link_as_str = link.target.to_string();
            if link_ignorer.matches(&link_as_str) {
                return;
            }
        }
        links.push(link);
    };
    let anchors_receiver = &mut async |anchor: Anchor| {
        anchors.push(anchor);
    };
    scan_for_links(file, conf, links_receiver, anchors_receiver).await?;
    Ok(ParseRes { links, anchors })
}

enum LinkExtractorCont {
    Markdown(markdown::LinkExtractor),
    Html(html::LinkExtractor),
}

impl LinkExtractor for LinkExtractorCont {
    async fn find_links_and_anchors<LR: AsyncFnMut(Link), AR: AsyncFnMut(Anchor)>(
        &self,
        file: &File<'_>,
        conf: &Config,
        links_receiver: &mut LR,
        anchors_receiver: &mut AR,
    ) -> std::io::Result<()> {
        match self {
            Self::Markdown(internal) => {
                internal
                    .find_links_and_anchors(file, conf, links_receiver, anchors_receiver)
                    .await
            }
            Self::Html(internal) => {
                internal
                    .find_links_and_anchors(file, conf, links_receiver, anchors_receiver)
                    .await
            }
        }
    }
}

const fn link_extractor_factory(markup_type: markup::Type) -> LinkExtractorCont {
    match markup_type {
        markup::Type::Markdown => LinkExtractorCont::Markdown(markdown::LinkExtractor()),
        markup::Type::Html => LinkExtractorCont::Html(html::LinkExtractor()),
    }
}

pub(crate) trait LinkExtractor {
    /// Finds links (and optionally anchors),
    /// using the markup file specific link extractor internally.
    ///
    /// # Errors
    /// If fetching the markup file content failed.
    async fn find_links_and_anchors<LR: AsyncFnMut(Link), AR: AsyncFnMut(Anchor)>(
        &self,
        file: &File<'_>,
        conf: &Config,
        links_receiver: &mut LR,
        anchors_receiver: &mut AR,
    ) -> std::io::Result<()>;
}
