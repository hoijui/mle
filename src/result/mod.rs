// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io::ErrorKind, str::FromStr};

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
