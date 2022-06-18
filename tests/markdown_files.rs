#[cfg(test)]
use mle::link_extractors::link_extractor::find_links;
use mle::{link::FileLoc, markup::{Content, MarkupFile, MarkupType}};

#[test]
fn no_links() {
    let locator = FileLoc::System(FileSystemLoc::new("./benches/benchmark/markdown/no_links/no_links.md"));
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: locator,
        content: Content::LocalFile(locator.to_string()),
    };
    let (links, _anchors) = find_links(&file, false);
    assert!(links.is_empty());
}

#[test]
fn some_links() {
    let locator = "./benches/benchmark/markdown/many_links/many_links.md";
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: locator.to_string(),
        content: Content::LocalFile(locator.to_string()),
    };
    let (links, _anchors) = find_links(&file, false);
    assert_eq!(links.len(), 11);
}
