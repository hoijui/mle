// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Write;
use std::sync::Mutex;

use crate::config::Config;
use crate::link::Link;
use crate::{anchor::Anchor, path_buf::PathBuf};

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
    fn write_header(&self) -> std::io::Result<()> {
        let mut writer = self.stream.lock().expect("we do not use MT");
        writeln!(
            writer,
            r"
# Report of Found Links and/or Anchors
"
        )?;
        // TODO Add more info here, like date and time the extraction was run, and version of the extraction tool used
        if self.extended {
            writeln!(
                writer,
                r"## Scanned markup files
"
            )?;
            for markup_file in &self.markup_files {
                writeln!(writer, "- `{}`", markup_file.display())?;
            }
        }
        Ok(())
    }

    fn write_header_conditionally(&mut self) -> std::io::Result<()> {
        if self.header_written {
            Ok(())
        } else {
            self.header_written = true;
            self.write_header()
        }
    }

    fn write_links_header(&mut self) -> std::io::Result<()> {
        if self.links_header_written {
            return Ok(());
        }
        self.links_header_written = true;
        self.write_header_conditionally()?;
        let mut writer = self.stream.lock().expect("we do not use MT");
        writeln!(
            writer,
            r"
## Links
"
        )?;
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
            )?;
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
            )?;
        } else {
            writeln!(
                writer,
                "File \
| Line \
| Column \
| Target \
| Fragment \
|"
            )?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
| --- \
|"
            )?;
        }
        Ok(())
    }

    fn write_anchors_header(&mut self) -> std::io::Result<()> {
        if self.anchors_header_written {
            return Ok(());
        }
        self.anchors_header_written = true;
        self.write_header_conditionally()?;
        let mut writer = self.stream.lock().expect("we do not use MT");
        writeln!(
            writer,
            r"
## Anchors
"
        )?;
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
            )?;
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
            )?;
        } else {
            writeln!(
                writer,
                "File \
| Line \
| Column \
| Name \
|"
            )?;
            writeln!(
                writer,
                "| --- \
| --- \
| --- \
| --- \
|"
            )?;
        }
        Ok(())
    }

    // fn write_footer(&mut self) -> std::io::Result<()> {
    //     Ok(())
    // }
}

impl super::Sink for Sink {
    fn init(
        _format: Type,
        config: &Config,
        links_stream: WriterOpt,
        anchors_stream: WriterOpt,
    ) -> std::io::Result<Box<dyn super::Sink>> {
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

    fn sink_link(&mut self, link: &Link) -> std::io::Result<()> {
        self.write_links_header()?;
        // let rec = LinkRec::new(link, self.extended);
        let mut writer = self.stream.lock().expect("we do not use MT");
        // writeln!(writer, "{link}")?;
        let target_without_fragment = link.target.without_fragment().to_string();
        let target_no_frag = if target_without_fragment.is_empty() {
            String::new()
        } else {
            format!("[`{target_without_fragment}`]({target_without_fragment}) ",)
        };
        let target_frag = link.target.fragment().map_or_else(String::new, |fragment| {
            format!("[`{}`]({}) ", fragment, link.target)
        });
        if self.extended {
            writeln!(
                writer,
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
            )?;
        } else {
            writeln!(
                writer,
                "| [`{}`]({}) | {} | {} | {}| {}|",
                link.source.file,
                link.source.file,
                link.source.pos.line,
                link.source.pos.column,
                target_no_frag,
                target_frag,
            )?;
        }
        if self.flush {
            writer.flush()?;
        }
        Ok(())
    }

    fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()> {
        self.write_anchors_header()?;
        let mut writer = self.stream.lock().expect("we do not use MT");
        // writeln!(writer, "{anchor}")?;
        if self.extended {
            writeln!(
                writer,
                "| [`{}`]({}) | {} | {} | {} | {} | {} | {} | {} | {:#?} |",
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
            )?;
        } else {
            writeln!(
                writer,
                "| [`{}`]({}) | {} | {} | {} |",
                anchor.source.file,
                anchor.source.file,
                anchor.source.pos.line,
                anchor.source.pos.column,
                anchor.name,
            )?;
        }
        if self.flush {
            writer.flush()?;
        }
        Ok(())
    }

    fn finalize(&mut self) -> std::io::Result<()> {
        // self.write_footer()
        Ok(())
    }
}
