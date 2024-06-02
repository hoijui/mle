// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{builder::PossibleValue, ValueEnum};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ffi::OsStr, rc::Rc, str::FromStr};

use crate::path_buf::PathBuf;
use async_std::{fs, path::Path};

use crate::link::{FileLoc, Position};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Content<'a> {
    /// stores the file-name
    LocalFile(PathBuf),
    /// stores the whole content of the file as a string
    InMemory(&'a str),
    // Url(Url, &'a str)
}

impl<'a> Content<'a> {
    /// Returns the actual content as str
    ///
    /// # Errors
    /// If the content has to be read from a URL or the File-System,
    /// there might be an read error.
    pub async fn fetch(&self) -> Result<Cow<'a, str>, std::io::Error> {
        match self {
            Self::LocalFile(file_name) => fs::read_to_string(file_name).await.map(Cow::Owned),
            Self::InMemory(content) => Ok(Cow::Borrowed(content)),
        }
    }
}

impl Default for Content<'_> {
    fn default() -> Self {
        Self::InMemory("")
    }
}

#[derive(Debug, Default)]
pub struct File<'a> {
    pub markup_type: Type,
    pub locator: Rc<FileLoc>,
    pub content: Content<'a>,
    /// The first position of the above `content` is at this location.
    /// In the normal sense, this is line 0, column 0,
    /// but in the case of e.g. in-line HTML in a Markdown file,
    /// this would commonly be a larger position.
    pub start: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Type {
    Markdown,
    Html,
}

// Can also be derived with feature flag `#[derive(ArgEnum)]`
impl ValueEnum for Type {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Markdown, Self::Html]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(self.as_str().into())
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Markdown
    }
}

impl FromStr for Type {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            _ => Err("Unknown markup file extension"),
        }
    }
}

impl Type {
    fn get_extension_from_filename(file_name: &str) -> Option<&str> {
        Path::new(file_name).extension().and_then(OsStr::to_str)
    }

    /// Analyzes whether a name of a file is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    #[must_use]
    pub fn is_markup_file(file_name: &str) -> bool {
        let ext_opt = Self::get_extension_from_filename(file_name);
        if let Some(ext) = ext_opt {
            for t in Self::value_variants() {
                for known_ext in t.file_extensions() {
                    if ext == known_ext {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Analyzes whether a URL, if pointing to a file, is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    #[must_use]
    pub fn is_markup_url(url: &Url) -> bool {
        url.path_segments().map_or(false, |path_segments| {
            path_segments.last().map_or(false, |last_path_segment| {
                Self::is_markup_file(last_path_segment)
            })
        })
    }

    #[must_use]
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Markdown => vec![
                "md", "markdown", "mkdown", "mkdn", "mkd", "mdwn", "mdtxt", "mdtext", "text", "rmd",
            ],
            Self::Html => vec!["htm", "html", "xhtml"],
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
        }
    }
}

impl<'a> File<'a> {
    #[must_use]
    pub fn dummy(content: &'a str, markup_type: Type) -> Self {
        Self {
            content: Content::InMemory(content),
            markup_type,
            locator: FileLoc::dummy(),
            start: Position::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lowercase_file_extensions() {
        for mt in [Type::Markdown, Type::Html] {
            for ext in mt.file_extensions() {
                assert_eq!(ext, ext.to_lowercase());
            }
        }
    }
}
