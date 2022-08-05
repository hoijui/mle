// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    /// Ignore path '{:?}' not found: {:?}.
    FailedToCanonicalize(PathBuf, std::io::Error),
    /// Ignore path '{path:?}' is neither a dir nor a regular file; Do not know how to use it.
    UnknownPathType(PathBuf),
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    /// Matches the whole path, so basically a full, canocnial, absolute path to a file
    Whole,
    /// Matches only a prefix of the path.
    Prefix,
}

#[derive(Debug, Clone)]
pub struct IgnorePath {
    pub r#type: Type,
    pub path: PathBuf,
}

impl IgnorePath {
    #[must_use]
    pub fn matches(&self, abs_path: &Path) -> bool {
        match self.r#type {
            Type::Whole => self.path == abs_path,
            Type::Prefix => abs_path.starts_with(&self.path),
        }
    }
}

impl TryFrom<&Path> for IgnorePath {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let can_path =
            fs::canonicalize(path).map_err(|err| Error::FailedToCanonicalize(path.into(), err))?;
        let r#type = if can_path.is_file() {
            Type::Whole
        } else if can_path.is_dir() {
            Type::Prefix
        } else {
            return Err(Error::UnknownPathType(can_path));
        };
        Ok(Self {
            r#type,
            path: can_path,
        })
    }
}

impl TryFrom<&str> for IgnorePath {
    type Error = Error;

    fn try_from(path_str: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(path_str))
    }
}

/// Parses the argument into an [`IgnorePath`].
///
/// # Errors
///
/// If the argument is not a valid path glob.
pub fn parse(path_str: &str) -> Result<IgnorePath, String> {
    IgnorePath::try_from(path_str).map_err(|err| format!("{:?}", err))
}

/// Checks if the argument is a valid ignore path (=> path glob).
///
/// # Errors
/// If the argument is not a valid path glob.
// pub fn is_valid(path_str: &str) -> Result<(), String> {
pub fn is_valid<S: AsRef<str>>(path_str: S) -> Result<(), String> {
    parse(path_str.as_ref()).map(|_| ())
}
