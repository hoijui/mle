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
        let str_errors = errors.iter().map(ToString::to_string).collect::<String>();
        let content = (
            ("links", links),
            ("groups", groups),
            ("anchors", anchors),
            ("errors", str_errors),
        );
        let json = serde_json::to_string_pretty(&content)?;
        write!(out_stream, "{}", json)?;

        Ok(())
    }
}
