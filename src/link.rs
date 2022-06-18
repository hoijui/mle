// use std::convert::TryFrom;
use std::ops::Add;
use std::rc::Rc;
use std::{convert::Infallible, fmt, str::FromStr};

// use async_std::path::PathBuf;
use relative_path::RelativePathBuf;
use reqwest::Url;
use std::path::{Path, PathBuf};
// use email_address::EmailAddress;

/// The source file a link was found in
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum FileSystemLoc {
    /// A relative file-system path
    Relative(RelativePathBuf),
    /// An absolute file-system path
    Absolute(PathBuf),
    // /// A (probably) remote file (has to be without anchor/fragment)
    // Url(reqwest::Url),
}

/// The location of a markup content (file)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
#[derive(PartialEq, Clone, Debug)]
pub struct Position {
    /// The line number in characters (not bytes)
    pub line: usize,
    /// The column number in characters (not bytes)
    pub column: usize,
    // /// The absolute position form the start in bytes (not characters)
    // pub pos: usize,
}

/// Where a link points to
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct FileSystemTarget {
    /// The target the link points to
    pub file: FileSystemLoc,
    /// The target the link points to
    pub anchor: Option<String>,
}

/// Where a piece of content (e.g. a link) was found;
/// including both the file and the position inside the file.
#[derive(PartialEq, Clone)]
pub struct Locator {
    pub file: Rc<FileLoc>,
    pub pos: Position,
}

/// Link found in markup files
#[derive(PartialEq, Clone)]
pub struct Link {
    /// Where the link was found
    pub source: Locator,
    /// The target the link points to
    pub target: Target,
}

impl FileLoc {
    pub fn dummy() -> Rc<Self> {
        Rc::new(Self::System(FileSystemLoc::Absolute(PathBuf::new())))
    }
}

impl Link {
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
        } else if let Ok(url) = Url::parse(&format!("file://{}", value)) {
            let path = PathBuf::from(url.path());
            let file = if path.is_absolute() {
                FileSystemLoc::Absolute(path)
            } else {
                FileSystemLoc::Relative(url.path().into())
            };
            let anchor = url.fragment().map(ToOwned::to_owned);
            Self::FileSystem(FileSystemTarget { file, anchor })
        } else {
            Self::Invalid(value.to_owned())
        }
    }
}

impl Target {
    /// Whether this target definitely points to a local resource.
    /// Note: This is **not** the same as the inversion of `::is_remote()`!
    pub fn is_local(&self) -> bool {
        match self {
            Self::Http(_) => false,
            Self::Ftp(_) => false,
            Self::EMail(_) => false,
            Self::FileUrl(_) => true,
            Self::FileSystem(_) => true,
            Self::UnknownUrlSchema(_) => false,
            Self::Invalid(_) => false,
        }
    }

    /// Whether this target most likely points to a remote resource.
    /// Note: This is **not** the same as the inversion of `::is_local()`!
    pub fn is_remote(&self) -> bool {
        match self {
            Self::Http(_) => true,
            Self::Ftp(_) => true,
            Self::EMail(_) => false,
            Self::FileUrl(_) => false,
            Self::FileSystem(_) => false,
            Self::UnknownUrlSchema(_) => true,
            Self::Invalid(_) => false,
        }
    }

    /// Whether this target is encoded as a file-system path.
    pub fn is_file_system(&self) -> bool {
        matches!(self, Self::FileSystem(_))
    }

    /// Whether this target is encoded as a URL.
    pub fn is_url(&self) -> bool {
        match self {
            Self::Http(_) => true,
            Self::Ftp(_) => true,
            Self::EMail(_) => true,
            Self::FileUrl(_) => true,
            Self::FileSystem(_) => false,
            Self::UnknownUrlSchema(_) => true,
            Self::Invalid(_) => false,
        }
    }

