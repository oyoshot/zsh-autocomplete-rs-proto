# Zsh integration ownership (#166)

Implemented against main `8db8fa755be2dde2fc6d8d500115d5601ae5a271`.
Automatic Space and Enter expansion remain optional. Rust keeps expansion and
selection policy; the adapter keeps native context and widget invocation.

## Problem and decision

FSH treats registered builtin aliases as canonical builtin names. The previous
`_zacrs_saved_self_insert` alias made it invoke nonexistent
`._zacrs_saved_self_insert`, breaking every inserted character. The old reload
recovery also rewrote foreign saved aliases and dispatcher bodies, because it
reused delegate storage while references to that storage were still alive.

Use an exact ZLE capture under a dot-prefixed internal name and give the insertion
entry a stable function identity for the lifetime of its registered references.
A dot name follows the internal-widget convention used by the affected binders;
it is not an inaccessible namespace. The lifecycle, not the name alone, prevents
recursion and foreign-state repair on reload. No per-plugin callbacks are added.

A dedicated Space binding was rejected because native search readers inherit
bindings but reject user widgets. Pre-redraw alone was rejected because it is
not a one-event-per-key notification and can miss the final queued insertion.
The existing deferred-rendering boundary therefore remains, under the same
ownership rules as Space.

## Insertion ownership

`P` is public `self-insert`, `E` our entry function, `D` its exact private delegate,
and `F` another plugin's wrapper.

| Event | Operations |
| --- | --- |
| First attachment | Capture `P` as `D` using `zle -A`, then publish `E` at `P`. Builtin aliases, user widgets and completion widgets retain their actual ZLE identity. |
| Plugin before zacrs | `P=E -> D`, where `D` includes the earlier plugin. |
| Plugin after zacrs | `P=F -> saved E -> D`. Do not alter `F` or its saved entry. |
| Reload | Refresh our bodies/configuration, keeping active `E`, `D`, and the public entry. No new wrapper generation. |
| Disable / drained batch, public entry ours | Restore `D` at `P` only when Space is off and no other registered widget references `E`. This attachment can then be reused. |
| Registered reference to `E` survives | Keep `E` and `D`, even after disabling. Cache this decision to avoid repeated alias scans. |
| Re-enable with retained entry | Reactivate the same `E`; never recapture the outer wrapper as `D`. |
| Re-enable after unshared retirement | Capture again only if public metadata still matches our restored implementation. Builtin metadata is opaque; exact capture preserves its current implementation. |
| Independent public replacement | Leave it alone. An active retained entry may become unreachable. Reload does not reclaim it. |

Space-off bodies contain the existing batching delegate without Space checks.
A fresh disabled shell installs no Space wrapper. Typeahead can temporarily
attach the batching entry, and a plugin capturing that entry extends its lifetime
in the same way as an enabled Space entry. This is an explicit qualification of
zero disabled-feature overhead after live integration changes.

Captured registered user widgets retain their function identity across reloads;
our function body changes only within the same invocation contract. Supported
capture interfaces are `zle -A` and registered user widgets using `zle -N`.
Direct invocation or function-body copies of private `_zacrs_*` functions are
outside that contract. Their reachability cannot be established from ZLE state.
No foreign aliases are deleted, retargeted or renamed.

Ordinary insertion and expansion separators still delegate exactly once with
the existing ZLE flags and status. Cursor markers consume the trigger without
insertion. Native numeric counts, suffixes, search readers, recursive editors,
unknown syntax, popup invalidation and suggestion cleanup keep their existing
behavior. No IPC or Rust protocol changes are needed.

## Hooks and reload

Register fixed line-init and pre-redraw callbacks once. The line-init callback
conditionally resets the editing session alongside unconditional popup reset,
so toggling expansion never removes and readds dispatcher entries.

`add-zle-hook-widget` captures a nonstandard public entry before checking callback
duplicates. Re-registering after another plugin wraps that dispatcher can create
`dispatcher -> wrapper -> dispatcher`. On first attachment with such an existing
dispatcher alias, append only our callback to its standard `widgets` style,
preserving the existing entries, dispatcher function and public wrapper.
Otherwise use the standard helper. Reload performs neither registration path.

There are no observer callbacks, frozen dispatcher generations or fallback
publishers. A failed callback keeps the standard dispatcher's early return.
Removal of our callback or independent replacement of a public hook is respected,
including across reloads. Later registrations by other plugins still need to
respect the standard helper's limitation; zacrs does not patch their dispatcher.
Accept/cancel widgets and Tab are installed initially and preserved on reload.

The former tests requiring automatic reclamation of independently replaced hooks
are intentionally superseded by ownership checks. Tests retain exactly-once
order, cancellation/failure status, recursive editing and saved-chain coverage.
Upgrade from the old mutable-delegate adapter requires a fresh shell: an early
source guard refuses the mixed lifecycle before changing helpers or widgets.

## Validation

