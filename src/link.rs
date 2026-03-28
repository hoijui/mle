// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use cli_utils::BoxResult;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::{Add, Sub};
use std::sync::Arc;
use std::{convert::Infallible, fmt, str::FromStr};

use relative_path::RelativePathBuf;
use url::Url;

use async_std::path::Path;
use cli_utils::path_buf::PathBuf;

use crate::markup;

/// The source file a link was found in
#[derive(Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileSystemLoc {
    /// A relative file-system path
    Relative(RelativePathBuf),
    /// An absolute file-system path
    Absolute(PathBuf),
    // /// A (probably) remote file (has to be without anchor/fragment)
    // Url(url::Url),
}

/// The location of a markup content (file)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum FileLoc {
    Url(Url),
    System(FileSystemLoc),
}

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
    pub file: Arc<FileLoc>,
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
        Self::System(FileSystemLoc::Absolute(PathBuf::new())) // FIXME Bad! that is not an absolute path
    }
}

impl FileLoc {
    #[must_use]
    pub fn dummy() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Parse a string as an URL, with this URL as the base URL.
    ///
    /// The inverse of this is [`make_relative`].
    ///
    /// # Notes
    ///
    /// - A trailing slash is significant.
    ///   Without it, the last path component is considered to be a “file” name
    ///   to be removed to get at the “directory” that is used as the base.
    /// - A [scheme relative special URL](https://url.spec.whatwg.org/#scheme-relative-special-url-string)
    ///   as input replaces everything in the base URL after the scheme.
    /// - An absolute URL (with a scheme) as input replaces the whole base URL (even the scheme).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url::Url;
    /// # use url::ParseError;
    ///
    /// // Base without a trailing slash
    /// # fn run() -> Result<(), ParseError> {
    /// let base = Url::parse("https://example.net/a/b.html")?;
    /// let url = base.join("c.png")?;
    /// assert_eq!(url.as_str(), "https://example.net/a/c.png");  // Not /a/b.html/c.png
    ///
    /// // Base with a trailing slash
    /// let base = Url::parse("https://example.net/a/b/")?;
    /// let url = base.join("c.png")?;
    /// assert_eq!(url.as_str(), "https://example.net/a/b/c.png");
    ///
    /// // Input as scheme relative special URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("//eve.com/b")?;
    /// assert_eq!(url.as_str(), "https://eve.com/b");
    ///
    /// // Input as base url relative special URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("/v1/meta")?;
    /// assert_eq!(url.as_str(), "https://alice.com/v1/meta");
    ///
    /// // Input as absolute URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("http://eve.com/b")?;
    /// assert_eq!(url.as_str(), "http://eve.com/b");  // http instead of https
    ///
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// If the function can not parse an URL from the given string
    /// with this URL as the base URL, a [`ParseError`] variant will be returned.
    ///
    /// [`ParseError`]: enum.ParseError.html
    /// [`make_relative`]: #method.make_relative
    #[inline]
    pub fn join(&self, relative_path: &str) -> BoxResult<Self> {
        Ok(match self {
            Self::Url(base_url) => Self::Url(base_url.join(relative_path)?),
            Self::System(base_path) => Self::System(base_path.join(relative_path)?),
        })
    }

    /// Returns the `Path` without its final component, if there is one.
    ///
    /// Returns [`None`] if the path terminates in a root or prefix.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    ///
    /// # Panics
    ///
    /// If there is les then one source file path part.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        Some(match self {
            Self::Url(url) => {
                let mut parent_url = url.clone();
                parent_url.set_path(
                    &PathBuf::from(url.path())
                        .parent()
                        .expect("There always has to be at least one source file path part")
                        .to_string_lossy(),
                );
                Self::Url(parent_url)
            }
            Self::System(file_system_loc) => Self::System(
                file_system_loc
                    .parent()
                    .expect("There always has to be at least one source file path part"),
            ),
        })
    }

    /// Makes relative paths absolute and resolves `../` and `./` relative parts.
    /// This is useful, for example when trying to group all `Target`s
    /// that point to the same resource/file.
    ///
    /// # Errors
    ///
    /// - never
    pub fn canonical(self: Arc<Self>, base: &PathBuf) -> BoxResult<Arc<Self>> {
        Ok(
            if let Self::System(FileSystemLoc::Relative(rel_source_path)) = &self.as_ref() {
                Arc::new(Self::System(FileSystemLoc::Absolute(
                    base.join(rel_source_path.as_str()),
                )))
            } else {
                self
            },
        )
    }
}

