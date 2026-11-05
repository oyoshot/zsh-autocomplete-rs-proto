use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::process::CommandExt;
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
fn zsh_enter_abbreviation_is_opt_in_and_preserves_input() {
    run_abbreviation_test(include_str!("support/abbreviation_enter.zsh"), false);
}

#[test]
fn zsh_enter_abbreviation_zle_without_daemon() {
    run_abbreviation_test(include_str!("support/abbreviation_zle.zsh"), false);
}

#[test]
fn zsh_enter_abbreviation_zle_with_daemon_popup() {
    run_abbreviation_test(include_str!("support/abbreviation_zle.zsh"), true);
}

fn run_abbreviation_test(script: &str, daemon: bool) {
    let root = tempfile::tempdir().unwrap();
    let runtime = tempfile::Builder::new()
        .prefix("zacrs-abbr.")
        .tempdir_in("/tmp")
        .unwrap();
    let config_dir = root.path().join("config/zacrs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"[abbreviations]
gs = "git status"
next = "gs"
zacrs_next = "gs"
connect = "ssh user@{{cursor}}"
literal = "echo $(touch should-not-exist)"
multi = "echo one\necho two"

[[abbreviation]]
trigger = "gcm"
expansion = "git commit -m '{{cursor}}'"

[[abbreviation]]
trigger = "ssh"
expansion = "ssh user@{{cursor}}"

[[abbreviation]]
trigger = "arg"
expansion = "argument only"
when.position = "argument"

[[abbreviation]]
trigger = "scoped"
expansion = "git status"
when.command = "scoped -s*"
"#,
    )
    .unwrap();
    let (output, timed_out) = run_shell(
        Command::new("zsh")
            .args(["-df", "-c", script])
            .env_clear()
            .env("PATH", env::var_os("PATH").unwrap_or_default())
            .env("HOME", root.path())
            .env("ZDOTDIR", root.path())
            .env("XDG_CONFIG_HOME", root.path().join("config"))
            .env("ZACRS_BIN", env!("CARGO_BIN_EXE_zsh-autocomplete-rs"))
            .env("XDG_RUNTIME_DIR", runtime.path())
            .env("ZACRS_TEST_DAEMON", if daemon { "1" } else { "0" })
            .env(
                "ZACRS_PLUGIN",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shell/zsh-autocomplete-rs.plugin.zsh"
                ),
            )
            .env("TERM", "xterm-256color")
            .current_dir(root.path()),
        Duration::from_secs(30),
    );
    // zpty children have their own process group. Also stop the private daemon
    // if the harness timed out before its EXIT trap could clean up.
    if daemon {
        let _ = Command::new(env!("CARGO_BIN_EXE_zsh-autocomplete-rs"))
            .args(["daemon", "stop"])
            .env("XDG_RUNTIME_DIR", runtime.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    assert!(!timed_out, "abbreviation test timed out: {output:?}");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.path().join("should-not-exist").exists());
}
