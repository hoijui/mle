extern crate walkdir;

use crate::link::{FileLoc, FileSystemLoc, Position};
use crate::markup::{Content, File, Type};
use crate::Config;
use std::fs;
use std::rc::Rc;
use std::str::FromStr;
use walkdir::WalkDir;

/// Searches for markup source files acording to the configuration,
/// and stores them in `result`.
pub fn find(config: &Config, result: &mut Vec<File>) {
    let root = &config.scan_root;
    let markup_types = &config.markup_types;
    let ignore_paths = &config.ignore_paths;

    info!(
        "Searching for files of markup types '{:?}' in directory '{:?}' ...",
        markup_types, root
    );

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = entry.file_name().to_string_lossy();

        if let Some(markup_type) = markup_type(&f_name, markup_types) {
            let path = entry.path();
            let abs_path = fs::canonicalize(path).expect("Expected path to exist.");
            if ignore_paths
                .iter()
                .any(|ignore_path| ignore_path.matches(&abs_path))
            {
                debug!(
                    "Ignoring file '{:?}', because it is in the ignore paths list.",
                    path
                );
            } else {
                let path_str = path.to_str().unwrap();
                let file = File {
                    markup_type,
                    locator: Rc::new(FileLoc::System(
                        FileSystemLoc::from_str(path_str)
                            .expect("FileSystemLoc creation from str should never fail"),
                    )),
                    content: Content::LocalFile(path_str.to_string()),
                    start: Position::new(),
                };
                debug!("Found file: '{:?}'", file);
                result.push(file);
            }
        }
    }
}

/// Identifies the markup type a file path belongs to,
/// if any, out of a given set of markup types.
#[must_use]
pub fn markup_type(file: &str, markup_types: &[Type]) -> Option<Type> {
    let file_low = file.to_lowercase();
    for markup_type in markup_types {
        let extensions = markup_type.file_extensions();
        for ext in extensions {
            let mut ext_low = String::from(".");
            ext_low.push_str(&ext);
            if file_low.ends_with(&ext_low) {
                return Some(*markup_type);
            }
        }
    }

    None
}
