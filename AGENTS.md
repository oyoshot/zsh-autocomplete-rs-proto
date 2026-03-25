# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Rust crate: `main.rs` wires the CLI subcommands (`complete`, `render`, `clear`, `daemon`), `lib.rs` re-exports modules, and core behavior lives in files such as `app.rs`, `candidate.rs`, `config.rs`, `fuzzy.rs`, `input.rs`, and `ui/`. Daemon and transport code lives in `client.rs`, `daemon.rs`, `protocol.rs`, and `tty.rs`. `shell/` holds the Zsh plugin entrypoint and helper scripts, all using the `_zacrs_*` prefix. `benches/` contains Criterion benchmarks, with shared fixtures in `benches/helpers/`. Treat `tmp/` as scratch or reference material, not primary source.

## Build, Test, and Development Commands
Use a workspace-local target dir when possible:

- `CARGO_TARGET_DIR=target cargo build` builds the binary at `target/debug/zsh-autocomplete-rs`.
- `CARGO_TARGET_DIR=target cargo test` runs the unit tests embedded under `src/`.
- `cargo fmt --check` verifies Rust formatting before review.
- `cargo clippy --all-targets --all-features -- -D warnings` matches the CI lint configuration.
- `cargo bench` runs Criterion benchmarks in `benches/`.
- `CARGO_TARGET_DIR=target cargo run -- complete --prefix gi --cursor-row 5 --cursor-col 2` performs a focused interactive popup check.
- `CARGO_TARGET_DIR=target cargo run -- render --prefix gi --cursor-row 5 --cursor-col 2` draws the popup once and prints popup metadata for follow-up `clear` testing.
- `CARGO_TARGET_DIR=target cargo run -- daemon start|status|stop` exercises the optional daemon flow used by the Zsh plugin.

For focused manual CLI checks, pipe tab-separated candidates into the `complete` or `render` subcommands.

## Coding Style & Naming Conventions
Follow standard Rust formatting with `rustfmt` and 4-space indentation. Use `snake_case` for functions, modules, and test names; `CamelCase` for types; `SCREAMING_SNAKE_CASE` for constants. Keep modules small and responsibility-focused, matching the current layout (`ui/render.rs`, `ui/popup.rs`, `ui/theme.rs`, etc.). In Zsh scripts, preserve the `_zacrs_*` function and file prefix pattern. When changing daemon IPC or popup metadata, update Rust and shell sides together so the protocol stays aligned.

## Testing Guidelines
Prefer unit tests colocated with the implementation under `#[cfg(test)]`. Name tests for observable behavior, for example `common_prefix_shared` or `new_filters_candidates`. Add or update tests for filtering, popup layout, config parsing, daemon protocol handling, and shell-facing edge cases whenever behavior changes. For UI or terminal-control changes, run a focused `cargo run -- complete` or `render` check in a real terminal, and verify the daemon path as well when touching `client.rs`, `daemon.rs`, `protocol.rs`, or the shell plugin.

## Commit & Pull Request Guidelines
Recent history follows scoped Conventional Commit prefixes such as `fix(shell): ...`, `feat(daemon): ...`, `refactor(ui): ...`, and `test: ...`. Keep subjects imperative and concise; add a scope when it clarifies the affected area. PRs should describe user-visible behavior, list verification steps run, and note any shell integration, daemon/socket, or terminal-specific risks. Include screenshots or terminal captures only when the popup rendering changes.
