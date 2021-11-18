#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::logger;
use mlc::markup::MarkupType;
use mlc::Config;
use std::convert::TryInto;

#[tokio::test]
async fn end_to_end() {
    let config = Config {
        folder: benches_dir().join("benchmark"),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        throttle: 0,
        ignore_links: vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")],
        ignore_paths: vec![
            "benches/benchmark/markdown/ignore_me.md"
                .try_into()
                .unwrap(),
            "./benches/benchmark/markdown/ignore_me_dir"
                .try_into()
                .unwrap(),
        ],
        root_dir: None,
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let config = Config {
        folder: test_files.clone(),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        ignore_links: vec![],
        ignore_paths: vec![],
        throttle: 0,
        root_dir: Some(test_files),
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
