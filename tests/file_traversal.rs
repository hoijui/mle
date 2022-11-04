// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use mle::config::Config;
#[cfg(test)]
use mle::file_traversal;
use mle::markup::{File, Type};
use std::path::Path;

#[test]
fn find_markdown_files() {
    let path = Path::new("./benches/benchmark/markdown/md_file_endings").to_path_buf();
    let config: Config = Config {
        files_and_dirs: vec![path],
        markup_types: vec![Type::Markdown],
        ..Default::default()
    };
    let mut result: Vec<File> = Vec::new();

    file_traversal::find(&config, &mut result).unwrap();
    assert_eq!(result.len(), 12);
}

#[test]
fn empty_folder() {
    let path = Path::new("./benches/benchmark/markdown/empty").to_path_buf();
    let config: Config = Config {
        files_and_dirs: vec![path],
        markup_types: vec![Type::Markdown],
        ..Default::default()
    };
    let mut result: Vec<File> = Vec::new();

    file_traversal::find(&config, &mut result).unwrap();
    assert!(result.is_empty());
}
