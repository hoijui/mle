// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::link::{FileLoc, FileSystemLoc, Position};
use crate::markup::{self, Content, File};
use crate::Config;
use std::ffi::OsStr;
use std::rc::Rc;

use crate::path_buf::PathBuf;
use futures::StreamExt;
use {
    async_std::{fs, path::Path},
    async_walkdir::WalkDir,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Supplied Input file is missing file name: '{0:#?}'")]
    MissingFileName(PathBuf),
    #[error("Input path does not exist: '{0:#?}'")]
    NonexistentPath(PathBuf),
    #[error("Input path is not a file or directory: '{0:#?}'")]
    NoFileNorDir(PathBuf),
    #[error("I/O Error: '{0:#?}'")]
    IO(#[from] std::io::Error),
}

// #[cfg(feature = "async")]
// fn follow_links_on<'a>(dir_walker: WalkDir) -> WalkDir {
//     dir_walker
// }
// #[cfg(not(feature = "async"))]
// fn follow_links_on<'a>(dir_walker: WalkDir) -> WalkDir {
//     dir_walker.follow_links(false)
// }

/// Searches for markup source files according to the configuration,
/// and stores them in `result`.
///
/// # Errors
///
/// If any of the (markup) files supplied or found through scanning supplied dirs
/// has no name (e.g. '.').
/// The code-logic should prevent this from ever happening.
pub async fn scan(config: &Config, root: &Path, result: &mut Vec<File<'_>>) -> Result<(), Error> {
    let markup_types = &config.markup_types;

    log::debug!(
        "Searching for files of markup types '{:?}' in directory '{:?}' ...",
        markup_types,
        root
    );

    let mut dir_walker = WalkDir::new(root);
    loop {
        match dir_walker.next().await {
            Some(Ok(entry)) => {
                if let Ok(file_type) = entry.file_type().await {
                    if !file_type.is_dir() {
                        add(config, entry.path().as_ref(), result).await?;
                    }
                }
            }
            // Some(Err(err)) => Err(err)?,
            Some(Err(_err)) => (),
            None => break,
        }
    }

    Ok(())
}

/// Stores a single file in `result`,
/// if it is accessible
/// and a markup source file according to the configuration.
///
/// # Errors
///
/// If the supplied `file` has no name (e.g. '.').
/// The code-logic should prevent this from ever being supplied.
pub async fn add(config: &Config, file: &Path, result: &mut Vec<File<'_>>) -> Result<(), Error> {
    let markup_types = &config.markup_types;
    let ignore_paths = &config.ignore_paths;

    let file_name_os_str = file
        .file_name()
        .map(OsStr::to_string_lossy)
        .ok_or_else(|| Error::MissingFileName(file.into()))?;

    let Some(markup_type) = markup_type(file_name_os_str.as_ref(), markup_types) else {
        log::trace!(
            "Not a file of a configured markup type: '{}'",
            file.display()
        );
        return Ok(());
    };

    let abs_path = fs::canonicalize(file)
        .await
        .map_err(|_err| Error::NonexistentPath(file.into()))?;
    if ignore_paths
        .iter()
        .any(|ignore_path| ignore_path.matches(abs_path.as_ref()))
    {
        log::debug!(
            "Ignoring file '{}', because it is in the ignore paths list.",
            file.display()
        );
    } else {
        let markup_file = File {
            markup_type,
            locator: Rc::new(FileLoc::System(FileSystemLoc::from(file))),
            content: Content::LocalFile(file.into()),
            start: Position::new(),
        };
        log::debug!("Found file: '{:?}'", markup_file);
        result.push(markup_file);
    }

    //     ,
    //     async |file_name: std::borrow::Cow<'_, str>| {
    //         async {
    //             async {
    //                 markup_type(file_name.as_ref(), markup_types).map_or_else(
    //                     || {
    //                         log::trace!(
    //                             "Not a file of a configured markup type: '{}'",
    //                             file.as_ref().display()
    //                         );
    //                         Ok(())
    //                     },
    //                     |markup_type| {
    //                         async {
    //                             let abs_path =
    //                                 fs::canonicalize(file.as_ref()).await.map_err(|_err| {
    //                                     Error::NonexistentPath(file.as_ref().into())
    //                                 })?;
    //                             if ignore_paths
    //                                 .iter()
    //                                 .any(|ignore_path| ignore_path.matches(abs_path.as_ref()))
    //                             {
    //                                 log::debug!(
    //                     "Ignoring file '{}', because it is in the ignore paths list.",
    //                     file.as_ref().display()
    //                 );
    //                             } else {
    //                                 let markup_file = File {
    //                                     markup_type,
    //                                     locator: Rc::new(FileLoc::System(FileSystemLoc::from(
    //                                         file.as_ref().into(),
    //                                     ))),
    //                                     content: Content::LocalFile(file.as_ref().into()),
    //                                     start: Position::new(),
    //                                 };
    //                                 log::debug!("Found file: '{:?}'", markup_file);
    //                                 result.push(markup_file);
    //                             }
    //                             Ok(())
    //                         }
    //                         .await
    //                     }.await,
    //                 )
    //             }
    //             .await
    //         }
    //         .await
    //     }.await,
    // )

    Ok(())
}

/// Searches for markup source files according to the configuration,
/// and stores them in `result`.
///
/// # Errors
///
/// If a file or path supplied does not exist,
/// or if any file supplied or found through scanning has no name (e.g. '.').
/// The code-logic should prevent the second case from ever happening.
pub async fn find(config: &Config, result: &mut Vec<File<'_>>) -> Result<(), Error> {
    for file_or_dir in &config.files_and_dirs {
        if file_or_dir.is_file().await {
            add(config, file_or_dir.as_ref(), result).await?;
        } else if file_or_dir.is_dir().await {
            scan(config, file_or_dir.as_ref(), result).await?;
        } else if !file_or_dir.exists().await {
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
