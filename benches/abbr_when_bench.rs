use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zsh_autocomplete_rs::config::{Abbreviation, AbbreviationPosition, AbbreviationWhen};

fn abbreviations(count: usize) -> Vec<Abbreviation> {
    (0..count)
        .map(|index| Abbreviation {
            trigger: format!("abbr-{index}"),
            expansion: format!("expansion-{index}"),
            description: String::new(),
            when: AbbreviationWhen::with_command_patterns(
                AbbreviationPosition::Argument,
                vec![format!("command-{index} *")],
            )
            .unwrap(),
        })
        .collect()
}

fn bench_command_globs(c: &mut Criterion) {
    let mut group = c.benchmark_group("abbreviation_when_command");
    for count in [10, 100, 1_000] {
        let abbreviations = abbreviations(count);
        group.bench_with_input(BenchmarkId::new("no_match", count), &count, |b, _| {
            b.iter(|| {
                abbreviations
                    .iter()
                    .filter(|abbr| abbr.is_available_at(false, Some(black_box("cargo test null"))))
                    .count()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_command_globs);
criterion_main!(benches);
