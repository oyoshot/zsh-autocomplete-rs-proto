mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::fuzzy::{FuzzyMatcher, damerau_levenshtein};

fn filter_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_scaling");
    for size in [100, 1_000, 10_000] {
        let candidates = helpers::generate_candidates(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &candidates,
            |b, cands| {
                let mut matcher = FuzzyMatcher::new();
                b.iter(|| matcher.filter(cands, "git"));
            },
        );
    }
    group.finish();
}

fn filter_query_variants(c: &mut Criterion) {
    let candidates = helpers::generate_candidates(1_000);
    let queries = [
        ("empty", ""),
        ("1char", "g"),
        ("3char", "git"),
        ("exact", "cargo-build"),
        ("typo", "gti"),
        ("typo_no_subseq", "calude"),
        ("no_match", "zzzzz"),
        ("long", "git-checkout-branch"),
    ];

    let mut group = c.benchmark_group("filter_query_variants");
    for (name, query) in queries {
        group.bench_with_input(BenchmarkId::from_parameter(name), &query, |b, &q| {
            let mut matcher = FuzzyMatcher::new();
            b.iter(|| matcher.filter(&candidates, q));
        });
    }
    group.finish();
}

fn bench_damerau_levenshtein(c: &mut Criterion) {
    let pairs = [
        ("identical", "cargo", "cargo"),
        ("transposition", "git", "gti"),
        ("substitution", "cargo", "carog"),
        ("long_strings", "git-checkout-branch", "git-chekoctu-branch"),
        ("different_len", "git", "github"),
    ];

    let mut group = c.benchmark_group("damerau_levenshtein");
    for (name, a, b) in pairs {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(a, b),
            |bench, &(a, b)| {
                bench.iter(|| damerau_levenshtein(a, b));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    filter_scaling,
    filter_query_variants,
    bench_damerau_levenshtein
);
criterion_main!(benches);
