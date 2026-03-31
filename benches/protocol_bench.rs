mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zsh_autocomplete_rs::protocol::{Request, Response};

fn generate_tty_bytes(size: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(size);
    while bytes.len() < size {
        bytes.extend_from_slice(b"\x1b[38;5;12m"); // color
        bytes.extend_from_slice(b"sample-text");
        bytes.extend_from_slice(b"\x1b[0m"); // reset
        bytes.extend_from_slice(b"\x1b[K"); // clear line
        bytes.push(b'\n');
    }
    bytes.truncate(size);
    bytes
}

const METADATA: &str = "popup_row=6 popup_height=12 cursor_row=5 total=200 filtered=45 selected=0";

fn make_render_request(candidate_count: usize) -> Request {
    let candidates = helpers::generate_candidates(candidate_count);
    let tsv = helpers::candidates_to_tsv(&candidates).into_bytes();
    Request::Render {
        prefix: "gi".to_string(),
        cursor_row: 5,
        cursor_col: 2,
        term_cols: 80,
        term_rows: 24,
        candidates_tsv: tsv,
        selected: Some(0),
    }
}

fn request_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_serialize");

    for size in [50, 200, 1_000] {
        let req = make_render_request(size);
        group.bench_with_input(BenchmarkId::new("RenderRequest", size), &req, |b, req| {
            b.iter(|| black_box(req).serialize());
        });
    }

    let clear_req = Request::Clear {
        popup_row: 6,
        popup_height: 12,
        cursor_row: 5,
    };
    group.bench_function("ClearRequest", |b| {
        b.iter(|| black_box(&clear_req).serialize());
    });

    group.bench_function("PingRequest", |b| {
        b.iter(|| black_box(&Request::Ping).serialize());
    });

    group.bench_function("ShutdownRequest", |b| {
        b.iter(|| black_box(&Request::Shutdown).serialize());
    });

    group.finish();
}

fn response_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_serialize");

    let tty_bytes = generate_tty_bytes(4096);
    let success = Response::Success {
        tty_bytes,
        metadata: Some(METADATA.to_string()),
    };
    group.bench_function("SuccessResponse", |b| {
        b.iter(|| black_box(&success).serialize());
    });

    group.bench_function("EmptyResponse", |b| {
        b.iter(|| black_box(&Response::Empty).serialize());
    });

    let error = Response::Error("something went wrong".to_string());
    group.bench_function("ErrorResponse", |b| {
        b.iter(|| black_box(&error).serialize());
    });

    group.finish();
}

fn request_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_deserialize");

    for size in [50, 200, 1_000] {
        let req = make_render_request(size);
        let bytes = req.serialize();
        group.bench_with_input(
            BenchmarkId::new("RenderRequest", size),
            &bytes,
            |b, bytes| {
                b.iter(|| black_box(Request::deserialize(&mut black_box(bytes.as_slice())).unwrap()));
            },
        );
    }

    let clear_bytes = Request::Clear {
        popup_row: 6,
        popup_height: 12,
        cursor_row: 5,
    }
    .serialize();
    group.bench_function("ClearRequest", |b| {
        b.iter(|| Request::deserialize(&mut black_box(clear_bytes.as_slice())));
    });

    let ping_bytes = Request::Ping.serialize();
    group.bench_function("PingRequest", |b| {
        b.iter(|| Request::deserialize(&mut black_box(ping_bytes.as_slice())));
    });

    let shutdown_bytes = Request::Shutdown.serialize();
    group.bench_function("ShutdownRequest", |b| {
        b.iter(|| Request::deserialize(&mut black_box(shutdown_bytes.as_slice())));
    });

    group.finish();
}

fn response_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_deserialize");

    let tty_bytes = generate_tty_bytes(4096);
    let success_bytes = Response::Success {
        tty_bytes,
        metadata: Some(METADATA.to_string()),
    }
    .serialize();
    group.bench_function("SuccessResponse", |b| {
        b.iter(|| black_box(Response::deserialize(&mut black_box(success_bytes.as_slice())).unwrap()));
    });

    let empty_bytes = Response::Empty.serialize();
    group.bench_function("EmptyResponse", |b| {
        b.iter(|| black_box(Response::deserialize(&mut black_box(empty_bytes.as_slice())).unwrap()));
    });

    let error_bytes = Response::Error("something went wrong".to_string()).serialize();
    group.bench_function("ErrorResponse", |b| {
        b.iter(|| black_box(Response::deserialize(&mut black_box(error_bytes.as_slice())).unwrap()));
    });

    group.finish();
}

fn request_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_roundtrip");

    for size in [50, 200, 1_000] {
        let req = make_render_request(size);
        group.bench_with_input(BenchmarkId::new("RenderRequest", size), &req, |b, req| {
            b.iter(|| {
                let bytes = black_box(req).serialize();
                black_box(Request::deserialize(&mut bytes.as_slice()).unwrap())
            });
        });
    }

    group.finish();
}

fn response_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_roundtrip");

    let tty_bytes = generate_tty_bytes(4096);
    let success = Response::Success {
        tty_bytes,
        metadata: Some(METADATA.to_string()),
    };
    group.bench_function("SuccessResponse", |b| {
        b.iter(|| {
            let bytes = black_box(&success).serialize();
            black_box(Response::deserialize(&mut bytes.as_slice()).unwrap())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    request_serialize,
    response_serialize,
    request_deserialize,
    response_deserialize,
    request_roundtrip,
    response_roundtrip,
);
criterion_main!(benches);
