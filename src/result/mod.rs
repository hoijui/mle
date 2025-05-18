// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod csv;
mod json;
mod txt;

use std::{
    fs::File,
    io::{ErrorKind, Write},
    str::FromStr,
};

// #[cfg(feature = "async")]
// use async_std::path::PathBuf;
// #[cfg(not(feature = "async"))]
// use std::path::PathBuf;
use crate::path_buf::PathBuf;

use clap::{builder::PossibleValue, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{
    anchor::{self, Anchor},
    config::Config,
    link::Link,
    BoxError,
};

type Writer = Option<Box<dyn Write + 'static>>;

const EXT_TEXT: &str = "txt";
const EXT_MARKDOWN: &str = "md";
const EXT_CSV: &str = "csv";
const EXT_JSON: &str = "json";
const EXT_RDF_TURTLE: &str = "ttl";
const ALL_EXTS: [&str; 5] = [EXT_TEXT, EXT_MARKDOWN, EXT_CSV, EXT_JSON, EXT_RDF_TURTLE];

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Type {
    #[default]
    Text,
    Markdown,
    Csv,
    // Tsv,
    Json,
    RdfTurtle,
}

impl ValueEnum for Type {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Text,
            Self::Markdown,
            Self::Csv,
            Self::Json,
            Self::RdfTurtle,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(self.as_str().into())
    }
}

impl Type {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => EXT_TEXT,
            Self::Markdown => EXT_MARKDOWN,
            Self::Csv => EXT_CSV,
            Self::Json => EXT_JSON,
            Self::RdfTurtle => EXT_RDF_TURTLE,
        }
    }
}

impl FromStr for Type {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            EXT_TEXT | "text" | "plain" | "grep" => Self::Text,
            EXT_MARKDOWN | "markdown" => Self::Markdown,
            EXT_CSV => Self::Csv,
            EXT_JSON => Self::Json,
            EXT_RDF_TURTLE | "turtle" | "rdf" | "rdf-turtle" => Self::RdfTurtle,
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid result format given: '{}' \nValid formats are: {}",
                    s,
                    ALL_EXTS.join(", ")
                ),
            ))?,
        })
    }
}

fn construct_out_stream(specifier: &Option<PathBuf>) -> Box<dyn Write + 'static> {
    specifier.as_ref().map_or_else(
        || Box::new(std::io::stdout()) as Box<dyn Write>,
        |file_path| Box::new(File::create(file_path).unwrap()) as Box<dyn Write>,
    )
}

/// Pretty-prints a list of errors to `log::error!`.
pub fn write_to_stderr(errors: &[BoxError]) {
    for error in errors {
        log::error!("{error:#?}");
    }
}

/// Write results to stdout or file.
///
/// # Errors
/// (I/)O-error when writing to a file.
pub fn sink(
    config: &Config,
    links: &[Link],
    anchors: &[Anchor],
    errors: &[BoxError],
) -> std::io::Result<()> {
    let sink_init = match config.result_format {
        Type::Text => txt::Sink::init,
        Type::Json => json::Sink::init,
        Type::Csv => csv::Sink::init,
        _ => Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Result format not yet supported",
        ))?,
    };
    let links_writer = config.links.as_ref().map(construct_out_stream);
    let anchors_writer = config.anchors.as_ref().map(construct_out_stream);
    let mut sink = sink_init(config, links_writer, anchors_writer)?;
    for link in links {
        // thread::sleep::sleep(std::time::Duration::new(0, 200000000));
        sink.sink_link(link)?;
    }
    for anchor in anchors {
        sink.sink_anchor(anchor)?;
    }
    for error in errors {
        sink.sink_error(error)?;
    }
    sink.finalize()
}

pub trait Sink {
    /// Initilaizes this sink.
    /// This will be called once only,
    /// and before any `sink_*` function may be called.
    ///
    /// # Errors
    /// If writing to a file or other (I)/O-device failed.
    fn init(
        config: &Config,
        links_stream: Writer,
        anchors_stream: Writer,
    ) -> std::io::Result<Box<dyn Sink>>
    where
        Self: Sized;

    /// Writes-out an extracted link.
    ///
    /// # Errors
    /// If writing to the output stream for links failed.
    fn sink_link(&mut self, link: &Link) -> std::io::Result<()>;

