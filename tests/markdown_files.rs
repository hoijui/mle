use std::{rc::Rc, str::FromStr};

#[cfg(test)]
use mle::link_extractors::link_extractor::find_links;
use mle::{
    link::{FileLoc, FileSystemLoc},
    markup::{Content, MarkupFile, MarkupType},
};

#[test]
fn no_links() {
    let locator = Rc::new(FileLoc::System(
        FileSystemLoc::from_str("./benches/benchmark/markdown/no_links/no_links.md")
            .expect("To never fail"),
    ));
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        content: Content::LocalFile(locator.to_string()),
        locator,
        ..Default::default()
    };
    let (links, _anchors) = find_links(&file, false).expect("No errors");
    assert!(links.is_empty());
}

#[test]
fn some_links() {
    let locator = Rc::new(FileLoc::System(
        FileSystemLoc::from_str("./benches/benchmark/markdown/many_links/many_links.md")
            .expect("To never fail"),
    ));
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        content: Content::LocalFile(locator.to_string()),
        locator,
        ..Default::default()
    };
    let (links, _anchors) = find_links(&file, false).expect("No errors");
    assert_eq!(links.len(), 11);
}
