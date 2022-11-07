// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use serde::Serialize;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::BoxError;

use super::Writer;

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
        _config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
        links: &[Link],
        anchors: &[Anchor],
        errors: &[BoxError],
    ) -> std::io::Result<()> {
        if let Some(mut links_writer) = links_stream {
            let content = RootSerLinks {
                links: &links.into(),
            };
            let json = serde_json::to_string_pretty(&content)?;
            write!(links_writer, "{}", json)?;
        }
        if let Some(mut anchors_writer) = anchors_stream {
            let content = RootSerAnchors {
                anchors: &anchors.into(),
            };
            let json = serde_json::to_string_pretty(&content)?;
            write!(anchors_writer, "{}", json)?;
        }

        let str_errors = errors.iter().map(ToString::to_string).collect::<String>();

        Ok(())
    }
}
