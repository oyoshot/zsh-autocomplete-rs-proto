mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::handoff::compute_reuse_token;
use zsh_autocomplete_rs::ui::popup::Popup;

fn candidates_to_tsv(app: &App) -> String {
    let mut tsv = String::new();
    for c in &app.all_candidates {
        tsv.push_str(&c.text);
        tsv.push('\t');
        tsv.push_str(&c.description);
        tsv.push('\t');
        tsv.push_str(&c.kind);
        tsv.push('\n');
    }
    tsv
}

fn token_candidate_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("reuse_token_candidate_scaling");
    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_candidates(size);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        let popup = Popup::compute(&app);
        let tsv = candidates_to_tsv(&app);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| compute_reuse_token("gi", &tsv, &app, &popup));
        });
    }
    group.finish();
}

fn token_prefix_length(c: &mut Criterion) {
    let candidates = helpers::generate_candidates(200);
    let prefixes = [
        ("1char", "g"),
        ("2char", "gi"),
        ("5char", "git-c"),
        ("long", "cargo-build-release"),
    ];

    let mut group = c.benchmark_group("reuse_token_prefix_length");
    for (name, prefix) in prefixes {
        let app = App::new_with_term_size(candidates.clone(), prefix.to_string(), 5, 2, 80, 24);
        let popup = Popup::compute(&app);
        let tsv = candidates_to_tsv(&app);

        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _| {
            b.iter(|| compute_reuse_token(prefix, &tsv, &app, &popup));
        });
    }
    group.finish();
}

fn token_stability(c: &mut Criterion) {
    let candidates = helpers::generate_candidates(200);
    let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
    let popup = Popup::compute(&app);
    let tsv = candidates_to_tsv(&app);

    let mut group = c.benchmark_group("reuse_token_stability");
    group.bench_function("repeated_identical_input", |b| {
        b.iter(|| {
            let t1 = compute_reuse_token("gi", &tsv, &app, &popup);
            let t2 = compute_reuse_token("gi", &tsv, &app, &popup);
            let t3 = compute_reuse_token("gi", &tsv, &app, &popup);
            assert_eq!(t1, t2);
            assert_eq!(t2, t3);
            t3
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    token_candidate_scaling,
    token_prefix_length,
    token_stability
);
criterion_main!(benches);
