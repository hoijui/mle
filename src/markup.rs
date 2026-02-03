// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{ValueEnum, builder::PossibleValue};
use cli_utils::{
    file_traversal::{self, PathFilterRet, create_combined_filter},
    ignore_path::IgnorePath,
    path_buf::PathBuf,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ffi::OsStr, str::FromStr, sync::Arc};
use thiserror::Error;
use url::Url;

use crate::markup;
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
    pub locator: Arc<FileLoc>,
    pub content: Content<'a>,
    /// The first position of the above `content` is at this location.
    /// In the normal sense, this is line 0, column 0,
    /// but in the case of e.g. in-line HTML in a Markdown file,
    /// this would commonly be a larger position.
    pub start: Position,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    #[default]
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

#[derive(Debug, Error)]
pub enum TypeExtractionError {
    #[error("File extension '{0}' does not match any supported markup type")]
    UnsupportedFileExt(String),
    #[error("File has no extension, so we can not determine markup type")]
    NoFileExt,
}

impl Type {
    fn try_from_file_name(file_name: impl AsRef<str>) -> Result<Self, TypeExtractionError> {
        let ext_opt = Self::get_extension_from_filename(file_name.as_ref());
        if let Some(ext) = ext_opt {
            let ext_lower = ext.to_lowercase();
            log::warn!("Extracted file ext: {ext_lower}");
            for t in Self::value_variants() {
                for known_ext in t.file_extensions() {
                    if ext_lower == known_ext {
                        return Ok(*t);
                    }
                }
            }
            Err(TypeExtractionError::UnsupportedFileExt(ext.to_string()))
        } else {
            Err(TypeExtractionError::NoFileExt)
        }
    }
}

impl TryFrom<&Path> for Type {
    type Error = TypeExtractionError;

    fn try_from(path: &Path) -> Result<Self, TypeExtractionError> {
        let file_name = path.display().to_string();
        Self::try_from_file_name(&file_name)
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
        Self::try_from_file_name(file_name).is_ok()
    }

    #[must_use]
    pub fn create_filter(types: Vec<Self>) -> Box<dyn Fn(&Path) -> PathFilterRet + Send + Sync> {
        Box::new(move |file: &Path| {
            let file_name_os_str = file
                .file_name()
                .map(OsStr::to_string_lossy)
                // .ok_or_else(|| Error::MissingFileName(file.into()))
                .ok_or_else(|| {
                    std::io::Error::other(format!(
                        "Missing file-name for path: '{}'",
                        file.display()
                    ))
                })?;

            if let Ok(extracted_type) = Self::try_from_file_name(file_name_os_str.as_ref())
                && types.contains(&extracted_type)
            {
                return Ok(true);
            }

            // #[cfg(feature = "logging")]
            log::trace!(
                "Not a file of a configured markup type: '{}'",
                file.display()
            );
            Ok(false)
        })
    }

    /// Searches for markup source files according to the configuration,
    /// and stores them in `result`.
    ///
    /// # Errors
    ///
    /// If a file or path supplied does not exist,
    /// or if any file supplied or found through scanning has no name (e.g. '.').
    /// The code-logic should prevent the second case from ever happening.
    pub async fn find(
        root: &Path,
        markup_types: Vec<Self>,
        ignore_paths: Vec<IgnorePath>,
    ) -> Result<Vec<PathBuf>, file_traversal::Error> {
        let filters = vec![
            Box::new(Self::create_filter(markup_types)),
            Box::new(IgnorePath::create_filter(ignore_paths)),
        ];
        let combined_filter = create_combined_filter(filters);
        file_traversal::find(root, &combined_filter).await
    }

    /// Analyzes whether a URL, if pointing to a file, is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    #[must_use]
    pub fn is_markup_url(url: &Url) -> bool {
        url.path_segments().is_some_and(|mut path_segments| {
            path_segments.next_back().is_some_and(Self::is_markup_file)
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

impl TryFrom<PathBuf> for File<'_> {
    type Error = TypeExtractionError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let markup_type = markup::Type::try_from(path.as_path())?;
        let locator = Arc::new(FileLoc::from(path.as_path()));
        Ok(Self {
            markup_type,
            locator,
            content: Content::LocalFile(path),
            start: Position::new(),
        })
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
