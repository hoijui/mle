
/// Different categories of paths:
///
/// | Path cathegory name | starts with FS root | may not include operators | has symbolic links resolved |
/// | ---                 | ---                 | ---                       | ---                         |
/// | Relative            | [ ]                 | [ ]                       | [ ]                         |
/// | Absolute            | [x]                 | [ ]                       | [ ]                         |
/// | Logical             | [x]                 | [x]                       | [ ]                         |
/// | Canonical           | [x]                 | [x]                       | [x]                         |
///
/// (path-)operators: '.' and '..'
/// file-system root: e.g. '/' on Linux and 'C:\' or 'F:/' on Windows

use std::borrow::Borrow;
use std::borrow::Cow;
use std::env;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::link::FileSystemLoc;
use crate::link::FileTarget;
use crate::link::MarkupAnchorTarget;
use crate::link::Link;
use crate::Config;
use crate::RemoteCache;
use crate::State;
use crate::link::Target;
use colored::ColoredString;
use colored::Colorize;
use relative_path::RelativePath;
use reqwest::Url;
use tokio::sync::Mutex;
use std::path::MAIN_SEPARATOR;
use walkdir::WalkDir;

pub async fn get_canonical(
    orig_link: &Link,
    config: &Config,
) -> Option<Target> {
    let canonicalized = to_canonical(orig_link, config);
    if canonicalized == orig_link.target {
        None
    } else {
        Some(canonicalized)
    }
}

/// Converts any valid file path, pointing to an existing,
/// local file-system entry, into its canonical, absolute form.
pub fn to_canonical(orig_link: &Link, config: &Config) -> std::io::Result<Target> {

    // We canonicalize Absolute and Relative file paths, and "file://" URLs
    // the rest we assume, is already canonicalized, or has no way of (un-)canonicalization 
    match orig_link.target {
        Target::FileUrl(url) => {},
        Target::FileSystem(file_target) => {
            Ok(Target::FileSystem(FileTarget {
                file: FileSystemLoc::Absolute(to_canonical_target_file(
                    &orig_link.source.file, // TODO XXX Oh No! This might also *not* be absolute/canonicalized yet, and even worse.. while it currently is a file-system path by its type, it really... could also be a URL! 
                    &orig_link.target.file,
                    config,
                )),
                anchor: None,
            }))
        },
        _ => { Ok(orig_link.target) },
    }
}

fn normalize(orig: &Path) -> std::io::Result<PathBuf> { // TODO
    let mut absolute = if orig.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    for component in orig.components() {
        match component {
            Component::CurDir => {},
            Component::ParentDir => { absolute.pop(); },
            component @ _ => absolute.push(component.as_os_str()),
        }
    }
    Ok(absolute)
}

fn to_lexical_absolute(orig: &Path) -> std::io::Result<PathBuf> { // TODO
    let mut absolute = if orig.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    for component in orig.components() {
        match component {
            Component::CurDir => {},
            Component::ParentDir => { absolute.pop(); },
            component @ _ => absolute.push(component.as_os_str()),
        }
    }
    Ok(absolute)
}

/// Converts a file-system location to an absolute form.
/// The result is guaranteed to be an absolute path,
/// but **not** to be canonical or normalized.
pub fn to_absolute_target<'t>(cwd: &Path, source: &FileSystemLoc, target: &'t FileSystemLoc) -> std::io::Result<Cow<'t, Path>> {

    let abs_target: Cow<Path> = match (source, target) {
        (_, FileSystemLoc::Absolute(abs_target)) => {
            Cow::Borrowed(abs_target)
        },
        (FileSystemLoc::Absolute(abs_source), FileSystemLoc::Relative(rel_target)) => {
            Cow::Owned(rel_target.to_path(abs_source))
        },
        (FileSystemLoc::Relative(rel_source), FileSystemLoc::Relative(rel_target)) => {
            // pwd + rel_source + rel_target
            Cow::Owned(rel_target.to_path(rel_source.to_path(env::current_dir()?)))
        },
    };

    Ok(abs_target)
}

