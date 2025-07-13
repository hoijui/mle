// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use mle::path_buf::PathBuf;
use std::{str::FromStr, sync::Arc};

#[cfg(test)]
use mle::extractors::find_links;
use mle::{
    config::Config,
    extractors::ParseRes,
    link::{FileLoc, FileSystemLoc},
    markup::{Content, File, Type},
};

async fn extract(md_file: PathBuf) -> std::io::Result<ParseRes> {
    let locator = Arc::new(FileLoc::System(FileSystemLoc::from(md_file.clone())));
    let file = File {
        markup_type: Type::Markdown,
        content: Content::LocalFile(md_file),
        locator,
        ..Default::default()
    };
    let conf = Config::default();
    find_links(&file, &conf).await
}

#[tokio::test]
async fn no_links() {
    let md_file = PathBuf::from_str("./benches/benchmark/markdown/no_links/no_links.md")
        .expect("To never fail");
    let parsed = extract(md_file).await.expect("No errors");
    assert!(parsed.links.is_empty());
}

#[tokio::test]
async fn some_links() {
    let md_file = PathBuf::from_str("./benches/benchmark/markdown/many_links/many_links.md")
        .expect("To never fail");
    let parsed = extract(md_file).await.expect("No errors");
    assert_eq!(parsed.links.len(), 11);
}
