use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Stage the public shell files outside the checkout, as a package would.
pub fn install_zsh_plugin(root: &Path, compiled: bool) -> PathBuf {
    let directory = root.join("share/zsh-autocomplete-rs");
    fs::create_dir_all(&directory).unwrap();
    for name in [
        "zsh-autocomplete-rs.plugin.zsh",
        "_zacrs_util.zsh",
        "_zacrs_gather.zsh",
        "_zacrs_compsys.zsh",
        "_zacrs_protocol.zsh",
    ] {
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("shell")
                .join(name),
            directory.join(name),
        )
        .unwrap();
    }
    if compiled {
        let output = Command::new("zsh")
            .args([
                "-dfc",
                "for file in \"$1\"/*.zsh; do zcompile -U \"$file\" || exit; done",
                "_",
            ])
            .arg(&directory)
            .env_clear()
            .env("PATH", env::var_os("PATH").unwrap_or_default())
            .env("HOME", root)
            .env("ZDOTDIR", root)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
    directory.join("zsh-autocomplete-rs.plugin.zsh")
}
