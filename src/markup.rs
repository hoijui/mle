use std::{borrow::Cow, fs, rc::Rc, str::FromStr};

use crate::link::{FileLoc, Position};

#[derive(Debug, PartialEq, Clone)]
pub enum Content<'a> {
    LocalFile(String), // stores the file-name
    InMemory(&'a str), // stores the whole content of the file as a string
                       // URL(Url, &'a str)
}

impl<'a> Content<'a> {
    pub fn fetch(&self) -> Result<Cow<'a, str>, std::io::Error> {
        match self {
            Self::LocalFile(file_name) => fs::read_to_string(file_name).map(Cow::Owned),
            Self::InMemory(content) => Ok(Cow::Borrowed(content)),
        }
    }
}

#[derive(Debug)]
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

impl FromStr for MarkupType {
    type Err = ();

    fn from_str(s: &str) -> Result<MarkupType, ()> {
        match s {
            "md" => Ok(MarkupType::Markdown),
            "html" => Ok(MarkupType::Html),
            _ => Err(()),
        }
    }
}

impl MarkupType {
    #[must_use]
    pub fn file_extensions(&self) -> Vec<String> {
        match self {
            MarkupType::Markdown => vec![
                "md".to_string(),
                "markdown".to_string(),
                "mkdown".to_string(),
                "mkdn".to_string(),
                "mkd".to_string(),
                "mdwn".to_string(),
                "mdtxt".to_string(),
                "mdtext".to_string(),
                "text".to_string(),
                "rmd".to_string(),
            ],
            MarkupType::Html => vec!["htm".to_string(), "html".to_string(), "xhtml".to_string()],
        }
    }
}

impl<'a> MarkupFile<'a> {
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
