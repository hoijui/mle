// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::group::Grouping;
use crate::BoxError;

use super::Writer;

pub struct Sink();

impl super::Sink for Sink {
    fn write_results(
        &self,
        _config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
        links: &Grouping,
        anchors: &[Anchor],
        errors: &[BoxError],
    ) -> std::io::Result<()> {
        if let Some(mut links_writer) = links_stream {
            writeln!(links_writer, "Links ...")?;
            for group in links {
                writeln!(links_writer, "  Group ...")?;
                for link in &group.1 {
                    writeln!(links_writer, "    {}", link)?;
                }
            }
        }

        if let Some(mut anchors_writer) = anchors_stream {
            if !anchors.is_empty() {
                writeln!(anchors_writer, "\nAnchors ...")?;
                for anchor in anchors {
                    writeln!(anchors_writer, "{}", anchor)?;
                }
            }
        }

        let mut stderr = Box::new(std::io::stderr()) as Box<dyn Write>;
        if !errors.is_empty() {
            writeln!(stderr, "\nErrors ...")?;
            for error in errors {
                writeln!(stderr, "{:#?}", error)?;
            }
        }

        Ok(())
    }
}