    /// Writes-out an extracted anchor.
    ///
    /// # Errors
    /// If writing to the output stream for anchors failed.
    fn sink_anchor(&mut self, anchor: &Anchor) -> std::io::Result<()>;

    /// Writes-out an error generated while extracting links/anchors.
    ///
    /// # Errors
    /// If writing to the output stream for errors failed.
    fn sink_error(&mut self, error: &BoxError) -> std::io::Result<()> {
        log::error!("{error:#?}");
        Ok(())
    }

    /// Finalizes/Clsoes this sink.
    /// This will be caleld exactly once,
    /// and no `sink_*` functions may be called after this function has been called.
    ///
    /// # Errors
    /// If writing to a file or other (I)/O-device failed.
    fn finalize(&mut self) -> std::io::Result<()>;
}

#[derive(Debug, Serialize)]
struct LinkExtendedRec<'a> {
    src_file: String,
    src_line: usize,
    src_column: usize,
    src_is_file_system: bool,
    src_is_url: bool,
    src_is_local: bool,
    src_is_remote: bool,
    trg_link: String,
    trg_fragment: Option<&'a str>,
    trg_is_file_system: bool,
    trg_is_url: bool,
    trg_is_local: bool,
    trg_is_remote: bool,
}

#[derive(Debug, Serialize)]
struct LinkSimpleRec<'a> {
    src_file: String,
    src_line: usize,
    src_column: usize,
    trg_link: String,
    trg_fragment: Option<&'a str>,
}

#[derive(Debug)]
enum LinkRec<'a> {
    Simple(LinkSimpleRec<'a>),
    Extended(LinkExtendedRec<'a>),
}

impl<'a> LinkRec<'a> {
    fn new(lnk: &'a Link, extended: bool) -> Self {
        if extended {
            Self::Extended(LinkExtendedRec {
                src_file: lnk.source.file.to_string(),
                src_line: lnk.source.pos.line,
                src_column: lnk.source.pos.column,
                src_is_file_system: lnk.source.file.is_file_system(),
                src_is_url: lnk.source.file.is_url(),
                src_is_local: lnk.source.file.is_local(),
                src_is_remote: lnk.source.file.is_remote(),
                trg_link: lnk.target.without_fragment().to_string(),
                trg_fragment: lnk.target.fragment(),
                trg_is_file_system: lnk.target.is_file_system(),
                trg_is_url: lnk.target.is_url(),
                trg_is_local: lnk.target.is_local(),
                trg_is_remote: lnk.target.is_remote(),
            })
        } else {
            Self::Simple(LinkSimpleRec {
                src_file: lnk.source.file.to_string(),
                src_line: lnk.source.pos.line,
                src_column: lnk.source.pos.column,
                trg_link: lnk.target.without_fragment().to_string(),
                trg_fragment: lnk.target.fragment(),
            })
        }
    }
}

impl Serialize for LinkRec<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Simple(rec) => rec.serialize(serializer),
            Self::Extended(rec) => rec.serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize)]
struct LinkExtendedOwnedRec {
    src_file: String,
    src_line: usize,
    src_column: usize,
    src_is_file_system: bool,
    src_is_url: bool,
    src_is_local: bool,
    src_is_remote: bool,
    trg_link: String,
    trg_fragment: Option<String>,
    trg_is_file_system: bool,
    trg_is_url: bool,
    trg_is_local: bool,
    trg_is_remote: bool,
}

#[derive(Debug, Serialize)]
struct LinkSimpleOwnedRec {
    src_file: String,
    src_line: usize,
    src_column: usize,
    trg_link: String,
    trg_fragment: Option<String>,
}

#[derive(Debug)]
enum LinkOwnedRec {
    Simple(LinkSimpleOwnedRec),
    Extended(LinkExtendedOwnedRec),
}

