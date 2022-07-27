#[cfg(test)]
mod helper;

use helper::benches_dir;
use mle::config::Config;
use mle::logger;
use mle::markup::Type;
use mle::state::State;
use std::convert::TryInto;

#[tokio::test]
async fn end_to_end() {
    let config = Config {
        scan_root: benches_dir().join("benchmark"),
        log_level: logger::LogLevel::Debug,
        log_file: None,
        recursive: true,
        links: true,
        anchors: true,
        result_file: None,
        result_format: "json".to_owned(),
        resolve_root: None,
        markup_types: vec![Type::Markdown],
        // match_file_extension: false,
        // throttle: 0,
        ignore_links: vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")],
        ignore_paths: vec![
            "benches/benchmark/markdown/ignore_me.md"
                .try_into()
                .unwrap(),
            "./benches/benchmark/markdown/ignore_me_dir"
                .try_into()
                .unwrap(),
        ],
        // root_dir: None,
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
        scan_root: test_files.clone(),
        log_level: logger::LogLevel::Debug,
        log_file: None,
        recursive: true,
        links: true,
        anchors: true,
        result_file: None,
        result_format: "json".to_owned(),
        resolve_root: Some(test_files),
        markup_types: vec![Type::Markdown],
        // match_file_extension: false,
        // throttle: 0,
        ignore_links: vec![],
        ignore_paths: vec![],
        // root_dir: None,
    };
    let mut state = State::new(config);
    if let Err(e) = mle::run(&mut state).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
