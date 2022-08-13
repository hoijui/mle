// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::{convert::Infallible, fmt, str::FromStr};

// use async_std::path::PathBuf;
use relative_path::RelativePathBuf;
use reqwest::Url;
use std::path::{Path, PathBuf};
// use email_address::EmailAddress;

/// The source file a link was found in
#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileSystemLoc {
    /// A relative file-system path
    Relative(RelativePathBuf),
    /// An absolute file-system path
    Absolute(PathBuf),
    // /// A (probably) remote file (has to be without anchor/fragment)
    // Url(reqwest::Url),
}

/// The location of a markup content (file)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum FileLoc {
    Url(Url),
    System(FileSystemLoc),
}

// /// The location of a markup content (file)
// #[derive(Debug, PartialEq, Eq, Hash, Clone)]
// pub struct MarkupContent<'a> {
//     text: &'a str,
//     source_file: Rc<FileLoc>,
// }

/// Where a link points to
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Http(Url),
    Ftp(Url),
    EMail(Url), // ... yeees, "mailto:..." is a valid URI!
    FileUrl(Url),
    FileSystem(FileSystemTarget),
    UnknownUrlSchema(Url),
    Invalid(String),
}

/// Where in the markup content (file/stream/string)
/// a piece of content (e.g. a link) was found.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    /// The line number in characters (not bytes)
    pub line: usize,
    /// The column number in characters (not bytes)
    pub column: usize,
    // /// The absolute position form the start in bytes (not characters)
    // pub pos: usize,
}

/// Where a link points to
#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileSystemTarget {
    /// The target the link points to
    pub file: FileSystemLoc,
    /// The target the link points to
    pub anchor: Option<String>,
}

/// Where a piece of content (e.g. a link) was found;
/// including both the file and the position inside the file.
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Locator {
    pub file: Rc<FileLoc>,
    /// Where in the `file` this locator points to
    pub pos: Position,
}

/// Link found in markup files
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Where the link was found
    pub source: Locator,
    /// The target the link points to
    pub target: Target,
}

impl Default for FileLoc {
    fn default() -> Self {
        Self::System(FileSystemLoc::Absolute(PathBuf::new()))
    }
}

impl FileLoc {
    #[must_use]
    pub fn dummy() -> Rc<Self> {
        Rc::new(Self::default())
    }
}

impl Link {
    #[must_use]
    pub fn new(file: Rc<FileLoc>, pos: Position, raw_target: &str) -> Self {
        Self {
            source: Locator { file, pos },
            target: Target::from(raw_target),
        }
    }
}

impl fmt::Display for Locator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file, self.pos)
    }
}

impl fmt::Display for FileLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(url) => url.fmt(f),
            Self::System(fs_loc) => fs_loc.fmt(f),
        }
    }
}

// impl FileSystemLoc {
//     pub fn is_url(&self) -> bool {
//         if let Self::Url(_) = self {
//             true
//         } else {
//             false
//         }
//     }
// }

impl From<&str> for Target {
    // type Error = std::io::Error; // TODO FIXME Make a TargetParseError and use it here

    fn from(value: &str) -> Self {
        if let Ok(url) = Url::parse(value) {
            Self::from(url)
        } else if let Ok(fs_target) = FileSystemTarget::from_str(value) {
            Self::FileSystem(fs_target)
        } else {
            Self::Invalid(value.to_owned())
        }
    }
}

impl From<Url> for Target {
    fn from(url: Url) -> Self {
        if url.scheme() == "http" || url.scheme() == "https" {
            Self::Http(url)
        } else if url.scheme() == "ftp" || url.scheme() == "sftp" || url.scheme() == "scp" {
            Self::Ftp(url)
        } else if url.scheme() == "mailto" {
            Self::EMail(url)
        } else if url.scheme() == "file" {
            Self::FileUrl(url)
        } else {
            Self::UnknownUrlSchema(url)
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(url)
            | Self::Ftp(url)
            | Self::EMail(url)
            | Self::FileUrl(url)
            | Self::UnknownUrlSchema(url) => write!(f, "{}", url),
            Self::FileSystem(fs_target) => write!(f, "{}", fs_target),
            Self::Invalid(msg) => write!(f, "{}", msg),
        }
    }
}

impl Target {
    /// Whether this target definitely points to a local resource.
    /// Note: This is **not** the same as the inversion of `::is_remote()`!
    #[must_use]
    pub const fn is_local(&self) -> bool {
        match self {
            Self::Http(_)
            | Self::Ftp(_)
            | Self::EMail(_)
            | Self::UnknownUrlSchema(_)
            | Self::Invalid(_) => false,
            Self::FileUrl(_) | Self::FileSystem(_) => true,
        }
    }

    /// Whether this target most likely points to a remote resource.
    /// Note: This is **not** the same as the inversion of `::is_local()`!
    #[must_use]
    pub const fn is_remote(&self) -> bool {
        match self {
            Self::Http(_) | Self::Ftp(_) | Self::UnknownUrlSchema(_) => true,
            Self::EMail(_) | Self::FileUrl(_) | Self::FileSystem(_) | Self::Invalid(_) => false,
        }
    }

    /// Whether this target is encoded as a file-system path.
    #[must_use]
    pub const fn is_file_system(&self) -> bool {
        matches!(self, Self::FileSystem(_))
    }

