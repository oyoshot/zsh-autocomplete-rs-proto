mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::app::compute_common_prefix;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::ui::render::truncate_to_width;

fn bench_truncate_to_width(c: &mut Criterion) {
    let cases = [
        ("ascii_no_trunc", "cargo-build --release", 40),
        ("ascii_trunc", "cargo-build --release --target x86_64-unknown-linux-gnu", 20),
        ("cjk_no_trunc", "ファイル一覧", 20),
        ("cjk_trunc", "ファイル一覧を表示するコマンド", 10),
        ("mixed_no_trunc", "git-日本語テスト", 30),
        ("mixed_trunc", "git-日本語テストの長い文字列", 12),
    ];

    let mut group = c.benchmark_group("truncate_to_width");
    for (name, input, width) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &(input, width), |b, &(s, w)| {
            b.iter(|| truncate_to_width(s, w));
        });
    }
    group.finish();
}

fn bench_parse_line(c: &mut Criterion) {
    let cases = [
        ("1field", "git"),
        ("2fields", "git\tversion control"),
        ("3fields", "git\tversion control\tcommand"),
        ("long_desc", "git\tthe stupid content tracker - a revision control system\tcommand"),
    ];

    let mut group = c.benchmark_group("parse_line");
    for (name, input) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, &s| {
            b.iter(|| Candidate::parse_line(s));
        });
    }
    group.finish();
}

fn bench_compute_common_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute_common_prefix");

    for size in [10, 100, 1_000] {
        let candidates = helpers::generate_prefixed_candidates("cargo-", size);
        group.bench_with_input(
            BenchmarkId::new("with_prefix", size),
            &candidates,
            |b, cands| {
                b.iter(|| compute_common_prefix(cands, "car"));
            },
        );
    }

    let candidates = helpers::generate_candidates(1_000);
    group.bench_with_input(
        BenchmarkId::new("no_prefix", 1_000),
        &candidates,
        |b, cands| {
            b.iter(|| compute_common_prefix(cands, ""));
        },
    );

    group.finish();
}

criterion_group!(benches, bench_truncate_to_width, bench_parse_line, bench_compute_common_prefix);
criterion_main!(benches);
