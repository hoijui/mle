#[cfg(test)]
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use mlc::logger;
use mlc::markup::MarkupType;
use mlc::Config;
use std::fs;

fn end_to_end_benchmark() {
    let config = Config {
        folder: fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        ignore_links: vec![],
        match_file_extension: false,
        ignore_paths: vec![],
        root_dir: None,
        throttle: 0,
        no_web_links: false,
    };
    let _ = mlc::run(&config);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("End to end benchmark", |b| {
        b.iter(|| end_to_end_benchmark())
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
