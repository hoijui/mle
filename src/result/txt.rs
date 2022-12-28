// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;
use std::sync::Mutex;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::BoxError;

use super::Writer;

pub struct Sink {
    flush: bool,
    links_stream: Option<Mutex<Box<dyn Write + 'static>>>,
    anchors_stream: Option<Mutex<Box<dyn Write + 'static>>>,
    errors_stream: Option<Mutex<Box<dyn Write + 'static>>>,
}

impl super::Sink for Sink {
    fn init(
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
    ) -> std::io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            flush: config.result_flush,
            links_stream: links_stream.map(Mutex::new),
            anchors_stream: anchors_stream.map(Mutex::new),
            errors_stream: Some(Mutex::new(Box::new(std::io::stderr()) as Box<dyn Write>)),
        }) as Box<dyn super::Sink>)
    }

    fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        if let Some(ref links_writer_m) = &self.links_stream {
            let mut links_writer = links_writer_m.lock().expect("we do not use MT");
            writeln!(links_writer, "{link}")?;
            if self.flush {
                links_writer.flush()?;
            }
        }
        Ok(())
    }

    fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        if let Some(ref anchors_writer_m) = self.anchors_stream {
            let mut anchors_writer = anchors_writer_m.lock().expect("we do not use MT");
            writeln!(anchors_writer, "{anchor}")?;
            if self.flush {
                anchors_writer.flush()?;
            }
        }
        Ok(())
    }

    fn sink_error(&mut self, error: &BoxError) -> std::io::Result<()> {
        if let Some(ref errors_writer_m) = self.errors_stream {
            let mut errors_writer = errors_writer_m.lock().expect("we do not use MT");
            writeln!(errors_writer, "{error:#?}")?;
            if self.flush {
                errors_writer.flush()?;
            }
        }
        Ok(())
    }

    fn finalize(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