impl LinkOwnedRec {
    fn new(lnk: &Link, extended: bool) -> Self {
        if extended {
            Self::Extended(LinkExtendedOwnedRec {
                src_file: lnk.source.file.to_string(),
                src_line: lnk.source.pos.line,
                src_column: lnk.source.pos.column,
                src_is_file_system: lnk.source.file.is_file_system(),
                src_is_url: lnk.source.file.is_url(),
                src_is_local: lnk.source.file.is_local(),
                src_is_remote: lnk.source.file.is_remote(),
                trg_link: lnk.target.without_fragment().to_string(),
                trg_fragment: lnk.target.fragment().map(ToOwned::to_owned),
                trg_is_file_system: lnk.target.is_file_system(),
                trg_is_url: lnk.target.is_url(),
                trg_is_local: lnk.target.is_local(),
                trg_is_remote: lnk.target.is_remote(),
            })
        } else {
            Self::Simple(LinkSimpleOwnedRec {
                src_file: lnk.source.file.to_string(),
                src_line: lnk.source.pos.line,
                src_column: lnk.source.pos.column,
                trg_link: lnk.target.without_fragment().to_string(),
                trg_fragment: lnk.target.fragment().map(ToOwned::to_owned),
            })
        }
    }
}

impl Serialize for LinkOwnedRec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Simple(rec) => rec.serialize(serializer),
            Self::Extended(rec) => rec.serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize)]
struct AnchorExtendedRec<'a> {
    src_file: String,
    src_line: usize,
    src_column: usize,
    src_is_file_system: bool,
    src_is_url: bool,
    src_is_local: bool,
    src_is_remote: bool,
    name: &'a str,
    r#type: anchor::Type,
}

#[derive(Debug, Serialize)]
struct AnchorSimpleRec<'a> {
    src_file: String,
    src_line: usize,
    src_column: usize,
    name: &'a str,
}

#[derive(Debug)]
enum AnchorRec<'a> {
    Simple(AnchorSimpleRec<'a>),
    Extended(AnchorExtendedRec<'a>),
}

impl Serialize for AnchorRec<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Simple(rec) => rec.serialize(serializer),
            Self::Extended(rec) => rec.serialize(serializer),
        }
    }
}

impl<'a> AnchorRec<'a> {
    fn new(anc: &'a Anchor, extended: bool) -> Self {
        if extended {
            Self::Extended(AnchorExtendedRec {
                src_file: anc.source.file.to_string(),
                src_line: anc.source.pos.line,
                src_column: anc.source.pos.column,
                src_is_file_system: anc.source.file.is_file_system(),
                src_is_url: anc.source.file.is_url(),
                src_is_local: anc.source.file.is_local(),
                src_is_remote: anc.source.file.is_remote(),
                name: &anc.name,
                // r#type: format!("{:?}", anc.r#type),
                r#type: anc.r#type,
            })
        } else {
            Self::Simple(AnchorSimpleRec {
                src_file: anc.source.file.to_string(),
                src_line: anc.source.pos.line,
                src_column: anc.source.pos.column,
                name: &anc.name,
            })
        }
    }
}

#[derive(Debug, Serialize)]
struct AnchorExtendedOwnedRec {
    src_file: String,
    src_line: usize,
    src_column: usize,
    src_is_file_system: bool,
    src_is_url: bool,
    src_is_local: bool,
    src_is_remote: bool,
    name: String,
    r#type: anchor::Type,
}

#[derive(Debug, Serialize)]
struct AnchorSimpleOwnedRec {
    src_file: String,
    src_line: usize,
    src_column: usize,
    name: String,
}

#[derive(Debug)]
enum AnchorOwnedRec {
    Simple(AnchorSimpleOwnedRec),
    Extended(AnchorExtendedOwnedRec),
}
impl Serialize for AnchorOwnedRec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Simple(rec) => rec.serialize(serializer),
            Self::Extended(rec) => rec.serialize(serializer),
        }
    }
}

impl AnchorOwnedRec {
    fn new(anc: &Anchor, extended: bool) -> Self {
        if extended {
            Self::Extended(AnchorExtendedOwnedRec {
                src_file: anc.source.file.to_string(),
                src_line: anc.source.pos.line,
                src_column: anc.source.pos.column,
                src_is_file_system: anc.source.file.is_file_system(),
                src_is_url: anc.source.file.is_url(),
                src_is_local: anc.source.file.is_local(),
                src_is_remote: anc.source.file.is_remote(),
                name: anc.name.to_string(),
                // r#type: format!("{:?}", anc.r#type),
                r#type: anc.r#type,
            })
        } else {
            Self::Simple(AnchorSimpleOwnedRec {
                src_file: anc.source.file.to_string(),
                src_line: anc.source.pos.line,
                src_column: anc.source.pos.column,
                name: anc.name.to_string(),
            })
        }
    }
}
