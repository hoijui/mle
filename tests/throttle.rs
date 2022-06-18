#[cfg(test)]
mod helper;

use helper::benches_dir;
use mle::markup::MarkupType;
use mle::Config;
use mle::{logger, State};
use std::time::{Duration, Instant};

const TEST_THROTTLE_MS: u32 = 100;
const TEST_URLS: u32 = 10;
const THROTTLED_TIME_MS: u64 = (TEST_THROTTLE_MS as u64) * ((TEST_URLS as u64) - 1);

#[tokio::test]
async fn throttle_different_hosts() {
    let test_file = benches_dir().join("throttle").join("different_host.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        no_web_anchors: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_paths: vec![],
        root_dir: None,
    };
    let mut state = State::new(config);

    let start = Instant::now();
    mle::run(&mut state).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(THROTTLED_TIME_MS))
}

#[tokio::test]
async fn throttle_same_hosts() {
    let test_file = benches_dir().join("throttle").join("same_host.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        no_web_anchors: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_paths: vec![],
        root_dir: None,
    };
    let mut state = State::new(config);

    let start = Instant::now();
    mle::run(&mut state).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS))
}

#[tokio::test]
async fn throttle_same_ip() {
    let test_file = benches_dir().join("throttle").join("same_ip.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        no_web_anchors: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_paths: vec![],
        root_dir: None,
    };
    let mut state = State::new(config);

    let start = Instant::now();
    mle::run(&mut state).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS))
}
