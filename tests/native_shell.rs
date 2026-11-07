use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct ShellGroup(Child);

impl Drop for ShellGroup {
    fn drop(&mut self) {
        // NO_MONITOR keeps even disowned daemon jobs in this private group.
        // Kill the group even if Zsh exited before the daemon opened its socket.
        unsafe { libc::kill(-(self.0.id() as i32), libc::SIGKILL) };
        let _ = self.0.wait();
    }
}

fn run_shell(command: &mut Command, timeout: Duration) -> (Output, bool) {
    // Files cannot keep us waiting for EOF when a daemon inherits the streams.
    let mut stdout = tempfile::tempfile().unwrap();
    let mut stderr = tempfile::tempfile().unwrap();
    let mut child = ShellGroup(
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(stdout.try_clone().unwrap())
            .stderr(stderr.try_clone().unwrap())
            .spawn()
            .expect("native shell smoke test requires Zsh on PATH"),
    );
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break (status, false);
        }
        if Instant::now() >= deadline {
            child.0.kill().unwrap();
            break (child.0.wait().unwrap(), true);
        }
        thread::sleep(Duration::from_millis(10));
    };
    drop(child);
    stdout.seek(SeekFrom::Start(0)).unwrap();
    stderr.seek(SeekFrom::Start(0)).unwrap();
    let mut output = Output {
        status,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    stdout.read_to_end(&mut output.stdout).unwrap();
    stderr.read_to_end(&mut output.stderr).unwrap();
    (output, timed_out)
}

#[test]
fn zsh_reload_reuses_daemon_without_shell_handshake_support() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    let root = tempfile::tempdir().unwrap();
    let runtime = tempfile::Builder::new()
        .prefix("zacrs-old.")
        .tempdir_in("/tmp")
        .unwrap();
    let listener = UnixListener::bind(runtime.path().join("zacrs.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut requests = Vec::new();
        while requests.len() < 4 && Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(error) => panic!("accept: {error}"),
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut request = String::new();
            BufReader::new(&stream).read_line(&mut request).unwrap();
            // Legacy daemons close unknown requests without a response.
            if request == "ping\n" {
                stream.write_all(b"OK\n").unwrap();
            }
            requests.push(request);
        }
        requests
    });
    let (output, timed_out) = run_shell(
        Command::new("zsh")
            .args([
                "-dfi",
                "-c",
                r#"
unsetopt MONITOR
_test_start() {
    [[ "$*" == 'daemon ensure' ]] && return 2
    print -r -- unexpected-start >> "$HOME/started"
}
ZACRS_BIN=_test_start
for reload in 1 2; do
    source "$ZACRS_PLUGIN"
    (( _zacrs_daemon_available == 1 && _zacrs_daemon_started == 0 )) || exit 1
    (( _zacrs_enter_enabled == 0 )) || exit 2
done
[[ ! -e "$HOME/started" ]]
"#,
            ])
            .env_clear()
            .env("PATH", env::var_os("PATH").unwrap_or_default())
            .env("HOME", root.path())
            .env("XDG_RUNTIME_DIR", runtime.path())
            .env(
                "ZACRS_PLUGIN",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shell/zsh-autocomplete-rs.plugin.zsh"
                ),
            ),
        Duration::from_secs(5),
    );
    let requests = server.join().unwrap();
    assert!(!timed_out, "{output:?}");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        requests,
        ["ping shell\n", "ping\n", "ping shell\n", "ping\n"]
    );
    assert!(!root.path().join("started").exists());
}

#[test]
fn zsh_public_init_starts_a_private_daemon() {
    // Cargo supplies the actual executable, including configured target triples.
    let binary = Path::new(env!("CARGO_BIN_EXE_zsh-autocomplete-rs"));
    let root = tempfile::Builder::new()
        .prefix("zacrs-smoke.")
        .tempdir()
        .unwrap();
    // Keep Unix socket paths short, including on macOS where TMPDIR can be long.
    let runtime = tempfile::Builder::new()
        .prefix("zacrs-run.")
        .tempdir_in("/tmp")
        .unwrap();
    fs::create_dir(root.path().join("config")).unwrap();
    let parent_path = env::var_os("PATH").unwrap_or_default();
    let path = env::join_paths(
        std::iter::once(binary.parent().unwrap().to_path_buf())
            .chain(env::split_paths(&parent_path)),
    )
    .unwrap();
    let (output, timed_out) = run_shell(
        Command::new("zsh")
            .args([
                "-d",
                "-f",
                "-i",
                "-c",
                r#"unsetopt MONITOR
trap 'zsh-autocomplete-rs daemon stop >/dev/null 2>&1' EXIT
autoload -Uz compinit
compinit
eval "$(zsh-autocomplete-rs init zsh)"
for attempt in {1..100}; do
    [[ "$(zsh-autocomplete-rs daemon status)" == running ]] && break
    sleep 0.05
done
[[ "$(zsh-autocomplete-rs daemon status)" == running ]] || exit 1
[[ "$(bindkey '^I')" == *'_zacrs_complete_popup' ]] || exit 1
"#,
            ])
            .env_clear()
            .env("PATH", path)
            .env("HOME", root.path())
            .env("ZDOTDIR", root.path())
            .env("XDG_CONFIG_HOME", root.path().join("config"))
            .env("XDG_RUNTIME_DIR", runtime.path())
            .env("TMPDIR", root.path())
            .env("TERM", "xterm-256color")
            .current_dir(root.path()),
        Duration::from_secs(15),
    );
    assert!(!timed_out, "Zsh initialization timed out: {:?}", output);
    assert!(
        output.status.success(),
        "Zsh initialization failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn shell_timeout_is_bounded() {
    let start = Instant::now();
    let (_, timed_out) = run_shell(
        Command::new("zsh").args(["-df", "-c", "unsetopt MONITOR; sleep 30"]),
        Duration::from_millis(100),
    );
    assert!(timed_out);
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn shell_exit_cleans_up_delayed_disowned_jobs() {
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("late-start");
    let (output, timed_out) = run_shell(
        Command::new("zsh")
            .args([
                "-dfi",
                "-c",
                "unsetopt MONITOR; (sleep 0.5; touch \"$MARKER\") &!; exit 1",
            ])
            .env("MARKER", &marker),
        Duration::from_secs(5),
    );
    assert!(!timed_out);
    assert!(!output.status.success());
    thread::sleep(Duration::from_secs(1));
    assert!(!marker.exists(), "delayed job survived shell cleanup");
}