impl Link {
    #[must_use]
    pub fn new(file: Arc<FileLoc>, pos: Position, raw_target: &str) -> Self {
        Self {
            source: Locator { file, pos },
            target: Target::from(raw_target),
        }
    }

    #[must_use]
    pub const fn is_invalid(&self) -> bool {
        matches!(self.target, Target::Invalid(..))
    }
}

impl fmt::Display for Locator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file, self.pos)
    }
}

impl From<&Path> for FileLoc {
    fn from(path: &Path) -> Self {
        Self::System(FileSystemLoc::from(path))
    }
}

impl From<Url> for FileLoc {
    fn from(url: Url) -> Self {
        Self::Url(url)
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
        Url::parse(value).map_or_else(
            |_| {
                FileSystemTarget::from_str(value)
                    .map_or_else(|_| Self::Invalid(value.to_owned()), Self::FileSystem)
            },
            Self::from,
        )
    }
}

impl From<Url> for Target {
    fn from(url: Url) -> Self {
        if ["http", "https"].contains(&url.scheme()) {
            Self::Http(url)
        } else if ["ftp", "sftp", "scp"].contains(&url.scheme()) {
            Self::Ftp(url)
        } else if ["mailto"].contains(&url.scheme()) {
            Self::EMail(url)
        } else if ["file"].contains(&url.scheme()) {
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
            | Self::UnknownUrlSchema(url) => write!(f, "{url}"),
            Self::FileSystem(fs_target) => write!(f, "{fs_target}"),
            Self::Invalid(msg) => write!(f, "{msg}"),
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

    /// Analyzes whether `self` is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension,
    /// if not already given by the enum item type alone.
    #[must_use]
    pub fn is_markup_content(&self) -> bool {
        match self {
            Self::Http(url) | Self::Ftp(url) | Self::FileUrl(url) | Self::UnknownUrlSchema(url) => {
                markup::Type::is_markup_url(url)
            }
            Self::FileSystem(target) => target.file.is_markup(),
            Self::EMail(_) | Self::Invalid(_) => false,
        }
    }

    /// Makes relative paths absolute and resolves `../` and `./` relative parts.
    /// This is useful, for example when trying to group all `Target`s
    /// that point to the same resource/file.
    ///
    /// # Panics
    ///
    /// - Failed to extract the FS rot from an absolute path
    /// - Failed to strip away the FS root from path
    ///   from of which it was previously extracted
    ///
    /// # Errors
    ///
    /// - If canonicalization of a relative path fails
    /// - If extracting the parent from that path fails
    pub fn canonical(
        &self,
        re_root_abs_paths: bool,
        source_file: Arc<FileLoc>,
        rel_path_base: &PathBuf, /*base: &FileLoc*/
    ) -> BoxResult<Cow<'_, Self>> {
        // eprintln!(
        //     "\n\ncanonical({self}, {re_root_abs_paths}, '{source_file}', '{rel_path_base}') ..."
        // );
        if let Self::FileSystem(fs_target) = self {
            match &fs_target.file {
                FileSystemLoc::Absolute(orig_abs_path) => {
                    if re_root_abs_paths {
                        // We need to remove the FS root from the absolute path first,
                        // in order to be able to re-root it
                        // (meaning: to replace the FS root with a directory).
                        // This way, the path is treated as relative by the `join` function,
                        // and appended to the new root,
                        // instead of leaving it as is, because it is already absolute.
                        let root = orig_abs_path
                            .iter()
                            .next()
                            .expect("Absolute path needs to have at least a (first) root part");
                        let relativized_abs_path = orig_abs_path.strip_prefix(root).expect(
                            "To be able to strip root from path of which it was extracted from",
                        );
                        // eprintln!(
                        //     "\t1 - {rel_path_base} * {orig_abs_path} -> {}",
                        //     relativized_abs_path.display()
                        // );
                        return Ok(Cow::Owned(Self::FileSystem(FileSystemTarget {
                            file: FileSystemLoc::Absolute(rel_path_base.join(relativized_abs_path)),
                            anchor: fs_target.anchor.clone(),
                        })));
                    }
                }
                FileSystemLoc::Relative(relative_path) => {
                    log::debug!(
                        "Target::canonical - FileSystemLoc::Relative - relative_path: '{relative_path}'"
                    );
                    log::debug!(
                        "Target::canonical - FileSystemLoc::Relative - source_file: '{source_file}'"
                    );
                    log::debug!(
                        "Target::canonical - FileSystemLoc::Relative - rel_path_base: '{rel_path_base}'"
                    );
                    let base = source_file.canonical(rel_path_base)?;
                    log::debug!("Target::canonical - FileSystemLoc::Relative - base 0: '{base}'");
                    let base = base
                        .parent()
                        .ok_or_else(|| format!("link source-file has no parent: '{base}'"))?;
                    log::debug!("Target::canonical - FileSystemLoc::Relative - base 1: '{base}'");
                    match base.join(relative_path.as_str())? {
                        FileLoc::Url(abs_url) => {
                            let mut abs_url = Self::from(abs_url);
                            abs_url.set_fragment(fs_target.anchor.clone());
                            // eprintln!("\t2 - {abs_url}");
                            log::debug!(
                                "Target::canonical - FileSystemLoc::Relative - FileLoc::Url - abs_url: '{abs_url}'"
                            );
                            return Ok(Cow::Owned(abs_url));
                        }
                        FileLoc::System(abs_path) => {
                            // eprintln!("\t3 - {abs_path}");
                            log::debug!(
                                "Target::canonical - FileSystemLoc::Relative - FileLoc::System - abs_url: '{abs_path}'"
                            );
                            return Ok(Cow::Owned(Self::FileSystem(FileSystemTarget {
                                file: abs_path,
                                anchor: fs_target.anchor.clone(),
                            })));
                        }
                    }
                }
            }
        }
        Ok(Cow::Borrowed(self))
    }

    /// Removes the fragment from a link, if one is present.
    /// Otherwise it returns `self`.
    #[must_use]
    pub fn without_fragment(&self) -> Cow<'_, Self> {
        match self {
            Self::Http(url) | Self::Ftp(url) | Self::FileUrl(url) | Self::UnknownUrlSchema(url)
                if url.fragment().is_some() =>
            {
                let mut no_frag = url.clone();
                no_frag.set_fragment(None);
                Cow::Owned(Self::from(no_frag))
            }
            Self::FileSystem(target) if target.anchor.is_some() => {
                let fs_target = FileSystemTarget {
                    file: target.file.clone(),
                    anchor: None,
                };
                Cow::Owned(Self::FileSystem(fs_target))
            }
            _ => Cow::Borrowed(self),
        }
    }

