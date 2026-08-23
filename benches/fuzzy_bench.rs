mod helpers;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::config::Abbreviation;
use zsh_autocomplete_rs::fuzzy::FuzzyMatcher;

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

fn filter_unicode_query_variants(c: &mut Criterion) {
    let candidates = helpers::generate_unicode_candidates(1_000);
    let queries = [
        ("3char", "res"),
        ("normalized_exact", "cafe"),
        ("long_normalized", "sao-paulo"),
        ("no_match", "ωωω"),
    ];

    let mut group = c.benchmark_group("filter_unicode_query_variants");
    for (name, query) in queries {
        group.bench_with_input(BenchmarkId::from_parameter(name), &query, |b, &q| {
            let mut matcher = FuzzyMatcher::new();
            b.iter(|| matcher.filter(&candidates, q));
        });
    }
    group.finish();
}

fn filter_unicode_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_unicode_scaling");
    for size in [100, 1_000, 10_000] {
        let candidates = helpers::generate_unicode_candidates(size);
        group.bench_with_input(
            BenchmarkId::new("normalized_primary", size),
            &candidates,
            |b, cands| {
                let mut matcher = FuzzyMatcher::new();
                b.iter(|| matcher.filter(cands, "cafe"));
            },
        );
    }
    group.finish();
}

fn filter_sequence(c: &mut Criterion) {
    let candidates = helpers::generate_candidates(1_000);
    let mut group = c.benchmark_group("filter_sequence");

    group.bench_function("full_rescan_git", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| {
            let _ = matcher.filter_matches(&candidates, "g", None);
            let _ = matcher.filter_matches(&candidates, "gi", None);
            let _ = matcher.filter_matches(&candidates, "git", None);
        });
    });

    group.bench_function("incremental_git", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| {
            let g = matcher.filter_matches(&candidates, "g", None);
            let g_idx: Vec<usize> = g.iter().map(|r| r.candidate_idx).collect();
            let gi = matcher.filter_matches(&candidates, "gi", Some(&g_idx));
            let gi_idx: Vec<usize> = gi.iter().map(|r| r.candidate_idx).collect();
            let _ = matcher.filter_matches(&candidates, "git", Some(&gi_idx));
        });
    });

    group.finish();
}

fn command_typo_rescue(c: &mut Criterion) {
    let normal_candidates = helpers::generate_realistic_command_candidates(10_000, false);
    let rescue_candidates = helpers::generate_realistic_command_candidates(10_000, true);
    let mut group = c.benchmark_group("command_typo_rescue");

    group.bench_function("normal_prefix_cargo", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| matcher.filter(&normal_candidates, "cargo"));
    });

    group.bench_function("normal_typo_calude", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| matcher.filter(&normal_candidates, "calude"));
    });

    group.bench_function("rescue_typo_calude", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| matcher.filter(&rescue_candidates, "calude"));
    });

    group.finish();
}

fn abbreviation_injection_and_filter(c: &mut Criterion) {
    let native_candidates = helpers::generate_candidates(1_000);
    let mut group = c.benchmark_group("abbreviation_injection_and_filter");

    for abbreviation_count in [0, 100, 1_000] {
        let abbreviations: Vec<_> = (0..abbreviation_count)
            .map(|index| Abbreviation {
                trigger: format!("abbr-{index}"),
                expansion: format!("command-{index} --option '{{{{cursor}}}}'"),
                description: format!("user abbreviation {index}"),
                when: zsh_autocomplete_rs::config::AbbreviationWhen::default(),
            })
            .collect();
        let mut matcher = FuzzyMatcher::new();

        group.bench_with_input(
            BenchmarkId::from_parameter(abbreviation_count),
            &abbreviations,
            |b, abbreviations| {
                b.iter_batched(
                    || native_candidates.clone(),
                    |mut candidates| {
                        candidates.extend(abbreviations.iter().map(|abbr| {
                            Candidate::abbreviation(
                                abbr.trigger.clone(),
                                abbr.expansion.clone(),
                                abbr.description.clone(),
                            )
                        }));
                        matcher.filter(&candidates, "abbr")
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn app_backspace_sequence(c: &mut Criterion) {
    let candidates = helpers::generate_candidates(1_000);
    let mut group = c.benchmark_group("app_backspace_sequence");

    group.bench_function("full_rescan_roundtrip_git", |b| {
        let mut matcher = FuzzyMatcher::new();
        b.iter(|| {
            let _ = matcher.filter_matches(&candidates, "g", None);
            let _ = matcher.filter_matches(&candidates, "gi", None);
            let _ = matcher.filter_matches(&candidates, "git", None);
            let _ = matcher.filter_matches(&candidates, "gi", None);
            let _ = matcher.filter_matches(&candidates, "g", None);
            let _ = matcher.filter_matches(&candidates, "", None);
        });
    });

    group.bench_function("app_cache_roundtrip_git", |b| {
        let mut app = App::new_with_term_size(candidates.clone(), "".to_string(), 5, 2, 80, 24);
        b.iter(|| {
            app.type_char('g');
            app.type_char('i');
            app.type_char('t');
            let _ = app.backspace();
            let _ = app.backspace();
            let _ = app.backspace();
        });
    });

    group.finish();
}
criterion_group!(
    benches,
    filter_scaling,
    filter_query_variants,
    filter_unicode_query_variants,
    filter_unicode_scaling,
    filter_sequence,
    command_typo_rescue,
    abbreviation_injection_and_filter,
    app_backspace_sequence
);
criterion_main!(benches);
