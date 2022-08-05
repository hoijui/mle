// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, rc::Rc, str::FromStr};

#[cfg(test)]
use mle::extractors::find_links;
use mle::{
    anchor::Anchor,
    config::Config,
    link::{FileLoc, FileSystemLoc, Link},
    markup::{Content, File, Type},
};

fn extract(md_file: PathBuf) -> std::io::Result<(Vec<Link>, Vec<Anchor>)> {
    let locator = Rc::new(FileLoc::System(FileSystemLoc::from(md_file.clone())));
    let file = File {
        markup_type: Type::Markdown,
        content: Content::LocalFile(md_file),
        locator,
        ..Default::default()
    };
    let conf = Config::default();
    find_links(&file, &conf)
}

#[test]
fn no_links() {
    let md_file = PathBuf::from_str("./benches/benchmark/markdown/no_links/no_links.md")
        .expect("To never fail");
    let (links, _anchors) = extract(md_file).expect("No errors");
    assert!(links.is_empty());
}

#[test]
fn some_links() {
    let md_file = PathBuf::from_str("./benches/benchmark/markdown/many_links/many_links.md")
        .expect("To never fail");
    let (links, _anchors) = extract(md_file).expect("No errors");
    assert_eq!(links.len(), 11);
}