    /// Removes the fragment from a link, if one is present.
    /// Otherwise it returns `self`.
    pub fn set_fragment(&mut self, fragment: Option<String>) {
        match self {
            Self::Http(url)
            | Self::EMail(url)
            | Self::Ftp(url)
            | Self::FileUrl(url)
            | Self::UnknownUrlSchema(url) => {
                url.set_fragment(fragment.as_deref());
            }
            Self::FileSystem(target) => {
                target.anchor = fragment;
            }
            Self::Invalid(_) => (),
        }
    }

    /// Removes the fragment from a link, if one is present.
    /// Otherwise it returns `self`.
    #[must_use]
    pub fn fragment(&self) -> Option<&'_ str> {
        match self {
            Self::Http(url) | Self::Ftp(url) | Self::FileUrl(url) | Self::UnknownUrlSchema(url) => {
                url.fragment()
            }
            Self::FileSystem(target) => target.anchor.as_deref(),
            Self::EMail(_) | Self::Invalid(_) => None,
        }
    }
}

impl FileLoc {
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Url(url) => url.scheme() == "file",
            Self::System(_) => true,
        }
    }

    #[must_use]
    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }

    /// Whether this target is encoded as a file-system path.
    #[must_use]
    pub const fn is_file_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

    /// Whether this target is encoded as a URL.
    #[must_use]
    pub const fn is_url(&self) -> bool {
        !self.is_file_system()
    }
}

