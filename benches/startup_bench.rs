mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::cli::Cli;
use zsh_autocomplete_rs::config::Config;

fn config_load(c: &mut Criterion) {
    c.bench_function("config_load", |b| {
        b.iter(Config::load);
    });
}

fn clap_parse(c: &mut Criterion) {
    let args = [
        "zsh-autocomplete-rs",
        "render",
        "--prefix",
        "gi",
        "--cursor-row",
        "5",
        "--cursor-col",
        "2",
    ];
    c.bench_function("clap_parse", |b| {
        b.iter(|| <Cli as clap::Parser>::try_parse_from(args));
    });
}

fn candidate_parse_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("candidate_parse_batch");
    for size in [100, 500, 1_000] {
        let lines: Vec<String> = helpers::generate_candidates(size)
            .iter()
            .map(|c| format!("{}\t{}\t{}", c.text, c.description, c.kind))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &lines, |b, lines| {
            b.iter(|| {
                lines
                    .iter()
                    .map(|line| Candidate::parse_line(line))
                    .collect::<Vec<_>>()
            });
        });
    }
    group.finish();
}

fn app_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("app_new");
    for size in [100, 500, 1_000] {
        let candidates = helpers::generate_candidates(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &candidates,
            |b, cands| {
                b.iter(|| App::new_with_term_size(cands.clone(), "gi".to_string(), 5, 2, 80, 24));
            },
        );
    }
    group.finish();
}

fn full_pipeline_no_tty(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline_no_tty");
    for size in [100, 500, 1_000] {
        let lines: Vec<String> = helpers::generate_candidates(size)
            .iter()
            .map(|c| format!("{}\t{}\t{}", c.text, c.description, c.kind))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &lines, |b, lines| {
            b.iter(|| {
                let _cfg = Config::load();
                let _cli = <Cli as clap::Parser>::try_parse_from([
                    "zsh-autocomplete-rs",
                    "render",
                    "--prefix",
                    "gi",
                    "--cursor-row",
                    "5",
                    "--cursor-col",
                    "2",
                ]);
                let candidates: Vec<Candidate> = lines
                    .iter()
                    .map(|line| Candidate::parse_line(line))
                    .collect();
                App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24)
            });
        });
    }
    group.finish();
}

/// Direct subprocess path: config + clap + parse + App::new + render_popup_to_bytes
/// This is what `zsh-autocomplete-rs render` does end-to-end (minus fork/exec and TTY write).
fn full_pipeline_with_render(c: &mut Criterion) {
    use zsh_autocomplete_rs::ui;

    let mut group = c.benchmark_group("full_pipeline_with_render");
    for size in [50, 200, 1_000] {
        let lines: Vec<String> = helpers::generate_candidates(size)
            .iter()
            .map(|c| format!("{}\t{}\t{}", c.text, c.description, c.kind))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &lines, |b, lines| {
            b.iter(|| {
                let cfg = Config::load();
                let _cli = <Cli as clap::Parser>::try_parse_from([
                    "zsh-autocomplete-rs",
                    "render",
                    "--prefix",
                    "gi",
                    "--cursor-row",
                    "5",
                    "--cursor-col",
                    "2",
                ]);
                let theme = cfg.theme();
                let candidates: Vec<Candidate> = lines
                    .iter()
                    .map(|line| Candidate::parse_line(line))
                    .collect();
                let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
                let _ = ui::render::render_popup_to_bytes(&app, &theme);
            });
        });
    }
    group.finish();
}

/// Daemon-equivalent path: parse + App::new_with_matcher + render_popup_to_bytes
/// No config load, no clap parse. Reuses FuzzyMatcher across iterations.
fn daemon_equivalent(c: &mut Criterion) {
    use zsh_autocomplete_rs::config::Theme;
    use zsh_autocomplete_rs::fuzzy::FuzzyMatcher;
    use zsh_autocomplete_rs::ui;

    let theme = Theme::default();
    let mut group = c.benchmark_group("daemon_equivalent");
    for size in [50, 200, 1_000] {
        let lines: Vec<String> = helpers::generate_candidates(size)
            .iter()
            .map(|c| format!("{}\t{}\t{}", c.text, c.description, c.kind))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &lines, |b, lines| {
            let mut fuzzy = Some(FuzzyMatcher::new());
            b.iter(|| {
                let candidates: Vec<Candidate> = lines
                    .iter()
                    .map(|line| Candidate::parse_line(line))
                    .collect();
                let fm = fuzzy.take().unwrap_or_default();
                let app = App::new_with_matcher(candidates, "gi".to_string(), 5, 2, 80, 24, fm);
                let result = ui::render::render_popup_to_bytes(&app, &theme);
                fuzzy = Some(app.take_fuzzy());
                result
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    config_load,
    clap_parse,
    candidate_parse_batch,
    app_new,
    full_pipeline_no_tty,
    full_pipeline_with_render,
    daemon_equivalent
);
criterion_main!(benches);