    /// Whether this target is encoded as a URL.
    #[must_use]
    pub const fn is_url(&self) -> bool {
        match self {
            Self::Http(_)
            | Self::Ftp(_)
            | Self::EMail(_)
            | Self::FileUrl(_)
            | Self::UnknownUrlSchema(_) => true,
            Self::FileSystem(_) | Self::Invalid(_) => false,
        }
    }

    /// Analyzes whether a name of a file is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    fn is_markup_file(_file_name: &str) -> bool {
        true // TODO FIXME Check file-extension against set of known file-extensions
    }

    /// Analyzes whether a URL, if pointing to a file, is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    fn is_markup_url(url: &Url) -> bool {
        url.path_segments().map_or(false, |path_segments| {
            path_segments.last().map_or(false, |last_path_segment| {
                Self::is_markup_file(last_path_segment)
            })
        })
    }

    /// Analyzes whether `self` is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension,
    /// if not already given by the enum item type alone.
    #[must_use]
    pub fn is_markup_content(&self) -> bool {
        match self {
            Self::Http(url) | Self::Ftp(url) | Self::FileUrl(url) | Self::UnknownUrlSchema(url) => {
                Self::is_markup_url(url)
            }
            Self::FileSystem(target) => target.file.is_markup(),
            Self::EMail(_) | Self::Invalid(_) => false,
        }
    }

    /// Removes the fragment from a link, if one is present.
    /// Otherwise it returns `self`.
    #[must_use]
    pub fn remove_anchor(&self) -> Cow<'_, Self> {
        match self {
            Self::Http(url) | Self::Ftp(url) | Self::FileUrl(url) | Self::UnknownUrlSchema(url)
                if url.fragment().is_some() =>
            {
                let mut no_frag = url.clone();
                no_frag.set_fragment(None);
                Cow::Owned(Self::from(no_frag))
            }
            Self::FileSystem(target) if (&target.anchor).is_some() => {
                let fs_target = FileSystemTarget {
                    file: target.file.clone(),
                    anchor: None,
                };
                Cow::Owned(Self::FileSystem(fs_target))
            }
            _ => Cow::Borrowed(self),
        }
    }
}

impl FileSystemLoc {
    /// Analyzes whether self is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    fn is_markup(&self) -> bool {
        match self {
            Self::Absolute(path) => path
                .file_name()
                .map(|file_name| Target::is_markup_file(format!("{:#?}", file_name).as_str())),
            Self::Relative(path) => path.file_name().map(Target::is_markup_file),
        }
        .unwrap_or(false)
    }
}

impl std::fmt::Display for FileSystemLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Relative(rel_path) => write!(f, "{}", rel_path)?,
            Self::Absolute(abs_path) => write!(f, "{}", abs_path.display())?,
            // Self::Url(url) => url.fmt(f)?,
        }
        Ok(())
    }
}

impl From<&Path> for FileSystemLoc {
    fn from(path: &Path) -> Self {
        if path.is_relative() {
            Self::Relative(RelativePathBuf::from_path(path).expect(
                "`Path.is_relative(path)` should mean `RelativePathBuf::from_path(path)` will not fail"))
        } else {
            Self::Absolute(path.to_owned())
        }
    }
}

impl From<PathBuf> for FileSystemLoc {
    fn from(path: PathBuf) -> Self {
        if path.is_relative() {
            Self::Relative(RelativePathBuf::from_path(&path).expect(
                "`PathBuf.is_relative(path)` should mean `RelativePathBuf::from_path(path)` will not fail"))
        } else {
            Self::Absolute(path)
        }
    }
}

impl FromStr for FileSystemLoc {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from_str(s)?;
        Ok(if path.is_absolute() {
            Self::Absolute(path)
        } else {
            Self::Relative(RelativePathBuf::from(s))
        })
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rrhs: Self) -> Self::Output {
        Self {
            line: self.line + rrhs.line,
            column: self.column + rrhs.column,
        }
    }
}

impl Add<&Self> for Position {
    type Output = Self;

    fn add(self, rrhs: &Self) -> Self::Output {
        Self {
            line: self.line + rrhs.line,
            column: self.column + rrhs.column,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rrhs: Self) -> Self::Output {
        Self {
            line: self.line - rrhs.line,
            column: self.column - rrhs.column,
        }
    }
}

impl Sub<&Self> for Position {
    type Output = Self;

    fn sub(self, rrhs: &Self) -> Self::Output {
        Self {
            line: self.line - rrhs.line,
            column: self.column - rrhs.column,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column,)
    }
}

impl Position {
    #[must_use]
    pub const fn new() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl fmt::Debug for Locator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}:{}", self.file, self.pos,)
    }
}

impl std::fmt::Display for FileSystemTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.file))?;
        if let Some(anchor) = &self.anchor {
            f.write_fmt(format_args!("#{}", anchor))?;
        }
        Ok(())
    }
}

impl FileSystemTarget {
    /// Splitting `link` of form `"file#anchor"` into `"file"` and `Option("anchor")`.
    /// TODO tests/samples here
    #[must_use]
    pub fn split(link: &str) -> (&str, Option<String>) {
        match link.find('#') {
            Some(anchor_sep) => (
                &link[..anchor_sep],
                Some(link[(anchor_sep + 1)..].to_owned()),
            ),
            None => (link, None),
        }
    }
}

impl FromStr for FileSystemTarget {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (file, anchor) = Self::split(s);
        Ok(Self {
            file: FileSystemLoc::from_str(file)?,
            anchor,
        })
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?} => {:#?}", self.source, self.target,)
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.source, self.target,)
    }
}
