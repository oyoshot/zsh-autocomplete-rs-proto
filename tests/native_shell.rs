use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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
    let output = Command::new("zsh")
        .args([
            "-d",
            "-f",
            "-i",
            "-c",
            r#"trap 'zsh-autocomplete-rs daemon stop >/dev/null 2>&1' EXIT
autoload -Uz compinit
compinit
eval "$(zsh-autocomplete-rs init zsh)"
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
        .current_dir(root.path())
        .output()
        .expect("native shell smoke test requires Zsh on PATH");
    assert!(
        output.status.success(),
        "Zsh initialization failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
