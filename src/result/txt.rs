// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_std::io::{self, WriteExt};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;
use crate::result::Type;

use super::{Writer, WriterOpt};

pub struct Sink {
    flush: bool,
    links_stream: Option<Mutex<Writer>>,
    anchors_stream: Option<Mutex<Writer>>,
}

#[async_trait]
impl super::Sink for Sink {
    async fn init(
        _format: Type,
        config: &Config,
        links_stream: WriterOpt,
        anchors_stream: WriterOpt,
    ) -> io::Result<Box<dyn super::Sink>> {
        Ok(Box::new(Self {
            flush: config.result_flush,
            links_stream: links_stream.map(Mutex::new),
            anchors_stream: anchors_stream.map(Mutex::new),
        }) as Box<dyn super::Sink>)
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn sink_link(&mut self, link: &Link) -> io::Result<()> {
        if let Some(links_writer_m) = &mut self.links_stream {
            let mut links_writer = links_writer_m.lock().await;
            let link_str = format!("L:{link}\n");
            links_writer.write_all(link_str.as_bytes()).await?;
            if self.flush {
                links_writer.flush().await?;
            }
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn sink_anchor(&mut self, anchor: &Anchor) -> io::Result<()> {
        if let Some(ref anchors_writer_m) = self.anchors_stream {
            let mut anchors_writer = anchors_writer_m.lock().await;
            let anchor_str = format!("A:{anchor}\n");
            anchors_writer.write_all(anchor_str.as_bytes()).await?;
            if self.flush {
                anchors_writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn finalize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
