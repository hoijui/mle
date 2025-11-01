// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io::Write, sync::Mutex};

use crate::config::Config;
use crate::link::Link;
use crate::{anchor::Anchor, result::Type};

use super::{AnchorOwnedRec, LinkOwnedRec, Writer, WriterOpt};

pub struct Sink {
    extended: bool,
    links_stream: Option<Mutex<Writer>>,
    anchors_stream: Option<Mutex<Writer>>,
    links: Vec<LinkOwnedRec>,
    anchors: Vec<AnchorOwnedRec>,
}

impl super::Sink for Sink {
    fn init(
        _format: Type,
        config: &Config,
        links_stream: WriterOpt,
        anchors_stream: WriterOpt,
    ) -> std::io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            extended: config.result_extended,
            links_stream: links_stream.map(Mutex::new),
            anchors_stream: anchors_stream.map(Mutex::new),
            links: vec![],
            anchors: vec![],
        }) as Box<dyn super::Sink>)
    }

    fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        self.links.push(LinkOwnedRec::new(link, self.extended));
        Ok(())
    }

    fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        self.anchors
            .push(AnchorOwnedRec::new(anchor, self.extended));
        Ok(())
    }

    // There are two false positives reported by clippy::significant_drop_tightening here
    #[allow(clippy::significant_drop_tightening)]
    fn finalize(&mut self) -> std::io::Result<()> {
        if let Some(links_writer_m) = &self.links_stream {
            let mut links_writer = links_writer_m.lock().expect("we do not use MT");
            let json = serde_json::to_string_pretty(&self.links)?;
            write!(links_writer, "{json}")?;
        }
        if let Some(anchors_writer_m) = &self.anchors_stream {
            let mut anchors_writer = anchors_writer_m.lock().expect("we do not use MT");
            let json = serde_json::to_string_pretty(&self.anchors)?;
            write!(anchors_writer, "{json}")?;
        }

        Ok(())
    }
}
