
use std::{borrow::Cow, fs, str::FromStr};

#[derive(Debug, PartialEq, Clone)]
pub enum Content<'a> {
    LocalFile(&'a str), // stores the file-name
    InMemory(&'a str),  // stores the whole content of the file as a string
                        // URL(Url, &'a str)
}

impl<'a> Content<'a> {
    pub fn fetch(&self) -> Result<Cow<'a, str>, std::io::Error> {
        match self {
            &Self::LocalFile(file_name) => {
                fs::read_to_string(file_name).map(|content| Cow::Owned(content))
            }
            &Self::InMemory(content) => Ok(Cow::Borrowed(content)),
        }
    }
}

#[derive(Debug)]
pub struct MarkupFile<'a> {
    pub markup_type: MarkupType,
    pub locator: &'a str, // local file path or URL
    pub content: Content<'a>,
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
