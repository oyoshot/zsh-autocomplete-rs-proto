# Repository Guidelines

## Design Philosophy

This project began after experience with zsh-autosuggestions exposed unwanted shell side effects, conflicts with other plugins, and overhead in both Zsh startup and completion. Minimizing shell interference, preserving plugin coexistence, and keeping startup and completion fast are core goals.

Rust is intentional: implementing everything in Zsh is difficult to maintain within its language and plugin ecosystem. Keep reusable behavior in a testable Rust engine, with thin adapters for shell-specific integration. Zsh is the current adapter; the project name does not limit future shell support.

- Keep completion, ranking, configuration, UI, and reusable editing policy in Rust. Keep shared models and protocols independent of Zsh syntax and APIs.
- Let each adapter establish syntax and editing context through its shell's native facilities. Moving a handwritten Zsh parser into Rust does not make it portable. If context cannot be verified, preserve the input. See `docs/shell-adapters.md`.
- Minimize shell side effects, not just shell code size. Prefer predictable behavior and explicit state. Reuse existing integration where practical; justify additional hooks, widgets, option changes, and buffer or cursor manipulation by the behavior they enable.
- Scope shell option changes and restore borrowed state. Handle cancellation, reloads, and failures without leaking changes to buffers, cursors, widgets, or other plugins. Preserve explicit user choices.
- Treat startup and completion overhead as design constraints. Avoid unnecessary per-keystroke work, synchronous helper processes, and IPC. Check performance when adding work to these paths; using Rust alone does not eliminate integration overhead.
- Keep abbreviation definitions declarative. Gathering and displaying them must not evaluate their expansions or install hooks. Additional shell integration, such as automatic expansion, must be explicit and optional; disabled features should add no feature-specific work to input handling.

## Repository Map

`src/` contains the Rust engine, CLI, daemon, and protocol; `shell/` contains the Zsh adapter. Tests live in `tests/` and `shell/tests/`, benchmarks in `benches/`, and disposable sessions in `dev/`. Treat `tmp/` as scratch, not primary source. See `docs/shell-adapters.md` for adapter boundaries, `docs/configuration.md` for behavior, and `docs/development.md` for native and container verification.

## Agent Workflow

- Read the relevant implementation, tests, and design documents before changing behavior. Verify current responsibilities in code.
- Prefer the smallest coherent change that satisfies the requirement without accumulating avoidable architectural debt.
- Update Rust and shell together when changing IPC, popup metadata, or reuse-token handoff; keep protocols and cached state aligned.
- Preserve regression coverage and fallback behavior. If a test expectation or fallback must change, explain the intended behavior and why the previous expectation was wrong or is superseded; do not relax checks merely to pass tests.
- Report what was verified, relevant checks not completed and why, and remaining integration or performance uncertainty. Distinguish environmental restrictions from test failures.

## Commands and Style

Prefer `CARGO_TARGET_DIR=target`. Native integration tests require Zsh on `PATH`, PTYs, and Unix sockets.

- `cargo build`: build the debug binary.
- `cargo test`: run Rust unit and integration tests.
- `sh shell/tests/run.sh`: run shell regressions with temporary application directories.
- `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`: match CI formatting and lint checks.
- `cargo bench --bench <name>`: run the relevant benchmark from `benches/`; CI tracks `fuzzy_bench` and `text_processing_bench`.
- `dev/shell`: open a disposable manual session using Podman or Docker; restart after source edits.

Follow rustfmt, Rust naming conventions, and existing shell indentation. Use `_zacrs_*` for shell internals. Keep modules responsibility-focused and name tests for observable behavior. CI also checks spelling and unused dependencies.

## Verification

- For code changes, run focused regressions, then `cargo test`, shell regressions when shell behavior is affected, formatting, and Clippy. Documentation-only changes need content, command/link accuracy, and whitespace checks.
- For shell editing or context changes, add regressions using real ZLE, covering relevant cursor positions and state restoration. For lifecycle changes, cover cancellation, reload, and failure paths. Automated PTY-driven ZLE tests qualify; CLI tests and mocked widgets alone do not establish ZLE correctness.
- For hooks, widgets, or option changes, check coexistence with the affected plugin behavior or a representative competing hook/widget, including load order where relevant. Isolate configuration and runtime state with temporary directories.
- Verify the daemon path for shell/protocol changes. For terminal/UI changes, check relevant resizing, cancellation, and Unicode behavior in a real terminal. If PTY, socket, or interactive verification is unavailable, report the exact unverified behavior and reason.
- When adding work to startup, completion, or input handling, compare the affected paths before and after under the same build profile, machine, shell/plugin configuration, and workload. Measure Zsh startup to input readiness, completion request to visible candidates, or added ZLE key-handling latency as applicable. Distinguish daemon startup from reuse; record the baseline revision, measurement method, repetitions, variability, and any regression. Use relevant existing benchmarks, but do not treat Rust benchmark success as proof of shell performance or claim improvements without measurements.

## Commits and Pull Requests

Use scoped Conventional Commit subjects such as `fix(shell): preserve selected commands`. Keep commits focused. PRs should describe the problem, resulting behavior, validation and its limits, and relevant issues. Update affected documentation and describe added shell interference or performance costs. Include terminal captures for visible popup changes.
