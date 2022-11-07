// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use serde::Serialize;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::BoxError;

use super::{AnchorRec, LinkRec, Writer};

pub struct Sink();

#[derive(Debug, Clone, Serialize)]
pub struct RootSerLinks<'a> {
    pub links: &'a Vec<Link>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootSerAnchors<'a> {
    pub anchors: &'a Vec<Anchor>,
}

impl super::Sink for Sink {
    fn write_results(
        &self,
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
        links: &[Link],
        anchors: &[Anchor],
        errors: &[BoxError],
    ) -> std::io::Result<()> {
        let extended = config.result_extended;

        if let Some(mut links_writer) = links_stream {
            let mut recs = vec![];
            for lnk in links {
                recs.push(LinkRec::new(lnk, extended));
            }
            let json = serde_json::to_string_pretty(&recs)?;
            write!(links_writer, "{}", json)?;
        }
        if let Some(mut anchors_writer) = anchors_stream {
            let mut recs = vec![];
            for anc in anchors {
                recs.push(AnchorRec::new(anc, extended));
            }
            let json = serde_json::to_string_pretty(&recs)?;
            write!(anchors_writer, "{}", json)?;
        }

        let str_errors = errors.iter().map(ToString::to_string).collect::<String>();

        Ok(())
    }
}
