// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::group::Grouping;

pub struct Sink();

impl super::Sink for Sink {
    fn write_results(
        &self,
        _config: &Config,
        out_stream: &mut Box<dyn Write + 'static>,
        links: &Grouping,
        anchors: &[Anchor],
        errors: &[Box<dyn std::error::Error>],
    ) -> std::io::Result<()> {
        writeln!(out_stream, "Links ...")?;
        for group in links {
            writeln!(out_stream, "  Group ...")?;
            for link in &group.1 {
                writeln!(out_stream, "    {}", link)?;
            }
        }

        if !anchors.is_empty() {
            writeln!(out_stream, "\nAnchors ...")?;
            for anchor in anchors {
                writeln!(out_stream, "{}", anchor)?;
            }
        }

        if !errors.is_empty() {
            writeln!(out_stream, "\nErrors ...")?;
            for error in errors {
                writeln!(out_stream, "{:#?}", error)?;
            }
        }

        Ok(())
    }
}