impl FileSystemLoc {
    /// Returns a raw string representation of the underlying path primitive.
    #[must_use]
    pub fn get_raw(&self) -> Cow<'_, str> {
        match self {
            Self::Absolute(path) => path.as_os_str().to_string_lossy(),
            Self::Relative(path) => path.as_str().into(),
        }
    }

    /// Analyzes whether self is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    fn is_markup(&self) -> bool {
        match self {
            Self::Absolute(path) => path.file_name().map(|file_name| {
                markup::Type::is_markup_file(format!("{}", file_name.display()).as_str())
            }),
            Self::Relative(path) => path.file_name().map(markup::Type::is_markup_file),
        }
        .unwrap_or(false)
    }

    /// Returns or constructs the absolute version of this location.
    #[must_use]
    pub fn to_absolute(&self, base: &Path) -> Cow<'_, Self> {
        match self {
            Self::Absolute(_path) => Cow::Borrowed(self),
            Self::Relative(path) => Cow::Owned(Self::Absolute(path.to_path(base).into())),
        }
    }

    /// Returns or constructs the absolute version of this location.
    ///
    /// # Panics
    ///
    /// If `to_absolute(&self, base: &Path)` returned `Self::Relative`.
    #[must_use]
    pub fn to_absolute_path(&self, base: &Path) -> Cow<'_, PathBuf> {
        match self.to_absolute(base).into_owned() {
            Self::Absolute(path) => Cow::Owned(path),
            Self::Relative(_) => {
                panic!("FileSystemLoc::to_absolute(base) returned a Self::Relative -> BAD!")
            }
        }
    }

    /// Parse a string as an URL, with this URL as the base URL.
    ///
    /// The inverse of this is [`make_relative`].
    ///
    /// # Notes
    ///
    /// - A trailing slash is significant.
    ///   Without it, the last path component is considered to be a “file” name
    ///   to be removed to get at the “directory” that is used as the base.
    /// - A [scheme relative special URL](https://url.spec.whatwg.org/#scheme-relative-special-url-string)
    ///   as input replaces everything in the base URL after the scheme.
    /// - An absolute URL (with a scheme) as input replaces the whole base URL (even the scheme).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url::Url;
    /// # use url::ParseError;
    ///
    /// // Base without a trailing slash
    /// # fn run() -> Result<(), ParseError> {
    /// let base = Url::parse("https://example.net/a/b.html")?;
    /// let url = base.join("c.png")?;
    /// assert_eq!(url.as_str(), "https://example.net/a/c.png");  // Not /a/b.html/c.png
    ///
    /// // Base with a trailing slash
    /// let base = Url::parse("https://example.net/a/b/")?;
    /// let url = base.join("c.png")?;
    /// assert_eq!(url.as_str(), "https://example.net/a/b/c.png");
    ///
    /// // Input as scheme relative special URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("//eve.com/b")?;
    /// assert_eq!(url.as_str(), "https://eve.com/b");
    ///
    /// // Input as base url relative special URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("/v1/meta")?;
    /// assert_eq!(url.as_str(), "https://alice.com/v1/meta");
    ///
    /// // Input as absolute URL
    /// let base = Url::parse("https://alice.com/a")?;
    /// let url = base.join("http://eve.com/b")?;
    /// assert_eq!(url.as_str(), "http://eve.com/b");  // http instead of https
    ///
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// If the function can not parse an URL from the given string
    /// with this URL as the base URL, a [`ParseError`] variant will be returned.
    ///
    /// [`ParseError`]: enum.ParseError.html
    /// [`make_relative`]: #method.make_relative
    #[inline]
    pub fn join(&self, relative_path: &str) -> BoxResult<Self> {
        Ok(match self {
            Self::Relative(rel_base_path) => Self::Relative(rel_base_path.join(relative_path)),
            Self::Absolute(abs_base_path) => Self::Absolute(abs_base_path.join(relative_path)),
        })
    }

    /// Returns the `Path` without its final component, if there is one.
    ///
    /// Returns [`None`] if the path terminates in a root or prefix.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::Relative(path) => path.parent().map(ToOwned::to_owned).map(Self::Relative),
            Self::Absolute(path) => path.parent().map(Into::into).map(Self::Absolute),
        }
    }
}

impl std::fmt::Display for FileSystemLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Relative(rel_path) => write!(f, "{rel_path}")?,
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
            Self::Absolute(path.to_owned().into())
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

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line + rhs.line,
            column: self.column + rhs.column,
        }
    }
}

impl Add<&Self> for Position {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Self {
            line: self.line + rhs.line,
            column: self.column + rhs.column,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line - rhs.line,
            column: self.column - rhs.column,
        }
    }
}

impl Sub<&Self> for Position {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        Self {
            line: self.line - rhs.line,
            column: self.column - rhs.column,
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
        write!(f, "{}:{}", self.line, self.column)
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
        write!(f, "{:#?}:{}", self.file, self.pos)
    }
}

impl std::fmt::Display for FileSystemTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.file))?;
        if let Some(anchor) = &self.anchor {
            f.write_fmt(format_args!("#{anchor}"))?;
        }
        Ok(())
    }
}

impl FileSystemTarget {
    /// Splitting `link` of form `"file#anchor"` into `"file"` and `Option("anchor")`.
    /// TODO tests/samples here
    #[must_use]
    pub fn split(link: &str) -> (&str, Option<String>) {
        link.find('#').map_or((link, None), |anchor_sep| {
            (
                &link[..anchor_sep],
                Some(link[(anchor_sep + 1)..].to_owned()),
            )
        })
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
        write!(f, "{:#?}:{:#?}", self.source, self.target)
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.source, self.target)
    }
}
