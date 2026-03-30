use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use zsh_autocomplete_rs::config::KeyBindings;
use zsh_autocomplete_rs::input::{Action, parse_raw_bytes};

fn parse_raw_bytes_bench(c: &mut Criterion) {
    let bindings = KeyBindings::default();
    let mut group = c.benchmark_group("parse_raw_bytes");

    let cases: &[(&str, &[u8])] = &[
        ("ascii_single", b"a"),
        ("escape_arrow_up", b"\x1b[A"),
        ("escape_ctrl_up", b"\x1b[1;5A"),
        ("utf8_cjk_3byte", "\u{3042}".as_bytes()),
    ];

    for &(name, bytes) in cases {
        group.bench_function(name, |b| {
            b.iter(|| parse_raw_bytes(black_box(bytes), &bindings));
        });
    }

    // Batch: multiple events concatenated in one read buffer.
    // parse_input_event rejects multi-event buffers (returns None),
    // so this measures the parser overhead for that path.
    group.bench_function("batch_concat_rejected", |b| {
        let concat = b"\x1b[A\x1b[B\x1b[C";
        b.iter(|| parse_raw_bytes(black_box(concat.as_slice()), &bindings));
    });

    let keystrokes: &[&[u8]] = &[b"g", b"i", b"t", b" ", b"s", b"t", b"a", b"t", b"u", b"s"];
    group.bench_function("batch_10_sequential", |b| {
        b.iter(|| {
            for &event in keystrokes {
                let _ = parse_raw_bytes(black_box(event), &bindings);
            }
        });
    });

    group.finish();
}

fn map_key_event_to_action_bench(c: &mut Criterion) {
    let default_bindings = KeyBindings::default();
    let mut group = c.benchmark_group("map_key_event_to_action");

    // Mapped keys (produce a recognized Action)
    let mapped: &[(&str, &[u8])] = &[
        ("tab", b"\t"),
        ("enter", b"\r"),
        ("arrow_up", b"\x1b[A"),
        ("ctrl_c", b"\x03"),
        ("escape", b"\x1b"),
        ("space", b" "),
        ("backspace", b"\x7f"),
    ];

    for &(name, bytes) in mapped {
        group.bench_function(format!("mapped/{name}"), |b| {
            b.iter(|| parse_raw_bytes(black_box(bytes), &default_bindings));
        });
    }

    // Unmapped passthrough (returns Action::None)
    let unmapped: &[(&str, &[u8])] = &[
        ("ctrl_a", b"\x01"),
        ("f5", b"\x1b[15~"),
        ("home", b"\x1b[H"),
    ];

    for &(name, bytes) in unmapped {
        group.bench_function(format!("unmapped/{name}"), |b| {
            b.iter(|| parse_raw_bytes(black_box(bytes), &default_bindings));
        });
    }

    // Custom bindings: verify no overhead from non-default ActionMap
    let custom_bindings = KeyBindings {
        tab: Action::Confirm,
        enter: Action::MoveDown,
        ..KeyBindings::default()
    };

    group.bench_function("custom_bindings/tab", |b| {
        b.iter(|| parse_raw_bytes(black_box(b"\t"), &custom_bindings));
    });

    group.bench_function("custom_bindings/enter", |b| {
        b.iter(|| parse_raw_bytes(black_box(b"\r"), &custom_bindings));
    });

    group.finish();
}

criterion_group!(
    benches,
    parse_raw_bytes_bench,
    map_key_event_to_action_bench
);
criterion_main!(benches);
