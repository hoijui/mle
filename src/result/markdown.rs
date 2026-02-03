// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use async_std::io;
use async_std::io::WriteExt;
use async_trait::async_trait;
use cli_utils::path_buf::PathBuf;
use tokio::sync::Mutex;

use crate::anchor::Anchor;
use crate::config::Config;
use crate::link::Link;

use super::{Type, Writer, WriterOpt};

#[allow(clippy::struct_excessive_bools)]
pub struct Sink {
    extended: bool,
    markup_files: Vec<PathBuf>,
    flush: bool,
    stream: Mutex<Writer>,
    header_written: bool,
    links_header_written: bool,
    anchors_header_written: bool,
}

impl Sink {
    #[allow(clippy::significant_drop_tightening)]
    async fn write_header(&self) -> io::Result<()> {
        let mut writer = self.stream.lock().await;
        writeln!(
            writer,
            r"
# Report of Found Links and/or Anchors
"
        )
        .await?;
        // TODO Add more info here, like date and time the extraction was run, and version of the extraction tool used
        if self.extended {
            writeln!(
                writer,
                r"## Scanned markup files
"
            )
            .await?;
            for markup_file in &self.markup_files {
                let line = format!("- `{}`", markup_file.display());
                writer.write_all(line.as_bytes()).await?;
            }
        }
        Ok(())
    }

    async fn write_header_conditionally(&mut self) -> io::Result<()> {
        if self.header_written {
            Ok(())
        } else {
            self.header_written = true;
            self.write_header().await
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn write_links_header(&mut self) -> io::Result<()> {
        if self.links_header_written {
            return Ok(());
        }
        self.links_header_written = true;
        self.write_header_conditionally().await?;
        let mut writer = self.stream.lock().await;
        writeln!(
            writer,
            r"
## Links
"
        )
        .await?;
        if self.extended {
            writeln!(
                writer,
                "Source-File \
| Source-Line \
| Source-Column \
| Source-is-File \
| Source-is-URL \
| Source-is-Local \
| Source-is-Remote \
| Target \
| Target-Fragment \
| Target-is-File \
| Target-is-URL \
| Target-is-Local \
| Target-is-Remote \
|"
            )
            .await?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
|"
            )
            .await?;
        } else {
            writeln!(
                writer,
                "File \
| Line \
| Column \
| Target \
| Fragment \
|"
            )
            .await?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
| --- \
|"
            )
            .await?;
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn write_anchors_header(&mut self) -> io::Result<()> {
        if self.anchors_header_written {
            return Ok(());
        }
        self.anchors_header_written = true;
        self.write_header_conditionally().await?;
        let mut writer = self.stream.lock().await;
        writeln!(
            writer,
            r"
## Anchors
"
        )
        .await?;
        if self.extended {
            writeln!(
                writer,
                "File \
| Line \
| Column \
| is-File \
| is-URL \
| is-Local \
| is-Remote \
| Name \
| Type \
|"
            )
            .await?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
| --- \
|"
            )
            .await?;
        } else {
            writeln!(
                writer,
                "File \
| Line \
| Column \
| Name \
|"
            )
            .await?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
|"
            )
            .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl super::Sink for Sink {
    async fn init(
        _format: Type,
        config: &Config,
        links_stream: WriterOpt,
        anchors_stream: WriterOpt,
    ) -> io::Result<Box<dyn super::Sink>> {
        if links_stream.is_some() && anchors_stream.is_some() {
            log::warn!(
                "Ignoring destination for anchors, \
because the chosen output format writes everything into one file."
            );
        }
        let stream = links_stream
            .or(anchors_stream)
            .expect("Either links or anchors (or both) to have an output target");
        Ok(Box::new(Self {
            extended: config.result_extended,
            markup_files: if config.result_extended {
                config.markup_files.clone()
            } else {
                vec![]
            },
            flush: config.result_flush,
            stream: Mutex::new(stream),
            header_written: false,
            links_header_written: false,
            anchors_header_written: false,
        }) as Box<dyn super::Sink>)
    }

    #[allow(clippy::significant_drop_tightening)]
    #[allow(clippy::similar_names)]
    async fn sink_link(&mut self, link: &Link) -> io::Result<()> {
        self.write_links_header().await?;
        let mut writer = self.stream.lock().await;
        let target_without_fragment = link.target.without_fragment().to_string();
        let target_no_frag = if target_without_fragment.is_empty() {
            String::new()
        } else {
            format!("[`{target_without_fragment}`]({target_without_fragment}) ",)
        };
        let target_frag = link.target.fragment().map_or_else(String::new, |fragment| {
            format!("[`{}`]({}) ", fragment, link.target)
        });
        let line = if self.extended {
            format!(
                "| [`{}`]({}) | {} | {} | {} | {} | {} | {} | {}| {}| {} | {} | {} | {} |",
                link.source.file,
                link.source.file,
                link.source.pos.line,
                link.source.pos.column,
                link.source.file.is_file_system(),
                link.source.file.is_url(),
                link.source.file.is_local(),
                link.source.file.is_remote(),
                target_no_frag,
                target_frag,
                link.target.is_file_system(),
                link.target.is_url(),
                link.target.is_local(),
                link.target.is_remote(),
            )
        } else {
            format!(
                "| [`{}`]({}) | {} | {} | {}| {}|",
                link.source.file,
                link.source.file,
                link.source.pos.line,
                link.source.pos.column,
                target_no_frag,
                target_frag,
            )
        };
        writer.write_all(line.as_bytes()).await?;
        if self.flush {
            writer.flush().await?;
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn sink_anchor(&mut self, anchor: &Anchor) -> io::Result<()> {
        self.write_anchors_header().await?;
        let mut writer = self.stream.lock().await;
        let line = if self.extended {
            format!(
                "| [`{}`]({}) | {} | {} | {} | {} | {} | {} | {} | {} |",
                anchor.source.file,
                anchor.source.file,
                anchor.source.pos.line,
                anchor.source.pos.column,
                anchor.source.file.is_file_system(),
                anchor.source.file.is_url(),
                anchor.source.file.is_local(),
                anchor.source.file.is_remote(),
                anchor.name,
                anchor.r#type,
            )
        } else {
            format!(
                "| [`{}`]({}) | {} | {} | {} |",
                anchor.source.file,
                anchor.source.file,
                anchor.source.pos.line,
                anchor.source.pos.column,
                anchor.name,
            )
        };
        writer.write_all(line.as_bytes()).await?;
        if self.flush {
            writer.flush().await?;
        }
        Ok(())
    }

    async fn finalize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
