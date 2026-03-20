# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Rust crate: `main.rs` wires the CLI, `lib.rs` re-exports modules, core behavior lives in files such as `app.rs`, `config.rs`, `fuzzy.rs`, and `ui/`. `shell/` holds the Zsh integration scripts and completion helpers. `benches/` contains Criterion benchmarks, with shared fixtures in `benches/helpers/`. `docs/` stores design notes and architecture analysis. `test-popup.sh` is the manual popup smoke test. Treat `tmp/` as scratch or reference material, not primary source.

## Build, Test, and Development Commands
Use a workspace-local target dir when possible:

- `CARGO_TARGET_DIR=target cargo build` builds the binary at `target/debug/zsh-autocomplete-rs`.
- `CARGO_TARGET_DIR=target cargo test` runs the unit tests embedded under `src/`.
- `cargo fmt --check` verifies Rust formatting before review.
- `cargo clippy` runs lint checks to catch common mistakes and suggest improvements.
- `cargo bench` runs Criterion benchmarks in `benches/`.
- `bash test-popup.sh` performs a manual terminal popup check after `cargo build`.

For focused manual CLI checks, pipe tab-separated candidates into `cargo run -- complete --prefix gi --cursor-row 5 --cursor-col 2`.

## Coding Style & Naming Conventions
Follow standard Rust formatting with `rustfmt` and 4-space indentation. Use `snake_case` for functions, modules, and test names; `CamelCase` for types; `SCREAMING_SNAKE_CASE` for constants. Keep modules small and responsibility-focused, matching the current layout (`ui/render.rs`, `ui/popup.rs`, etc.). In Zsh scripts, preserve the `_zacrs_*` function and file prefix pattern.

## Testing Guidelines
Prefer unit tests colocated with the implementation under `#[cfg(test)]`. Name tests for observable behavior, for example `common_prefix_shared` or `new_filters_candidates`. Add or update tests for filtering, popup layout, config parsing, and shell-facing edge cases whenever behavior changes. Run the manual popup script for UI or terminal-control changes.

## Commit & Pull Request Guidelines
Recent history follows scoped Conventional Commit prefixes such as `fix(shell): ...`, `refactor(ui): ...`, and `test: ...`. Keep subjects imperative and concise; add a scope when it clarifies the affected area. PRs should describe user-visible behavior, list verification steps run, and note any shell integration or terminal-specific risks. Include screenshots or terminal captures only when the popup rendering changes.
