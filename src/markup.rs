use std::{borrow::Cow, fs, rc::Rc, str::FromStr};

use crate::link::{FileLoc, Position};

#[derive(Debug, PartialEq, Clone)]
pub enum Content<'a> {
    /// stores the file-name
    LocalFile(String),
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
pub struct MarkupFile<'a> {
    pub markup_type: MarkupType,
    pub locator: Rc<FileLoc>,
    pub content: Content<'a>,
    /// The first position of the above `content` is at this location.
    /// In the normal sense, this is line 0, column 0,
    /// but in the case of e.g. in-line HTML in a Markdown file,
    /// this would commonly be a larger position.
    pub start: Position,
}

#[derive(Debug, Clone, Copy)]
pub enum MarkupType {
    Markdown,
    Html,
}

impl Default for MarkupType {
    fn default() -> Self {
        Self::Markdown
    }
}

impl FromStr for MarkupType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<MarkupType, Self::Err> {
        match s {
            "md" => Ok(MarkupType::Markdown),
            "html" => Ok(MarkupType::Html),
            _ => Err("Unknown markup file extension"),
        }
    }
}

impl MarkupType {
    #[must_use]
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            MarkupType::Markdown => vec![
                "md",
                "markdown",
                "mkdown",
                "mkdn",
                "mkd",
                "mdwn",
                "mdtxt",
                "mdtext",
                "text",
                "rmd",
            ],
            MarkupType::Html => vec!["htm", "html", "xhtml"],
        }
    }
}

impl<'a> MarkupFile<'a> {
    #[must_use]
    pub fn dummy(content: &'a str, markup_type: MarkupType) -> Self {
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
        for mt in [MarkupType::Markdown, MarkupType::Html] {
            for ext in mt.file_extensions() {
                assert_eq!(ext, ext.to_lowercase());
            }
        }
    }
}