pub async fn normalize_path(source: &FileSystemLoc, target: &FileSystemLoc) -> PathBuf {
    let /*mut*/ normalized_link = orig_link.target.file.to_string()
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string());
    // if let Some(idx) = normalized_link.find('#') {
    //     warn!(
    //         "Strip everything after #. The chapter (aka anchor aka fragment) part '{}' is not checked.",
    //         &normalized_link[idx..]
    //     );
    //     normalized_link = normalized_link[..idx].to_string();
    // }
    let mut fs_link_target = Path::new(&normalized_link).to_path_buf();
    if normalized_link.starts_with(MAIN_SEPARATOR) && config.root_dir.is_some() {
        match canonicalize(&config.root_dir.as_ref().unwrap()).await {
            Ok(new_root) => fs_link_target = new_root.join(Path::new(&normalized_link[1..])),
            Err(e) => panic!(
                "Root path could not be converted to an absolute path. Does the directory exit? '{}'",
                e
            ),
        }
    }

    debug!("Checking file system link target '{:?}' ...", link.target);
    let abs_path = absolute_target_path(&link.source.file, &fs_link_target)
        .await
        .to_str()
        .unwrap_or_else(|| panic!("Could not resolve target path '{}'", link.target))
        .to_string();
    // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
    Target::from_str(abs_path
        .strip_prefix(r#"\\?\"#)
        .unwrap_or(&abs_path)).unwrap()
}

async fn absolute_target_path(source: &FileSystemLoc, target: &Path) -> PathBuf {
    lazy_static! {
        static ref ROOT: PathBuf = PathBuf::from(&format!("{}", MAIN_SEPARATOR));
    }
    if target.is_relative() {
        let abs_source = canonicalize(source)
            .await
            .unwrap_or_else(|_| panic!("Path '{}' does not exist.", source));
        let parent = abs_source.parent().unwrap_or(&ROOT);
        let new_target = match target.strip_prefix(format!(".{}", MAIN_SEPARATOR)) {
            Ok(t) => t,
            Err(_) => target,
        };
        parent.join(new_target)
    } else {
        target.clone()
    }
}

use url::Url;
use regex::Regex;

#[must_use]
pub fn get_link_type(link: &str) -> Target {
    lazy_static! {
        static ref FILE_SYSTEM_REGEX: Regex =
            Regex::new(r"^(([[:alpha:]]:(\\|/))|(..?(\\|/))|((\\\\?|//?))).*").unwrap();
    }

    if FILE_SYSTEM_REGEX.is_match(link) || !link.contains(':') {
        return if link.contains('@') {
            Type::Mail
        } else {
            Type::FileSystem
        };
    }

    if let Ok(url) = Url::parse(link) {
        let scheme = url.scheme();
        debug!("Link '{}' is a URL type with scheme {}", link, scheme);
        return match scheme {
            "http" | "https" => Type::Http,
            "ftp" | "ftps" => Type::Ftp,
            "mailto" => Type::Mail,
            "file" => Type::FileSystem,
            _ => Type::UnknownUrlSchema,
        };
    }
    Type::UnknownUrlSchema
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    fn test_link(link: &str, expected_type: &Type) {
        let link_type = get_link_type(link);
        assert_eq!(link_type, *expected_type);
    }

    #[test_case("https://doc.rust-lang.org.html")]
    #[test_case("http://www.website.php")]
    fn http_link_types(link: &str) {
        test_link(link, &Type::Http);
    }

    #[test_case("ftp://mueller:12345@ftp.downloading.ch")]
    fn ftp_link_types(ftp: &str) {
        test_link(ftp, &Type::Ftp);
    }

    #[test_case("F:/fake/windows/paths")]
    #[test_case("\\\\smb}\\paths")]
    #[test_case("C:\\traditional\\paths")]
    #[test_case("\\file.ext")]
    #[test_case("file:///some/path/")]
    #[test_case("path")]
    #[test_case("./file.ext")]
    #[test_case(".\\file.md")]
    #[test_case("../upper_dir.md")]
    #[test_case("..\\upper_dir.mdc")]
    #[test_case("D:\\Program Files(x86)\\file.log")]
    #[test_case("D:\\Program Files(x86)\\folder\\file.log")]
    fn test_file_system_link_types(link: &str) {
        test_link(link, &Type::FileSystem);
    }

    #[tokio::test]
    async fn remove_dot() {
        let source = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark");
        let target = Path::new("./script_and_comments.md").to_path_buf();

        let path = absolute_target_path(&source.to_str().unwrap(), &target).await;

        let path_str = path.to_str().unwrap().to_string();
        println!("{:?}", path_str);
        assert_eq!(path_str.matches('.').count(), 1);
    }
}
