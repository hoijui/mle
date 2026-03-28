// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod helper;

use cli_utils::StreamIdent;
use helper::benches_dir;
use mle::config::Extractor as ExtractorConfig;
use mle::config::Tool as Config;
use mle::markup;
use mle::result;
use mle::state::State;
use std::convert::TryInto;

#[tokio::test]
async fn end_to_end() {
    let markup_types = vec![markup::Type::Markdown];
    let root = benches_dir().join("benchmark");
    let ignore_paths = vec![
        "benches/benchmark/markdown/ignore_me.md"
            .try_into()
            .unwrap(),
        "./benches/benchmark/markdown/ignore_me_dir"
            .try_into()
            .unwrap(),
    ];
    let ignore_links = vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config {
        extractor: ExtractorConfig {
            markup_files,
            links: true,
            anchors: true,
            ignore_links,
        },
        links: Some(StreamIdent::StdOut),
        anchors: Some(StreamIdent::StdOut),
        result_format: result::Type::Json,
        ..Default::default()
    };
    let mut state = State::new(config);
    if let Err(err) = mle::run(&mut state).await {
        panic!("Test with custom root failed. {err:?}");
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let markup_types = vec![markup::Type::Markdown];
    let root = benches_dir().join("different_root");
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config {
        extractor: ExtractorConfig {
            markup_files,
            links: true,
            anchors: true,
            ..Default::default()
        },
        links: Some(StreamIdent::StdOut),
        anchors: Some(StreamIdent::StdOut),
        result_format: result::Type::Json,
        ..Default::default()
    };
    let mut state = State::new(config);
    if let Err(err) = mle::run(&mut state).await {
        panic!("Test with custom root failed. {err:?}");
    }
}
