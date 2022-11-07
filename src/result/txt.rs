// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::BoxError;

use super::Writer;

pub struct Sink();

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
        if !links.is_empty() {
            if let Some(mut links_writer) = links_stream {
                for link in links {
                    writeln!(links_writer, "{}", link)?;
                }
            }
        }

        if !anchors.is_empty() {
            if let Some(mut anchors_writer) = anchors_stream {
                for anchor in anchors {
                    writeln!(anchors_writer, "{}", anchor)?;
                }
            }
        }

        if !errors.is_empty() {
            let mut stderr = Box::new(std::io::stderr()) as Box<dyn Write>;
            for error in errors {
                writeln!(stderr, "{:#?}", error)?;
            }
        }

        Ok(())
    }
}
