# Adapter prototypes

Throwaway experiments behind the "Other shells" table in
[`../../adapter-contract.md`](../../adapter-contract.md). Each binds Tab in one
shell, takes the word and candidates from that shell's own completion, runs the
unchanged engine (`zsh-autocomplete-rs complete`) and applies the reply.

They are not adapters. They ignore most reply codes, use the subprocess path
only, start a Python helper to read the cursor position where the shell cannot,
and have no tests. Nothing builds or ships them.

Run one with a release build of the engine:

```sh
CARGO_TARGET_DIR=target cargo build --release
SCENARIOS='[["git ch, Down, Enter", "git ch", ["\u001b[B", "\r"], true]]' \
    python3 docs/design/adapter-prototypes/drive.py docs/design/adapter-prototypes "$PWD" bash
```

The last argument is `fish`, `bash`, `pwsh`, `nu`, `xonsh` or `elvish`. The
bash prototype needs `/usr/share/bash-completion/completions/git`.
