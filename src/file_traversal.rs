extern crate walkdir;

use crate::markup::{MarkupFile, MarkupType};
use crate::Config;
use std::fs;
use walkdir::WalkDir;

/// Searches for markup source files acording to the configuration,
/// and stores them in `result`.
pub fn find(config: &Config, result: &mut Vec<MarkupFile>) {
    let root = &config.folder;
    let markup_types = &config.markup_types;
    let ignore_paths = &config.ignore_path;

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
            if ignore_paths.iter().any(|ignore_path| {
                if ignore_path.is_file() {
                    ignore_path == &abs_path
                } else if ignore_path.is_dir() {
                    abs_path.starts_with(ignore_path)
                } else {
                    false
                }
            }) {
                debug!(
                    "Ignoring file '{:?}', because it is in the ignore path list.",
                    path
                );
            } else {
                let file = MarkupFile {
                    markup_type,
                    path: path.to_string_lossy().to_string(),
                };
                debug!("Found file: '{:?}'", file);
                result.push(file);
            }
        }
    }
}

/// Identifies the markup type a file path belongs to,
/// if any, out of a given set of markup types.
fn markup_type(file: &str, markup_types: &[MarkupType]) -> Option<MarkupType> {
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
