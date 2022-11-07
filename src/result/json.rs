// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io::Write, sync::Mutex};

use serde::Serialize;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::BoxError;

use super::{AnchorOwnedRec, LinkOwnedRec, Writer};

pub struct Sink {
    extended: bool,
    links_stream: Option<Mutex<Box<dyn Write + 'static>>>,
    anchors_stream: Option<Mutex<Box<dyn Write + 'static>>>,
    errors_stream: Option<Mutex<Box<dyn Write + 'static>>>,
    links: Vec<LinkOwnedRec>,
    anchors: Vec<AnchorOwnedRec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootSerLinks<'a> {
    pub links: &'a Vec<Link>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootSerAnchors<'a> {
    pub anchors: &'a Vec<Anchor>,
}

impl super::Sink for Sink {
    fn init(
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
    ) -> std::io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            extended: config.result_extended,
            links_stream: links_stream.map(Mutex::new),
            anchors_stream: anchors_stream.map(Mutex::new),
            errors_stream: Some(Mutex::new(Box::new(std::io::stderr()) as Box<dyn Write>)),
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

    fn sink_error(&mut self, error: &BoxError) -> std::io::Result<()> {
        if let Some(ref errors_writer_m) = self.errors_stream {
            let mut errors_writer = errors_writer_m.lock().expect("we do not use MT");
            writeln!(errors_writer, "{:#?}", error)?;
        }
        Ok(())
    }

    fn finalize(&mut self) -> std::io::Result<()> {
        if let Some(ref links_writer_m) = &self.links_stream {
            let mut links_writer = links_writer_m.lock().expect("we do not use MT");
            let json = serde_json::to_string_pretty(&self.links)?;
            write!(links_writer, "{}", json)?;
        }
        if let Some(ref anchors_writer_m) = &self.anchors_stream {
            let mut anchors_writer = anchors_writer_m.lock().expect("we do not use MT");
            let json = serde_json::to_string_pretty(&self.anchors)?;
            write!(anchors_writer, "{}", json)?;
        }

        Ok(())
    }
}
