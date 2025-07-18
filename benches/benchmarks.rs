// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
// SPDX-FileCopyrightText: 2020 Armin Becher <becherarmin@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use mle::config::Config;
use mle::markup::Type;
use mle::state::State;
use std::fs;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn end_to_end_benchmark() {
    init();
    let config = Config {
        files_and_dirs: vec![
            fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir")
                .unwrap()
                .into(),
        ],
        markup_types: vec![Type::Markdown],
        ..Default::default()
    };
    let mut state = State::new(config);
    let _ = mle::run(&mut state);
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
