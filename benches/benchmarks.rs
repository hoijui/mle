#[cfg(test)]
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use mle::config::Config;
use mle::markup::MarkupType;
use mle::{logger, state::State};
use std::fs;

fn end_to_end_benchmark() {
    let config = Config {
        scan_root: fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut state = State::new(config);
    let _ = mle::run(&mut state);
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
