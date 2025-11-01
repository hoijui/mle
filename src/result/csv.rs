// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Mutex;

use csv;

use crate::config::Config;
use crate::result::Type;
use crate::{anchor::Anchor, link::Link};

use super::{AnchorRec, LinkRec, Writer, WriterOpt};

pub struct Sink {
    extended: bool,
    flush: bool,
    links_writer: Option<Mutex<csv::Writer<Writer>>>,
    anchors_writer: Option<Mutex<csv::Writer<Writer>>>,
}

impl Sink {
    fn delimiter(format: Type) -> u8 {
        match format {
            Type::Csv => b';',
            Type::Tsv => b'\t',
            _ => panic!("Result format {format:?} is not supported by the CSV sink."),
        }
    }

    fn writer(format: Type, stream_opt: WriterOpt) -> Option<Mutex<csv::Writer<Writer>>> {
        stream_opt
            .map(|stream| {
                csv::WriterBuilder::new()
                    .delimiter(Self::delimiter(format))
                    // .has_headers(true)
                    // .quote_style(csv::QuoteStyle::Necessary)
                    // .quote(b'"')
                    // .double_quote(true)
                    .from_writer(stream)
            })
            .map(Mutex::new)
    }
}

impl super::Sink for Sink {
    fn init(
        format: Type,
        config: &Config,
        links_stream: WriterOpt,
        anchors_stream: WriterOpt,
    ) -> std::io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            extended: config.result_extended,
            flush: config.result_flush,
            links_writer: Self::writer(format, links_stream),
            anchors_writer: Self::writer(format, anchors_stream),
        }) as Box<dyn super::Sink>)
    }

    fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        if let Some(links_writer_m) = &self.links_writer {
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
