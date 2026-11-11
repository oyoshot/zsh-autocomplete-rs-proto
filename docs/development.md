# Development

## Start a manual session

With a running Podman or Docker engine:

```sh
dev/shell
```

This builds the current checkout in a Debian 13 / Rust 1.98.0 container and opens
Zsh as a non-root user. After installing the binary, the launcher stops any
existing daemon in the disposable session. Zsh loads the built binary through
`zsh-autocomplete-rs init zsh`, which starts the session's daemon. The host needs
no Rust or Zsh installation. The first run downloads the image and dependencies.

On macOS, start Docker Desktop or a Podman machine first. The engine must be able
to mount the checkout and the host temporary directory. The container runs Linux
on either host; testing the macOS binary still requires native macOS checks.

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

`dev/shell` exercises one Linux/glibc userland.
Containers share the host kernel and relay the terminal through a PTY, so macOS,
kernel/WSL differences and terminal-specific behavior need native checks.
Release portability requires testing the same release binary across the
intended runtimes, rather than rebuilding it in each container.

When a reload finds a wrapped ZLE hook, zacrs preserves its delegation through
a fallback while keeping the public `azhw:zle-*` dispatcher identity recognized
by `add-zle-hook-widget`. This keeps later standard registrations from capturing
the fallback as a dispatcher child. Zacrs saves the native dispatcher function,
then updates that hook's dispatcher body; aliases saved by other plugins are
rebound to immutable copies of the previous body before each update.
Captured hooks are recognized by their registered alias as well as their current
widget implementation, so wrapping a captured alias does not add a second path
to its original hook. A small observer at the start of the native dispatcher
body counts invocations by hook name and ZLE recursive editing depth. It runs
before user callbacks, even when a callback fails and a surrounding wrapper
handles that failure. Observation is independent of the callback list, so
standard hook-list resets and later registrations remain safe without another
zacrs reload. Removing callbacks does not reinstall them.
Each wrapper compares the count before and after its delegate; only an unchanged
count triggers a fallback. Nested wrappers share increments, while successive
events and recursive editors remain independent. Reload removes legacy observer
callbacks and upgrades saved native dispatcher bodies from older versions.
The upgrade check matches Zsh's tab-indented function representation so repeated
reloads keep one observer call per native dispatch.
The observer uses the public widget name so another hook cannot mark this
invocation as dispatched. This avoids recursion through active aliases and lost
completion through stale aliases. It adds shell function calls on wrapped hook invocations,
but no helper process or IPC. Delegates remain immutable so other plugins can
save and wrap them across further reloads. Delegated widget calls use `nolast`
to preserve `LASTWIDGET` for downstream hooks. The competing wrapper still
surrounds its original dispatcher calls.
