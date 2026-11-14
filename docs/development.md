# Development

## Start a manual session

With a running Podman or Docker engine:

```sh
dev/shell
```

This builds the current checkout in a Debian 13 / Rust 1.98.0 container and opens
Zsh as a non-root user. After installing the binary, the launcher stops any
existing daemon in the disposable session. The launcher also installs and
compiles the shell files. Zsh sources the installed plugin, which starts the
session's daemon. The host needs no Rust or Zsh installation. The first run
downloads the image and dependencies.

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
| Shell adapter | Private compiled snapshot from the same checkout |
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

Dependabot enables auto-merge for patch and minor updates after checking every
updated dependency. Cargo updates must stay within the same major version, or
the same minor version for `0.x` crates; `0.0.x`, prerelease, and unrecognized
Cargo versions require manual review. Major updates also require manual review.
The `main` branch must require the seven CI checks (Spell Check, Format, Clippy,
Unused Deps, MSRV Check, and Test on both Linux and macOS) so auto-merge waits for
successful validation. Benchmark results remain advisory. The automation runs
when a Dependabot PR is opened, reopened, or updated.

At each ZLE line-init, zacrs discards popup coordinates and snapshots, resets
the previous input and suppression state, and restores a deferred self-insert
delegate. Terminal interrupts can bypass both send-break and line-finish. The
new prompt is already on screen at line-init, so this reset emits no terminal
erase sequences. This callback runs even when Enter abbreviation expansion is
disabled; the same callback conditionally resets the selection session. The reset
adds shell function calls once per editing line, with no helper process or IPC.

Zacrs registers fixed line-init and pre-redraw callbacks once. Reload updates
owned function bodies and configuration without re-registering callbacks,
changing dispatcher bodies, or rewriting aliases captured by another plugin.
The public insertion widget, accept/cancel wrappers, and user Tab bindings are
preserved across reloads. Removing a callback or independently replacing a
public hook is respected; reload does not reclaim that entry.

When a standard dispatcher is already behind a foreign wrapper on first load,
zacrs appends only its callback to that dispatcher's `widgets` style, preserving
existing entries and the public widget. Calling `add-zle-hook-widget` on that
public wrapper would capture the dispatcher into itself. Later registrations by
other code must also respect that standard-helper limitation; zacrs no longer
patches the dispatcher to compensate for arbitrary subsequent re-registration.
A failing callback retains the standard dispatcher's early-return behavior.

Insertion captures an exact ZLE delegate under a dot-prefixed internal name.
Builtin aliases are not assumed to be canonical `.self-insert`. Captured entries
keep the same function identity and delegate across reloads and toggles. When
zacrs still owns the public entry, it restores that entry only if no other
registered widget references its entry function. Such an unshared attachment
can then be reused for another batch. After observing a saved reference, it
retains forwarding state until a fresh shell, avoiding repeated widget scans.
Direct calls or function-body copies of private `_zacrs_*` functions are not a
supported capture interface; ZLE aliases and registered user widgets are.
Independent public replacements are preserved. Upgrading the previous adapter's
mutable live captures requires a fresh shell; the new source returns before
altering an old installation.

The lifecycle regression can also run against actual plugins:

```sh
zsh -df shell/tests/insertion_ownership.zsh fsh /path/to/fast-syntax-highlighting.plugin.zsh
zsh -df shell/tests/insertion_ownership.zsh autosuggestions /path/to/zsh-autosuggestions.zsh
```

These tests isolate transport while using real ZLE and plugin UI state; the
abbreviation integration suite separately uses the actual daemon. For adapter
performance comparisons, retain the baseline's entire shell directory, build one
binary, then run:

```sh
CARGO_TARGET_DIR=target cargo build --release
CARGO_TARGET_DIR=target cargo bench --bench zle_bench -- \
    /path/to/baseline/shell/zsh-autocomplete-rs.plugin.zsh
```

This uses isolated HOME/config/runtime directories, a reused daemon, and plain
adapters. It records startup and queued-key workload latency with completion
drawing suppressed, including terminal output and control-key overhead. It does
not measure request-to-visible-candidate latency or cold daemon startup.
The harness uses the existing Rust dependencies, Zsh, and Git; Python is not
required. Its default binary is `target/release/zsh-autocomplete-rs`; select
another build with `--binary`. Building the harness is outside all timed intervals.
Omitting the baseline uses the current adapter for both variants as a smoke
check. Supply a baseline from another revision for a performance comparison.

### Shell startup comparisons

Use the startup workload to measure adapter loading separately from input
handling. Build one binary and use it for both compatible adapter revisions:

```sh
CARGO_TARGET_DIR=target cargo build --release
CARGO_TARGET_DIR=target cargo bench --bench zle_bench -- \
    /path/to/baseline/shell/zsh-autocomplete-rs.plugin.zsh \
    --workload startup --binary target/release/zsh-autocomplete-rs \
    --samples 500 --warmups 100 --expansion enter --format compiled \
    --output /tmp/zacrs-startup.toml
```

Each repetition compares no adapter, the baseline, and the current checkout in
rotating order. The harness stages private copies of the shell sources. With
`--format compiled`, it compiles all files using the same Zsh that runs the
samples; existing bytecode in either checkout is ignored. Use `--zsh /path/to/zsh`
to select it explicitly. The defaults measure both plain and compiled sources
and all four expansion configurations: off, Enter only, Space only, and both.

The common `compinit` dump is generated before timing. Fresh interactive shells
load it with `compinit -C`. `source_ms` covers just the source call, timed inside
Zsh with `EPOCHREALTIME`; the no-adapter case measures the empty conditional.
`ready_ms` covers PTY creation through the final line-init marker,
including zacrs's selection reset when expansion is enabled. Connection and
capability checks run after source timing and before readiness; failed
initialization aborts the run instead of producing a fast sample. The fixture
loads synchronously and uses no other plugins or user dotfiles. It does not
measure deferred loading, the first completion, or a full personal shell setup.
The PTY size is fixed at 80 columns by 24 rows. Compare revisions with this same
harness; absolute timings from the former Python driver are not interchangeable.

The default `--daemon reused` starts the private daemon before timing. Use
`--daemon cold` to start without a running daemon for every sample, or
`--daemon both` to compare both lifecycles. Stopping the daemon is outside the
timed interval. "Cold" does not flush filesystem caches. Both adapters use the
same binary; comparing incompatible protocol revisions requires a separate
end-to-end experiment with matching binaries.

TOML includes raw samples, mean, standard deviation, median, 10th/90th
percentiles, and paired differences between adapters and against no adapter.
It also records the command, platform, Zsh version, Git blob hashes of the binary
and sources, and adapter revisions when the sources are in Git checkouts. For exported
sources, retain the revision alongside the result. Keep the build profile and
machine information with the result. Compare repeated runs under similar
machine load before drawing conclusions from small differences. These shell
measurements are manual;
the CI benchmark workflow currently tracks only Rust fuzzy/text processing.
