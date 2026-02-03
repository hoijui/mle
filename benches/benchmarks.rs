// SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{Criterion, criterion_group, criterion_main};
use mle::config::Config;
use mle::markup;
use mle::state::State;
use std::fs;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

async fn end_to_end_benchmark() {
    init();
    let markup_types = vec![markup::Type::Markdown];
    let root = fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap();
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config {
        markup_files,
        ..Default::default()
    };
    let mut state = State::new(config);
    mle::run(&mut state).await.unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("End to end benchmark", |b| b.iter(end_to_end_benchmark));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
