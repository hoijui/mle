// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, fs, path::PathBuf, rc::Rc, str::FromStr};

use crate::link::{FileLoc, Position};

#[derive(Debug, PartialEq, Clone)]
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
    pub fn fetch(&self) -> Result<Cow<'a, str>, std::io::Error> {
        match self {
            Self::LocalFile(file_name) => fs::read_to_string(file_name).map(Cow::Owned),
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

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Markdown,
    Html,
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
    #[must_use]
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Markdown => vec![
                "md", "markdown", "mkdown", "mkdn", "mkd", "mdwn", "mdtxt", "mdtext", "text", "rmd",
            ],
            Self::Html => vec!["htm", "html", "xhtml"],
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
