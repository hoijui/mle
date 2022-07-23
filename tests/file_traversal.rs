use mle::config::Config;
#[cfg(test)]
use mle::file_traversal;
use mle::markup::{MarkupFile, MarkupType};
use std::path::Path;

#[test]
fn find_markdown_files() {
    let path = Path::new("./benches/benchmark/markdown/md_file_endings").to_path_buf();
    let config: Config = Config {
        scan_root: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert_eq!(result.len(), 12);
}

#[test]
fn empty_folder() {
    let path = Path::new("./benches/benchmark/markdown/empty").to_path_buf();
    let config: Config = Config {
        scan_root: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert!(result.is_empty());
}
