// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate walkdir;

use crate::link::{FileLoc, FileSystemLoc, Position};
use crate::markup::{self, Content, File};
use crate::Config;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Supplied Input file is missing file name: '{0}'")]
    MissingFileName(PathBuf),
    #[error("Input path does not exist: '{0}'")]
    NonexistentPath(PathBuf),
    #[error("Input path is not a file or directory: '{0}'")]
    NoFileNorDir(PathBuf),
}

/// Searches for markup source files acording to the configuration,
/// and stores them in `result`.
///
/// # Errors
///
/// If any of the (markup) files supplied or found through scanning supplied dirs
/// has no name (e.g. '.').
/// The code-logic should prevent this from ever happening.
pub fn scan(config: &Config, root: &Path, result: &mut Vec<File>) -> Result<(), Error> {
    let markup_types = &config.markup_types;

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
        add(config, entry.path(), result)?;
    }

    Ok(())
}

/// Stores a single file in `result`,
/// if it is accessible
/// and a markup source file acording to the configuration.
///
/// # Errors
///
/// If the supplied `file` has no name (e.g. '.').
/// The code-logic should prevent this from ever happening.
pub fn add(config: &Config, file: &Path, result: &mut Vec<File>) -> Result<(), Error> {
    let markup_types = &config.markup_types;
    let ignore_paths = &config.ignore_paths;

    if let Some(file_name) = file.file_name().map(OsStr::to_string_lossy) {
        if let Some(markup_type) = markup_type(&file_name, markup_types) {
            let abs_path = fs::canonicalize(file).expect("Expected path to exist.");
            if ignore_paths
                .iter()
                .any(|ignore_path| ignore_path.matches(&abs_path))
            {
                debug!(
                    "Ignoring file '{}', because it is in the ignore paths list.",
                    file.display()
                );
            } else {
                let markup_file = File {
                    markup_type,
                    locator: Rc::new(FileLoc::System(FileSystemLoc::from(file))),
                    content: Content::LocalFile(file.to_owned()),
                    start: Position::new(),
                };
                debug!("Found file: '{:?}'", markup_file);
                result.push(markup_file);
            }
        } else {
            trace!(
                "Not a file of a configured markup type: '{}'",
                file.display()
            );
        }
    } else {
        return Err(Error::MissingFileName(file.to_path_buf()));
    }
    Ok(())
}

/// Searches for markup source files acording to the configuration,
/// and stores them in `result`.
///
/// # Errors
///
/// If a file or path supplied does not exist,
/// or if any file supplied or found through scannig has no name (e.g. '.').
/// The code-logic should prevent the second case from ever happening.
pub fn find(config: &Config, result: &mut Vec<File>) -> Result<(), Error> {
    for file_or_dir in &config.files_and_dirs {
        if file_or_dir.is_file() {
            add(config, file_or_dir, result)?;
        } else if file_or_dir.is_dir() {
            scan(config, file_or_dir, result)?;
        } else if !file_or_dir.exists() {
            return Err(Error::NonexistentPath(file_or_dir.clone()));
        } else {
            return Err(Error::MissingFileName(file_or_dir.clone()));
        }
    }
    Ok(())
}

/// Identifies the markup type a file path belongs to,
/// if any, out of a given set of markup types.
#[must_use]
pub fn markup_type(file: &str, markup_types: &[markup::Type]) -> Option<markup::Type> {
    let file_low = file.to_lowercase();
    for markup_type in markup_types {
        let extensions = markup_type.file_extensions();
        for ext in extensions {
            let mut ext_low = String::from(".");
            ext_low.push_str(ext);
            if file_low.ends_with(&ext_low) {
                return Some(*markup_type);
            }
        }
    }

    None
}