On Linux / Zsh 5.9.2, the old adapter reproduces #166 with actual FSH loaded after
it: `No such widget '._zacrs_saved_self_insert'`. The new adapter passes the same
fixture, including before/after load order, repeated reload, disable/re-enable,
retained aliases, ordinary insertion and marker offsets 0/middle/end. Actual
FSH highlighting and actual autosuggestions state are checked. These plugin
fixtures isolate transport; the existing daemon-backed suite separately covers
native editing/context and real expansion policy, including compiled adapters.

Actual plugin source SHA-256 values used locally:

- FSH: `19167193747abbbf68d5b4248b2d2e3b90777d910b1f4c5f1918bf227676b03c`
- autosuggestions v0.7.1: `eec7ba8f7a71414ace0ea0fab0908d005b24cf65d83b169c0ff97815d3cfc51a`

Run `shell/tests/insertion_ownership.zsh` with the plugin arguments documented in
[development.md](../development.md) to repeat these checks. The default shell
suite uses a representative binder reproducing the exposed-builtin-alias error,
without downloading plugins. Native macOS execution remains a CI check; it was
not performed locally.

## Performance

Measured with `dev/bench-zle.py`, one debug binary built by Rust 1.99.1, Zsh 5.9.2,
Ryzen 7 9800X3D, Linux 6.18.58.1 / WSL2. Baseline shell files are from `8db8fa7`.
Each state has 3 warmups and 15 measured shells per revision; baseline/candidate
alternate within each repetition. HOME, configuration and runtime are private.
Daemon startup is excluded: all startups reuse the same binary's daemon.
Plain adapters only; compiled startup and real-plugin latency were not benchmarked.

Startup ends at a real line-init readiness marker and includes compinit. Queued
workloads measure PTY write through an ending ZLE probe, including terminal output
and control-key overhead. Ordinary input uses 1,000 characters plus Ctrl-U;
Space workloads use 100 repetitions of `"gs" Space Ctrl-U` or `gs Space Ctrl-U`.
The deferred batch explicitly arms the existing batching integration before
1,000 characters. Completion drawing is suppressed; these measurements do not
establish request-to-visible-candidate latency. Hot-disabled startup includes
source, a representative outer wrapper capture, disabling config, and reload.

Values below are median [second-smallest, second-largest] of 15 samples, a rough
10th–90th percentile range. Key workloads are microseconds per character for
ordinary/batch input and per entire sequence for Space workloads.

| State | Workload | Main | Changed |
| --- | --- | --- | --- |
| off | Startup (ms) | 97.15 [95.51, 103.50] | 96.82 [95.46, 100.68] |
| off | Ordinary (µs/char) | 102.70 [101.39, 105.84] | 103.41 [102.13, 106.24] |
| off | Quoted Space sequence (µs) | 592.45 [578.76, 648.07] | 598.04 [586.06, 615.74] |
| off | Expansion sequence (µs) | 394.97 [388.84, 413.36] | 396.47 [387.67, 409.97] |
| off | Deferred batch (µs/char) | 124.04 [122.67, 129.20] | 124.22 [122.29, 130.14] |
| on | Startup (ms) | 98.27 [97.02, 100.72] | 97.75 [96.57, 103.08] |
| on | Ordinary (µs/char) | 134.94 [132.34, 139.90] | 136.23 [133.82, 140.60] |
| on | Quoted Space sequence (µs) | 1270.57 [1243.55, 1320.19] | 1299.64 [1256.58, 1323.48] |
| on | Expansion sequence (µs) | 1525.75 [1502.90, 1555.25] | 1552.25 [1512.70, 1603.18] |
| on | Deferred batch (µs/char) | 137.91 [133.80, 143.00] | 139.42 [134.93, 143.79] |
| hot-disabled | Startup (ms) | 99.93 [97.54, 102.74] | 102.39 [98.60, 117.78] |
| hot-disabled | Ordinary (µs/char) | 118.71 [117.29, 122.08] | 143.30 [140.36, 147.32] |
| hot-disabled | Quoted Space sequence (µs) | 681.51 [661.93, 707.14] | 796.02 [787.51, 831.76] |
| hot-disabled | Expansion sequence (µs) | 440.59 [433.68, 469.03] | 513.08 [504.73, 532.88] |
| hot-disabled | Deferred batch (µs/char) | 142.59 [139.56, 147.60] | 142.96 [140.09, 150.25] |

Enabled ordinary input changes by about 1.3 µs/character and expansion sequences
by about 27 µs, with overlapping ranges. Captured-but-disabled forwarding costs
about 25 µs/character compared with main's alias-rewriting detach. This retained
cost is deliberate: a fresh shell removes it without modifying foreign state.
No general speedup or candidate-display latency improvement is claimed.

## References

- [Issue #166](https://github.com/oyoshot/zsh-autocomplete-rs-proto/issues/166)
- [ZLE widget contracts](https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html)
- [FSH binding at the issue revision](https://github.com/zdharma-continuum/fast-syntax-highlighting/blob/4672ad5dd9ad68a7effc1476d65afb7c584ce2b3/fast-syntax-highlighting.plugin.zsh#L242)
- [Autosuggestions delegation](https://github.com/zsh-users/zsh-autosuggestions/blob/master/src/bind.zsh)
