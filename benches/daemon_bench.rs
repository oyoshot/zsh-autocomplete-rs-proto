mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;

fn socket_path() -> String {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{}/zacrs.sock", dir)
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        format!("/tmp/zacrs-{}.sock", user)
    }
}

fn text_ping(sock: &str) -> bool {
    let Ok(stream) = UnixStream::connect(sock) else {
        return false;
    };
    let mut writer = &stream;
    let _ = writer.write_all(b"ping\n");
    let mut reader = BufReader::new(&stream);
    let mut resp = String::new();
    let _ = reader.read_line(&mut resp);
    resp.starts_with("OK")
}

fn text_render(sock: &str, request: &[u8]) -> Option<usize> {
    let stream = UnixStream::connect(sock).ok()?;
    let mut writer = &stream;
    writer.write_all(request).ok()?;
    let mut reader = BufReader::new(&stream);
    let mut header = String::new();
    reader.read_line(&mut header).ok()?;
    if !header.starts_with("OK") {
        return None;
    }
    // Parse tty_len from last token
    let tty_len: usize = header.trim().rsplit(' ').next()?.parse().ok()?;
    let mut tty_bytes = vec![0u8; tty_len];
    reader.read_exact(&mut tty_bytes).ok()?;
    Some(tty_len)
}

fn build_render_request(candidates: &[zsh_autocomplete_rs::candidate::Candidate]) -> Vec<u8> {
    let mut req = String::from("render 5 2 80 24\n");
    req.push_str("gi\n");
    for c in candidates {
        req.push_str(&format!("{}\t{}\t{}\n", c.text, c.description, c.kind));
    }
    req.push_str("END\n");
    req.into_bytes()
}

fn daemon_ping(c: &mut Criterion) {
    let sock = socket_path();
    if !text_ping(&sock) {
        eprintln!("WARNING: daemon not running, skipping daemon benchmarks");
        eprintln!("Start with: cargo run --release -- daemon start &");
        return;
    }

    c.bench_function("daemon_ping_roundtrip", |b| {
        b.iter(|| text_ping(&sock));
    });
}

fn daemon_render(c: &mut Criterion) {
    let sock = socket_path();
    if !text_ping(&sock) {
        return;
    }

    let mut group = c.benchmark_group("daemon_render");
    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_candidates(size);
        let request = build_render_request(&candidates);
        group.bench_with_input(BenchmarkId::from_parameter(size), &request, |b, req| {
            b.iter(|| text_render(&sock, req));
        });
    }
    group.finish();
}

criterion_group!(benches, daemon_ping, daemon_render);
criterion_main!(benches);
