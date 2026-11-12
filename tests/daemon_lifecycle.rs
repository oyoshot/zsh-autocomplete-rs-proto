use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use zsh_autocomplete_rs::protocol::TextFrameHeader;

struct Fixture {
    home: tempfile::TempDir,
    runtime: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
            runtime: tempfile::Builder::new()
                .prefix("zacrs-life.")
                .tempdir_in("/tmp")
                .unwrap(),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_zsh-autocomplete-rs"));
        command
            .env_clear()
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path().join("config"))
            .env("XDG_RUNTIME_DIR", self.runtime.path());
        command
    }

    fn connect(&self) -> UnixStream {
        let stream = UnixStream::connect(self.runtime.path().join("zacrs.sock")).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
    }

    fn ensure(&self) -> String {
        let output = self.command().args(["daemon", "ensure"]).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let reply = String::from_utf8(output.stdout).unwrap();
        assert!(reply.starts_with("OK"), "{reply}");
        reply
    }

    fn inode(&self) -> u64 {
        fs::metadata(self.runtime.path().join("zacrs.sock"))
            .unwrap()
            .ino()
    }

    fn shell(&self, binary: &std::path::Path, script: &str) {
        let output = Command::new("zsh")
            .args(["-dfi", "-c", script])
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path().join("config"))
            .env("XDG_RUNTIME_DIR", self.runtime.path())
            .env("ZACRS_BIN", binary)
            .env(
                "ZACRS_PLUGIN",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shell/zsh-autocomplete-rs.plugin.zsh"
                ),
            )
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self
            .command()
            .args(["daemon", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[test]
fn current_daemon_is_reused_and_configuration_is_returned() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.home.path().join("config/zacrs")).unwrap();
    fs::write(
        fixture.home.path().join("config/zacrs/config.toml"),
        "[completion]\nexpand_abbreviation_on_enter = true\n",
    )
    .unwrap();
    let first = fixture.ensure();
    assert!(first.contains("enter_expansion=1 edit_context=2"));
    let inode = fixture.inode();
    assert_eq!(fixture.ensure(), first);
    assert_eq!(fixture.inode(), inode);
}

