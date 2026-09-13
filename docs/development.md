# Development

## Start a manual session

On Linux, with Podman or Docker installed:

```sh
dev/shell
```

This builds the current checkout in a Debian 13 / Rust 1.98.0 container and opens
Zsh as a non-root user. Zsh loads the built binary through
`zsh-autocomplete-rs init zsh`, which starts the session's daemon. The host needs
no Rust or Zsh installation. The first run downloads the image and dependencies.

Podman is selected when available. Use `CONTAINER_ENGINE=docker dev/shell` to
select Docker, or `dev/shell --release` for a release build. With Docker, the
checkout must be readable by container UID 1000; rootless Podman maps your user
to the container user.

The launcher passes `TERM` and optional `COLORTERM`. If host `infocmp` is available,
it also imports the terminal description. Unknown terminal entries cause an
error; install host `infocmp` if your terminal is not covered by the image.

## Session boundaries

| Input or state | Location / lifetime |
| --- | --- |
| Checkout, including uncommitted files and project Cargo configuration | `/src`, read-only bind mount |
| Installed binary | Private snapshot of the successful startup build |
| HOME, shell settings/history, zacrs state, `/tmp`, `/run` and daemon | Disposable per session |
| Sample Git project | `~/work`, writable and disposable |
| Cargo dependency caches | Persistent container volumes |
| Cargo target cache | Persistent volume per image ID and canonical checkout path |

Your personal HOME, installed zacrs and daemon socket are not mounted. Multiple
sessions can run alongside your normal shell. Start a new session after editing
source to use the new build; a failed build does not open a shell with an old binary.

Exit with `exit` or Ctrl-D to discard the container and its runtime state. Build
caches remain. After a terminal disconnect or runtime crash, inspect leftovers
with `podman ps --filter label=dev.zacrs.manual=true` (substitute `docker` as needed)
and stop the specific container.

The launcher disables SELinux label separation for this container to read bind
mounts without relabeling host files. Use it with trusted checkouts; it is not a
security sandbox for malicious code.

## Manual checks

Edit configuration inside the session. To try the abbreviation examples:

```sh
cp /opt/zacrs/fixtures/config.toml ~/.config/zacrs/config.toml
```

- Type `gs`, open completion with Tab, and check Enter and Space against
  [configuration.md](configuration.md).
- Try file completion in `~/work`, including spaces and Japanese filenames,
  `git switch`, directory changes, Ctrl-C and resizing with a popup visible.
- Add files or branches to check cache invalidation.
- Open two sessions and verify that configuration changes and stopping one
  daemon leave the other session and the host unaffected. Restart a session to
  verify its configuration, history and test files were discarded.

For a scripted startup check:

```sh
dev/shell -- -c 'zsh-autocomplete-rs daemon status; cargo --version'
```

Without a terminal this does not allocate a PTY or test interactive completion.

## Automated tests and platform coverage

Run native checks with Rust and Zsh installed:

```sh
CARGO_TARGET_DIR=target cargo test
sh shell/tests/run.sh
cargo fmt --all --check
CARGO_TARGET_DIR=target cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` includes a Zsh initialization and daemon smoke test;
`shell/tests/run.sh` runs shell regression tests with temporary application
directories. CI runs these checks on Linux and macOS.

`dev/shell` currently supports Linux hosts only and exercises one glibc userland.
Containers share the host kernel and relay the terminal through a PTY, so macOS,
kernel/WSL differences and terminal-specific behavior need native checks.
Release portability requires testing the same release binary across the
intended runtimes, rather than rebuilding it in each container.
