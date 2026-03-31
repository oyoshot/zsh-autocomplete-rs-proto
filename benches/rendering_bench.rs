mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use crossterm::style::Color;
use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::config::Theme;
use zsh_autocomplete_rs::ui::popup::Popup;
use zsh_autocomplete_rs::ui::render::{layout_candidate, render_popup_to_bytes};

fn bench_popup_compute(c: &mut Criterion) {
    // visible_candidate_indices() is capped at max_visible (10 on 80×24),
    // so only the description variant matters, not total candidate count.
    let mut group = c.benchmark_group("popup_compute");

    {
        let candidates = helpers::generate_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("with_desc"), &app, |b, app| {
            b.iter(|| Popup::compute(app));
        });
    }

    {
        let candidates = helpers::generate_no_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("no_desc"), &app, |b, app| {
            b.iter(|| Popup::compute(app));
        });
    }

    {
        // Long descriptions exercise the max-width scan with wider unicode-width input
        let candidates = helpers::generate_long_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("long_desc"), &app, |b, app| {
            b.iter(|| Popup::compute(app));
        });
    }

    group.finish();
}

fn bench_render_popup_to_bytes(c: &mut Criterion) {
    let default_theme = Theme::default();

    let mut group = c.benchmark_group("render_popup_to_bytes");

    // visible_candidate_indices() is capped at max_visible (10 on 80×24),
    // so varying total candidate count does not change measured work.
    // Instead, vary description style and text encoding.

    {
        let candidates = helpers::generate_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("standard"), &app, |b, app| {
            b.iter(|| render_popup_to_bytes(app, &default_theme));
        });
    }

    {
        let candidates = helpers::generate_no_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("no_desc"), &app, |b, app| {
            b.iter(|| render_popup_to_bytes(app, &default_theme));
        });
    }

    {
        let candidates = helpers::generate_cjk_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("cjk"), &app, |b, app| {
            b.iter(|| render_popup_to_bytes(app, &default_theme));
        });
    }

    {
        let themed = Theme {
            border: Some(Color::Blue),
            selected_fg: Some(Color::White),
            selected_bg: Some(Color::DarkBlue),
            filter: Some(Color::Yellow),
            candidate: Some(Color::Green),
            ..Theme::default()
        };
        let candidates = helpers::generate_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("themed"), &app, |b, app| {
            b.iter(|| render_popup_to_bytes(app, &themed));
        });
    }

    {
        // Exercise the selected-row highlight path (render.rs selected_fg/selected_bg branch)
        let themed = Theme {
            border: Some(Color::Blue),
            selected_fg: Some(Color::White),
            selected_bg: Some(Color::DarkBlue),
            filter: Some(Color::Yellow),
            candidate: Some(Color::Green),
            ..Theme::default()
        };
        let candidates = helpers::generate_candidates(200);
        let mut app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        app.select_first();
        group.bench_with_input(
            BenchmarkId::from_parameter("themed_selected"),
            &app,
            |b, app| {
                b.iter(|| render_popup_to_bytes(app, &themed));
            },
        );
    }

    {
        let candidates = helpers::generate_long_description_candidates(200);
        let app = App::new_with_term_size(candidates, "gi".to_string(), 5, 2, 80, 24);
        group.bench_with_input(BenchmarkId::from_parameter("long_desc"), &app, |b, app| {
            b.iter(|| render_popup_to_bytes(app, &default_theme));
        });
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