#[test]
fn concurrent_initializations_share_one_daemon() {
    let fixture = Fixture::new();
    let children: Vec<_> = (0..8)
        .map(|_| {
            fixture
                .command()
                .args(["daemon", "ensure"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(String::from_utf8(output.stdout).unwrap().starts_with("OK"));
    }
    let inode = fixture.inode();
    fixture.ensure();
    assert_eq!(fixture.inode(), inode);
}

#[test]
fn warm_shell_reload_launches_no_helper() {
    let fixture = Fixture::new();
    fixture.ensure();
    let inode = fixture.inode();
    fixture.shell(
        std::path::Path::new(env!("CARGO_BIN_EXE_zsh-autocomplete-rs")),
        r#"
functions[$ZACRS_BIN]='print -u2 -- unexpected-helper; return 1'
setopt hashlistall
for reload in 1 2; do
    source "$ZACRS_PLUGIN"
    (( _zacrs_daemon_available == 1 )) || exit 1
    [[ "$options[hashlistall]" == on ]] || exit 2
done
# The normal init path uses a command name, not an absolute path.
path=( "${ZACRS_BIN:h}" "${path[@]}" )
ZACRS_BIN="${ZACRS_BIN:t}"
functions[$ZACRS_BIN]='print -u2 -- unexpected-named-helper; return 1'
source "$ZACRS_PLUGIN"
(( _zacrs_daemon_available == 1 )) || exit 1
[[ "$options[hashlistall]" == on ]] || exit 2
"#,
    );
    assert_eq!(fixture.inode(), inode);
}

#[test]
fn update_stops_daemon_before_shell_starts_replacement() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let original = std::path::Path::new(env!("CARGO_BIN_EXE_zsh-autocomplete-rs"));
    let installed = fixture.home.path().join("installed");
    symlink(original, &installed).unwrap();
    fixture.ensure();
    let replacement = fixture.home.path().join("new-binary");
    fs::copy(original, &replacement).unwrap();
    fs::remove_file(&installed).unwrap();
    symlink(&replacement, &installed).unwrap();
    // Updating stops the listener; shell startup performs no binary identity check.
    let status = Command::new(&installed)
        .args(["daemon", "stop"])
        .env("XDG_RUNTIME_DIR", fixture.runtime.path())
        .status()
        .unwrap();
    assert!(status.success());
    assert!(!fixture.runtime.path().join("zacrs.sock").exists());
    fixture.shell(
        &installed,
        r#"
source "$ZACRS_PLUGIN"
(( _zacrs_daemon_available == 1 )) || exit 1
functions[$ZACRS_BIN]='print -u2 -- unexpected-helper-after-upgrade; return 1'
source "$ZACRS_PLUGIN"
(( _zacrs_daemon_available == 1 )) || exit 1
"#,
    );
    assert!(fixture.ensure().contains("edit_context=2"));
}

#[test]
fn stop_is_idempotent_and_stops_even_a_matching_daemon() {
    for action in ["stop", "retire"] {
        let fixture = Fixture::new();
        for running in [false, true, false] {
            if running {
                fixture.ensure();
            }
            assert!(
                fixture
                    .command()
                    .args(["daemon", action])
                    .status()
                    .unwrap()
                    .success()
            );
            assert!(!fixture.runtime.path().join("zacrs.sock").exists());
        }
    }
}

#[test]
fn running_daemons_are_reused_until_explicitly_stopped() {
    for handshake in [
        None,
        Some("OK enter_expansion=0 edit_context=2 space_expansion=0"),
        Some("OK build_id=old"),
    ] {
        let fixture = Fixture::new();
        let socket = fixture.runtime.path().join("zacrs.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let old = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "old daemon was not stopped");
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut line = String::new();
                BufReader::new(&stream).read_line(&mut line).unwrap();
                match line.as_str() {
                    "ping shell\n" => {
                        if let Some(reply) = handshake {
                            writeln!(stream, "{reply}").unwrap();
                        }
                    }
                    "ping\n" => stream.write_all(b"OK\n").unwrap(),
                    "shutdown\n" => {
                        stream.write_all(b"OK\n").unwrap();
                        drop(listener);
                        fs::remove_file(socket).unwrap();
                        break;
                    }
                    _ => panic!("unexpected request: {line}"),
                }
            }
        });
        let reply = fixture.ensure();
        assert_eq!(reply.trim(), handshake.unwrap_or("OK"));
        assert!(
            fixture
                .command()
                .args(["daemon", "stop"])
                .status()
                .unwrap()
                .success()
        );
        old.join().unwrap();
    }
}

#[test]
fn replacement_listener_does_not_interrupt_an_open_popup() {
    let fixture = Fixture::new();
    fixture.ensure();
    let mut stream = fixture.connect();
    stream
        .write_all(
            b"complete 2 2 100 24 candidates_present=1\n\nalpha\t\tcommand\nbeta\t\tcommand\nEND\n",
        )
        .unwrap();
    let mut reader = BufReader::new(stream);
    let mut header = String::new();
    reader.read_line(&mut header).unwrap();
    let frame = TextFrameHeader::parse(&header).unwrap();
    let mut bytes = vec![0; frame.tty_len];
    reader.read_exact(&mut bytes).unwrap();

    let mut shutdown = fixture.connect();
    shutdown.write_all(b"shutdown\n").unwrap();
    let mut reply = String::new();
    BufReader::new(shutdown).read_line(&mut reply).unwrap();
    assert_eq!(reply, "OK\n");
    let deadline = Instant::now() + Duration::from_secs(2);
    while fixture.runtime.path().join("zacrs.sock").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    fixture.ensure();
    // The original socket still belongs to the old process and accepts input.
    reader.get_mut().write_all(b"KEY 1\n\x1b").unwrap();
    let mut result = String::new();
    reader.read_to_string(&mut result).unwrap();
    assert!(result.contains("DONE 1"), "{result:?}");
}
