mod helpers;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zsh_autocomplete_rs::config::Config;

const BENCH_CONFIG_TOML: &str = "\
[theme]\n\
border = \"blue\"\n\
selected-bg = \"dark-green\"\n\
\n\
[keybindings]\n\
tab = \"move-down\"\n\
";

struct BenchDaemon {
    socket_path: String,
    runtime_dir: std::path::PathBuf,
    child: Child,
}

impl BenchDaemon {
    fn start() -> Self {
        let runtime_dir = bench_runtime_dir();
        fs::create_dir_all(&runtime_dir).expect("failed to create bench runtime dir");

        let socket_path = runtime_dir.join("zacrs.sock");
        let socket_path_str = socket_path.to_string_lossy().into_owned();
        let mut child = Command::new(release_daemon_binary())
            .env("XDG_RUNTIME_DIR", &runtime_dir)
            .arg("daemon")
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn bench daemon");

        for _ in 0..100 {
            if text_ping(&socket_path_str) {
                return Self {
                    socket_path: socket_path_str,
                    runtime_dir,
                    child,
                };
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "bench daemon did not start listening on {}",
            socket_path.display()
        );
    }
}

impl Drop for BenchDaemon {
    fn drop(&mut self) {
        if let Ok(stream) = UnixStream::connect(&self.socket_path) {
            let mut writer = &stream;
            let _ = writer.write_all(&zsh_autocomplete_rs::protocol::Request::Shutdown.serialize());
            let mut reader = BufReader::new(&stream);
            let _ = zsh_autocomplete_rs::protocol::Response::deserialize(&mut reader);
        }

        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }

        let _ = fs::remove_dir_all(&self.runtime_dir);
    }
}

fn bench_runtime_dir() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zacrs-bench-{}-{nonce}", std::process::id()))
}

fn release_daemon_binary() -> &'static std::path::PathBuf {
    static RELEASE_BIN: OnceLock<std::path::PathBuf> = OnceLock::new();

    RELEASE_BIN.get_or_init(|| {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target_dir = std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| manifest_dir.join("target"));
        let target_dir = if target_dir.is_relative() {
            manifest_dir.join(target_dir)
        } else {
            target_dir
        };

        let status = Command::new("cargo")
            .current_dir(&manifest_dir)
            .env("CARGO_TARGET_DIR", &target_dir)
            .arg("build")
            .arg("--release")
            .arg("--bin")
            .arg("zsh-autocomplete-rs")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to build release daemon binary");
        assert!(status.success(), "release daemon build failed");

        target_dir.join("release").join("zsh-autocomplete-rs")
    })
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
    let daemon = BenchDaemon::start();
    let sock = &daemon.socket_path;

    c.bench_function("daemon_ping_roundtrip", |b| {
        b.iter(|| text_ping(sock));
    });
}

fn daemon_render(c: &mut Criterion) {
    let daemon = BenchDaemon::start();
    let sock = &daemon.socket_path;

    let mut group = c.benchmark_group("daemon_render");
    for size in [50, 200, 1_000] {
        let candidates = helpers::generate_candidates(size);
        let request = build_request("render", &candidates);
        group.bench_with_input(BenchmarkId::from_parameter(size), &request, |b, req| {
            b.iter(|| text_render(sock, req));
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
    let daemon = BenchDaemon::start();
    let sock = &daemon.socket_path;

    let candidates = helpers::generate_candidates(50);
    let request = build_request("complete", &candidates);

    let arrow_down = b"KEY 3\n\x1b[B";
    let enter_key = b"KEY 1\n\r";

    c.bench_function("daemon_complete_session", |b| {
        b.iter(|| {
            let stream = UnixStream::connect(sock).unwrap();
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

fn config_reload_mtime_check(c: &mut Criterion) {
    let dir = bench_runtime_dir();
    let cfg_dir = dir.join("zacrs");
    fs::create_dir_all(&cfg_dir).expect("failed to create temp config dir");
    let cfg_path = cfg_dir.join("config.toml");
    fs::write(&cfg_path, BENCH_CONFIG_TOML).expect("failed to write temp config");

    c.bench_function("config_reload_mtime_check", |b| {
        b.iter(|| fs::metadata(&cfg_path).ok().and_then(|m| m.modified().ok()));
    });

    let _ = fs::remove_dir_all(&dir);
}

/// Benchmarks Config::load() + theme() + key_bindings() against a real TOML file
/// to capture the full reload cost (not just Config::default()).
fn config_reload_full(c: &mut Criterion) {
    let dir = bench_runtime_dir();
    let cfg_dir = dir.join("zacrs");
    fs::create_dir_all(&cfg_dir).expect("failed to create temp config dir");
    fs::write(cfg_dir.join("config.toml"), BENCH_CONFIG_TOML).expect("failed to write temp config");

    // SAFETY: benchmark is single-threaded at this point (before criterion harness).
    unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };

    c.bench_function("config_reload_full", |b| {
        b.iter(|| {
            let config = Config::load();
            let _theme = config.theme();
            let _key_bindings = config.key_bindings();
        });
    });

    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    let _ = fs::remove_dir_all(&dir);
}

criterion_group!(
    benches,
    daemon_ping,
    daemon_render,
    daemon_complete_session,
    config_reload_mtime_check,
    config_reload_full
);
criterion_main!(benches);
