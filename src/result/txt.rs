// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;
use std::io::Write;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::{Link, Target};

pub struct Sink();

impl super::Sink for Sink {
    fn write_results(
        &self,
        _config: &Config,
        out_stream: &mut Box<dyn Write + 'static>,
        links: &[Link],
        groups: &[(Cow<'_, Target>, Vec<&Link>)],
        anchors: &[Anchor],
        errors: &[Box<dyn std::error::Error>],
    ) -> std::io::Result<()> {
        if !links.is_empty() {
            writeln!(out_stream, "Links ...")?;
            if groups.is_empty() {
                for link in links {
                    writeln!(out_stream, "{}", link)?;
                }
            } else {
                for group in groups {
                    writeln!(out_stream, "  Group ...")?;
                    for link in &group.1 {
                        writeln!(out_stream, "    {}", link)?;
                    }
                }
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