    /// Analyzes whether a name of a file is likely to contain
    /// content in one of our supported markup languages,
    /// (usually) judging from the file-extension.
    fn is_markup_file(file_name: &str) -> bool {
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
    pub fn is_markup_content(&self) -> bool {
        match self {
            Self::Http(url) => Self::is_markup_url(url),
            Self::Ftp(url) => Self::is_markup_url(url),
            Self::EMail(_uri) => false,
            Self::FileUrl(url) => Self::is_markup_url(url),
            Self::FileSystem(target) => target.file.is_markup(),
            Self::UnknownUrlSchema(url) => Self::is_markup_url(url),
            Self::Invalid(_str) => false,
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
            Self::Relative(rel_path) => rel_path.fmt(f)?,
            Self::Absolute(abs_path) => write!(f, "{:#?}", abs_path)?,
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
        let path = PathBuf::from_str(s).unwrap();
        Ok(
            // if let Ok(url) = reqwest::Url::parse(s) {
            //     Self::Url(url)
            // } else {
            if path.is_absolute() {
                Self::Absolute(path)
            } else {
                // Self::Relative(RelativePathBuf::from_path(path))
                Self::Relative(RelativePathBuf::from(s))
            }, // }
        )
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rrhs: Self) -> Self::Output {
        Position {
            line: self.line + rrhs.line,
            column: self.column + rrhs.column,
        }
    }
}

impl Add<&Position> for Position {
    type Output = Position;

    fn add(self, rrhs: &Self) -> Self::Output {
        Position {
            line: self.line + rrhs.line,
            column: self.column + rrhs.column,
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

// impl fmt::Debug for Position {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{}:{}",
//             self.line,
//             self.column,
//         )
//     }
// }

impl Position {
    pub fn new() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl fmt::Debug for Locator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}:{}", self.file, self.pos,)
    }
}

// /// Where a link points to
// #[derive(Hash, PartialEq, Eq, Clone, Debug)]
// pub struct Target {
//     /// The target the link points to
//     pub file: FileLoc,
//     /// The kind of link this is (HTML, Email, ...)
//     pub r#type: Type,
// }

impl std::fmt::Display for FileSystemTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.file.fmt(f)?;
        if let Some(anchor) = &self.anchor {
            f.write_fmt(format_args!("#{}", anchor))?;
        }
        Ok(())
    }
}

impl FileSystemTarget {
    /// Splitting `link` of form `"file#anchor"` into `"file"` and `Option("anchor")`.
    /// TODO tests/samples here
    pub fn split(link: &str) -> (&str, Option<String>) {
        match link.find('#') {
            Some(anchor_sep) => (
                &link[..anchor_sep],
                Some(link[(anchor_sep + 1)..].to_owned()),
            ),
            None => (link, None),
        }
    }

    // pub fn new_typed(link: &str, r#type: Type) -> Self {
    //     let (file, anchor) = Self::split(link);
    //     Self {
    //         file: FileLoc::from_str(file).unwrap(),
    //         r#type,
    //         anchor,
    //     }
    // }

    // pub fn new(link: &str) -> Self {
    //     Self::new_typed(link, get_link_type(link))
    // }
}

impl FromStr for FileSystemTarget {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (file, anchor) = Self::split(s);
        Ok(Self {
            file: FileSystemLoc::from_str(file)?,
            // r#type: get_link_type(s),
            anchor,
        })
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?} => {:#?}", self.source, self.target,)
    }
}

impl Link {
    // fn anchor_sep(&self) -> &str {
    //     match &self.anchor {
    //         Some(_) => "#",
    //         None => "",
    //     }
    // }

    // pub fn new_src(source: String, target: &str, line: usize, column: usize) -> Self {
    //     let (target, anchor) = Target::split(target);

    //     Self {
    //         source,
    //         target: target.to_string(),
    //         anchor,
    //         line,
    //         column,
    //     }
    // }

    // pub fn new(target: &str, line: usize, column: usize) -> Self {
    //     Self::new_src(String::new(), target, line, column)
    // }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MarkupAnchorType {
    /// An anchor associated to a title, auto generated from the title
    TitleAuto,
    /// An anchor associated to a title, manually defined for the title
    TitleManual,
    /// A dedicated anchor, defined as such (`<a name="..."/>` or `<a id="..."/>`)
    Direct,
    /// An anchor associated to an HTML element (e.g. a div)
    ElementId,
}

/// Anchor target found in markup files
///
/// In HTML, these look like:
/// <a name="manual-anchor">target part</a>
/// <a id="manual-anchor">target part</a>
/// <p id="manual-anchor">target part</p>
/// <div id="manual-anchor">target part</div>
/// <... id="manual-anchor">target part</...>
///
/// In Markdown - in addition to the HTML form -
/// different MD flavors support different anchors:
/// * GFM supplies auto-generated anchors for headers,
///   using the following rules:
///   1. downcase the headline
///   2. remove anything that is not a letter, number, space or hyphen
///   3. change any space to a hyphen
///   so `# My 1. @#%^$^-cool header!!` will have the anchor `my-1--cool-header`
/// * Pandoc MD supports similar (but sadly not equal) auto-generated anchors,
///   or additionally manually set anchors for headers,
///   using the following syntax:
///   `# My header {#manual-anchor}`
///
#[derive(PartialEq, Clone)]
pub struct MarkupAnchorTarget {
    /// Where the anchor was found
    pub source: Locator,
    /// The anchor name (the thing one links to)
    pub name: String,
    /// The anchor type
    pub r#type: MarkupAnchorType,
}

impl fmt::Debug for MarkupAnchorTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{} (type {:#?})", self.source, self.name, self.r#type)
    }
}

impl fmt::Display for MarkupAnchorTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.source, self.name)
    }
}