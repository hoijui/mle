// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use futures::StreamExt;
use mle::ignore_path::IgnorePath;
use mle::markup::{self};
use mle::path_buf::PathBuf;
use std::ffi::OsStr;
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
    // #[error("Input path is not a file or directory: '{0:#?}'")]
    // NoFileNorDir(PathBuf),
    #[error("I/O Error: '{0:#?}'")]
    IO(#[from] std::io::Error),
}

/// Searches for markup source files according to the configuration,
/// and stores them in `result`.
///
/// # Errors
///
/// If any of the (markup) files supplied or found through scanning supplied dirs
/// has no name (e.g. '.').
/// The code-logic should prevent this from ever happening.
pub async fn scan(
    root: &Path,
    markup_types: &[markup::Type],
    ignore_paths: &[IgnorePath],
    result: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    log::debug!(
        "Searching for files of markup types '{markup_types:?}' in directory '{root:?}' ..."
    );

    let mut dir_walker = WalkDir::new(root);
    loop {
        match dir_walker.next().await {
            Some(Ok(entry)) => {
                if let Ok(file_type) = entry.file_type().await
                    && !file_type.is_dir()
                {
                    add(markup_types, ignore_paths, entry.path().as_ref(), result).await?;
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
pub async fn add(
    markup_types: &[markup::Type],
    ignore_paths: &[IgnorePath],
    file: &Path,
    result: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    // let markup_types = &config.markup_types;
    // let ignore_paths = all_ignored_paths(config);

    // let gitignored_files: Option<Vec<PathBuf>> = if config.optional.gitignore.is_some() {
    //     let files = find_git_ignored_files();
    //     debug!("Found gitignored files: {files:?}");
    //     files
    // } else {
    //     None
    // };

    // let is_gitignore_enabled = gitignored_files.is_some();

    // let gituntracked_files: Option<Vec<PathBuf>> = if config.optional.gituntracked.is_some() {
    //     let files = find_git_untracked_files();
    //     debug!("Found gituntracked files: {files:?}");
    //     files
    // } else {
    //     None
    // };

    let file_name_os_str = file
        .file_name()
        .map(OsStr::to_string_lossy)
        .ok_or_else(|| Error::MissingFileName(file.into()))?;

    let Some(_markup_type) = markup_type(file_name_os_str.as_ref(), markup_types) else {
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
        // let markup_file = File {
        //     markup_type,
        //     locator: Arc::new(FileLoc::System(FileSystemLoc::from(file))),
        //     content: Content::LocalFile(file.into()),
        //     start: Position::new(),
        // };
        // log::debug!("Found file: '{markup_file:?}'");
        // result.push(markup_file);
        log::debug!("Found file: '{file:?}'");
        result.push(file.into());
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
pub async fn find(
    root: &Path,
    markup_types: &[markup::Type],
    ignore_paths: &[IgnorePath],
) -> Result<Vec<PathBuf>, Error> {
    let mut result = vec![];
    scan(root, markup_types, ignore_paths, &mut result).await?;
    Ok(result)
}

/// Identifies the markup type a file path belongs to,
/// if any, out of a given set of markup types.
#[must_use]
fn markup_type(file: &str, markup_types: &[markup::Type]) -> Option<markup::Type> {
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
