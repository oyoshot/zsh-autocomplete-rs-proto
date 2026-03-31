mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use crossterm::style::Color;
use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::config::Theme;
use zsh_autocomplete_rs::ui::popup::Popup;
use zsh_autocomplete_rs::ui::render::{layout_candidate, render_popup_to_bytes};

fn bench_popup_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("popup_compute");

    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_candidates(size);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("with_desc", size),
            &app,
            |b, app| {
                b.iter(|| Popup::compute(app));
            },
        );
    }

    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_no_description_candidates(size);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("no_desc", size),
            &app,
            |b, app| {
                b.iter(|| Popup::compute(app));
            },
        );
    }

    // Long descriptions exercise the max-width scan with wider unicode-width input
    let candidates = helpers::generate_long_description_candidates(200);
    let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
    group.bench_with_input(
        BenchmarkId::new("long_desc", 200),
        &app,
        |b, app| {
            b.iter(|| Popup::compute(app));
        },
    );

    group.finish();
}

fn bench_render_popup_to_bytes(c: &mut Criterion) {
    let default_theme = Theme::default();

    let mut group = c.benchmark_group("render_popup_to_bytes");

    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_candidates(size);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("standard", size),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &default_theme));
            },
        );
    }

    {
        let candidates = helpers::generate_no_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("no_desc", 200),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &default_theme));
            },
        );
    }

    {
        let candidates = helpers::generate_cjk_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("cjk", 200),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &default_theme));
            },
        );
    }

    {
        let themed = Theme {
            border: Some(Color::Blue),
            selected_fg: Some(Color::White),
            selected_bg: Some(Color::DarkBlue),
            description: Color::DarkGrey,
            filter: Some(Color::Yellow),
            candidate: Some(Color::Green),
        };
        let candidates = helpers::generate_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("themed", 200),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &themed));
            },
        );
    }

    {
        let candidates = helpers::generate_long_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(
            BenchmarkId::new("long_desc", 200),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &default_theme));
            },
        );
    }

    group.finish();
}

fn bench_layout_candidate(c: &mut Criterion) {
    let inner: usize = 58; // MAX_POPUP_WIDTH(60) - 2 borders

    let mut group = c.benchmark_group("layout_candidate");

    {
        let candidate = Candidate {
            text: "cargo-build".to_string(),
            description: "compile the current package".to_string(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("ascii_with_desc"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    {
        let candidate = Candidate {
            text: "cargo-build".to_string(),
            description: String::new(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("ascii_no_desc"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    {
        let candidate = Candidate {
            text: "ファイル一覧表示".to_string(),
            description: "ディレクトリの内容を表示するコマンド".to_string(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("cjk_with_desc"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    {
        let candidate = Candidate {
            text: "ファイル一覧表示".to_string(),
            description: String::new(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("cjk_no_desc"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    {
        let candidate = Candidate {
            text: "git-ブランチ一覧".to_string(),
            description: "list all branches ブランチ表示".to_string(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("mixed_with_desc"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    {
        let candidate = Candidate {
            text: "cargo-build".to_string(),
            description: "compile the current package with all dependencies resolved \
                          and optimizations applied across the entire workspace"
                .to_string(),
            kind: "command".to_string(),
        };
        group.bench_with_input(
            BenchmarkId::from_parameter("long_desc_truncation"),
            &candidate,
            |b, c| {
                b.iter(|| layout_candidate(c, inner));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_popup_compute,
    bench_render_popup_to_bytes,
    bench_layout_candidate,
);
criterion_main!(benches);
