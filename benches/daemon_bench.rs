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

fn build_request(
    command: &str,
    candidates: &[zsh_autocomplete_rs::candidate::Candidate],
) -> Vec<u8> {
    let mut req = format!("{} 5 2 80 24\n", command);
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
        let request = build_request("render", &candidates);
        group.bench_with_input(BenchmarkId::from_parameter(size), &request, |b, req| {
            b.iter(|| text_render(&sock, req));
        });
    }
    group.finish();
}

fn read_frame(reader: &mut BufReader<&UnixStream>) -> Option<usize> {
    let mut header = String::new();
    reader.read_line(&mut header).ok()?;
    if !header.starts_with("FRAME") {
        return None;
    }
    let tty_len: usize = header.trim().rsplit(' ').next()?.parse().ok()?;
    if tty_len > 0 {
        let mut tty_bytes = vec![0u8; tty_len];
        reader.read_exact(&mut tty_bytes).ok()?;
    }
    Some(tty_len)
}

/// Benchmark a full interactive complete session:
/// 1. Send complete request (50 candidates)
/// 2. Receive initial FRAME
/// 3. Send KEY ↓ twice, receive FRAME each time
/// 4. Send KEY Enter, receive DONE
fn daemon_complete_session(c: &mut Criterion) {
    let sock = socket_path();
    if !text_ping(&sock) {
        return;
    }

    let candidates = helpers::generate_candidates(50);
    let request = build_request("complete", &candidates);

    let arrow_down = b"KEY 3\n\x1b[B";
    let enter_key = b"KEY 1\n\r";

    c.bench_function("daemon_complete_session", |b| {
        b.iter(|| {
            let stream = UnixStream::connect(&sock).unwrap();
            let mut writer = &stream;
            let mut reader = BufReader::new(&stream);

            // Send complete request + read initial FRAME
            writer.write_all(&request).unwrap();
            read_frame(&mut reader).unwrap();

            // Two arrow-down keypresses
            writer.write_all(arrow_down).unwrap();
            read_frame(&mut reader).unwrap();
            writer.write_all(arrow_down).unwrap();
            read_frame(&mut reader).unwrap();

            // Confirm with Enter
            writer.write_all(enter_key).unwrap();
            let mut done_line = String::new();
            reader.read_line(&mut done_line).unwrap();
            assert!(
                done_line.starts_with("DONE"),
                "expected DONE, got: {done_line}"
            );
        });
    });
}

criterion_group!(benches, daemon_ping, daemon_render, daemon_complete_session);
criterion_main!(benches);
