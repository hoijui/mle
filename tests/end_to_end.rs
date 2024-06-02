// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod helper;

use helper::benches_dir;
use mle::config::Config;
use mle::markup::Type;
use mle::result;
use mle::state::State;
use std::convert::TryInto;

#[tokio::test]
async fn end_to_end() {
    let config = Config {
        files_and_dirs: vec![benches_dir().join("benchmark").into()],
        recursive: true,
        links: Some(None),
        anchors: Some(None),
        result_format: result::Type::Json,
        markup_types: vec![Type::Markdown],
        ignore_links: vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")],
        ignore_paths: vec![
            "benches/benchmark/markdown/ignore_me.md"
                .try_into()
                .unwrap(),
            "./benches/benchmark/markdown/ignore_me_dir"
                .try_into()
                .unwrap(),
        ],
        ..Default::default()
    };
    let mut state = State::new(config);
    if let Err(e) = mle::run(&mut state).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let config = Config {
        files_and_dirs: vec![test_files.clone().into()],
        links: Some(None),
        anchors: Some(None),
        result_format: result::Type::Json,
        markup_types: vec![Type::Markdown],
        ..Default::default()
    };
    let mut state = State::new(config);
    if let Err(e) = mle::run(&mut state).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
