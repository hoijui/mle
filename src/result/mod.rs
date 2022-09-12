// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod json;
mod txt;

use std::{
    fs::File,
    io::{ErrorKind, Write},
    path::Path,
    str::FromStr,
};

use clap::{PossibleValue, ValueEnum};

use crate::{anchor::Anchor, config::Config, group::Grouping};

const EXT_TEXT: &str = "txt";
const EXT_MARKDOWN: &str = "md";
const EXT_CSV: &str = "csv";
const EXT_JSON: &str = "json";
const EXT_RDF_TURTLE: &str = "ttl";
const ALL_EXTS: [&str; 5] = [EXT_TEXT, EXT_MARKDOWN, EXT_CSV, EXT_JSON, EXT_RDF_TURTLE];

#[derive(Default, Debug, Clone, Copy)]
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

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
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

fn construct_out_stream(specifier: Option<&str>) -> Box<dyn Write + 'static> {
    match specifier {
        None | Some("-") => Box::new(std::io::stdout()) as Box<dyn Write>,
        Some(file_path) => {
            let path = Path::new(file_path);
            Box::new(File::create(&path).unwrap()) as Box<dyn Write>
        }
    }
}

/// Write results to stdout or file.
///
/// # Errors
/// (I/)O-error when writing to a file.
pub fn sink(
    config: &Config,
    links: &Grouping,
    anchors: &[Anchor],
    errors: &[Box<dyn std::error::Error>],
) -> std::io::Result<()> {
    let sink: Box<dyn Sink> = match config.result_format {
        Type::Text => Box::new(txt::Sink()),
        Type::Json => Box::new(json::Sink()),
        _ => Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Result format not yet supported",
        ))?,
    };
    let mut out_writer = construct_out_stream(config.result_file);
    sink.write_results(config, &mut out_writer, links, anchors, errors)
}

pub trait Sink {
    /// Writes-out the extraction results.
    ///
    /// # Errors
    /// If writing to a file or other (I)/O-device failed.
    fn write_results(
        &self,
        config: &Config,
        out_stream: &mut Box<dyn Write>,
        links: &Grouping,
        anchors: &[Anchor],
        errors: &[Box<dyn std::error::Error>],
    ) -> std::io::Result<()>;
}
