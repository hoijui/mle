// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;
use std::sync::Mutex;

use csv;

use crate::config::Config;
use crate::BoxError;
use crate::{anchor::Anchor, link::Link};

use super::{AnchorRec, LinkRec, Writer};

pub struct Sink {
    extended: bool,
    flush: bool,
    links_writer: Option<Mutex<csv::Writer<Box<dyn Write + 'static>>>>,
    anchors_writer: Option<Mutex<csv::Writer<Box<dyn Write + 'static>>>>,
    errors_stream: Option<Mutex<Box<dyn Write + 'static>>>,
}

impl super::Sink for Sink {
    fn init(
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
    ) -> std::io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            extended: config.result_extended,
            flush: config.result_flush,
            links_writer: links_stream.map(csv::Writer::from_writer).map(Mutex::new),
            anchors_writer: anchors_stream.map(csv::Writer::from_writer).map(Mutex::new),
            errors_stream: Some(Mutex::new(Box::new(std::io::stderr()) as Box<dyn Write>)),
        }) as Box<dyn super::Sink>)
    }

    fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        if let Some(ref links_writer_m) = &self.links_writer {
            let mut links_writer = links_writer_m.lock().expect("we do not use MT");
            let rec = LinkRec::new(link, self.extended);
            links_writer.serialize(rec)?;
            if self.flush {
                links_writer.flush()?;
            }
        }
        Ok(())
    }

    fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        if let Some(ref anchors_writer_m) = self.anchors_writer {
            let mut anchors_writer = anchors_writer_m.lock().expect("we do not use MT");
            let rec = AnchorRec::new(anchor, self.extended);
            anchors_writer.serialize(rec)?;
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
        if let Some(links_writer_m) = &self.links_writer {
            let mut links_writer = links_writer_m.lock().expect("we do not use MT");
            links_writer.flush()?;
        }

        if let Some(anchors_writer_m) = &self.anchors_writer {
            let mut anchors_writer = anchors_writer_m.lock().expect("we do not use MT");
            anchors_writer.flush()?;
        }

        Ok(())
    }
}
