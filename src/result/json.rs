// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_std::io::WriteExt;
use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::Tool as Config;
use crate::link::Link;
use crate::{anchor::Anchor, result::Type};

use super::{AnchorOwnedRec, LinkOwnedRec, Writer, WriterOpt};

#[derive(Serialize)]
pub struct Sink {
    #[serde(skip)]
    extended: bool,
    #[serde(skip)]
    links_stream: Option<Mutex<Writer>>,
    #[serde(skip)]
    anchors_stream: Option<Mutex<Writer>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<LinkOwnedRec>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    anchors: Vec<AnchorOwnedRec>,
}

#[async_trait]
impl super::Sink for Sink {
    async fn init(
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

    async fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        self.links.push(LinkOwnedRec::new(link, self.extended));
        Ok(())
    }

    async fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        self.anchors
            .push(AnchorOwnedRec::new(anchor, self.extended));
        Ok(())
    }

    // There are two false positives reported by clippy::significant_drop_tightening here
    #[allow(clippy::significant_drop_tightening)]
    async fn finalize(&mut self) -> std::io::Result<()> {
        let mut writer = if let Some(links_writer_m) = &self.links_stream {
            // NOTE This is also the branch used if both outptu streams are equal!
            links_writer_m.lock().await
        } else if let Some(anchors_writer_m) = &self.anchors_stream {
            anchors_writer_m.lock().await
        } else {
            panic!("We need always either a links or an anchors stream to write to");
        };
        let json = serde_json::to_string_pretty(&self)?;
        writer.write_all(json.as_bytes()).await?;
        Ok(())
    }
}
