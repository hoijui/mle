// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_trait::async_trait;
use csv_async as csv;
use tokio::sync::Mutex;

use crate::config::Tool as Config;
use crate::result::Type;
use crate::{anchor::Anchor, link::Link};

use super::{AnchorRec, LinkRec, Writer, WriterOpt};

pub struct Sink {
    extended: bool,
    flush: bool,
    links_writer: Option<Mutex<csv::AsyncSerializer<Writer>>>,
    anchors_writer: Option<Mutex<csv::AsyncSerializer<Writer>>>,
}

impl Sink {
    fn delimiter(format: Type) -> u8 {
        match format {
            Type::Csv => b';',
            Type::Tsv => b'\t',
            _ => panic!("Result format {format:?} is not supported by the CSV sink."),
        }
    }

    fn writer(format: Type, stream_opt: WriterOpt) -> Option<Mutex<csv::AsyncSerializer<Writer>>> {
        stream_opt
            .map(|stream| {
                csv::AsyncWriterBuilder::new()
                    .delimiter(Self::delimiter(format))
                    // .has_headers(true)
                    // .quote_style(csv::QuoteStyle::Necessary)
                    // .quote(b'"')
                    // .double_quote(true)
                    .create_serializer(stream)
            })
            .map(Mutex::new)
    }
}

#[async_trait]
impl super::Sink for Sink {
    async fn init(
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

    #[allow(clippy::significant_drop_tightening)]
    async fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        if let Some(links_writer_m) = &self.links_writer {
            let mut links_writer = links_writer_m.lock().await;
            let rec = LinkRec::new(link, self.extended);
            links_writer.serialize(rec).await?;
            if self.flush {
                links_writer.flush().await?;
            }
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        if let Some(ref anchors_writer_m) = self.anchors_writer {
            let mut anchors_writer = anchors_writer_m.lock().await;
            let rec = AnchorRec::new(anchor, self.extended);
            anchors_writer.serialize(rec).await?;
            if self.flush {
                anchors_writer.flush().await?;
            }
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn finalize(&mut self) -> std::io::Result<()> {
        if let Some(links_writer_m) = &self.links_writer {
            let mut links_writer = links_writer_m.lock().await;
            links_writer.flush().await?;
        }

        if let Some(anchors_writer_m) = &self.anchors_writer {
            let mut anchors_writer = anchors_writer_m.lock().await;
            anchors_writer.flush().await?;
        }

        Ok(())
    }
}
