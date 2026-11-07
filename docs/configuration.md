# Configuration

The configuration file is read from `$XDG_CONFIG_HOME/zacrs/config.toml`, or
`~/.config/zacrs/config.toml` when `XDG_CONFIG_HOME` is not set.

## Abbreviations

Static abbreviations are portable completion candidates. By default they are
offered only at command position. Expansions are inserted literally without
running shell evaluation while candidates are being gathered or displayed.

Abbreviations use the same matching score as other candidates, without an extra
priority boost. For equal scores and text lengths, files, directories, and
commands precede abbreviations. With an empty query, candidates are ordered by
kind, so those native candidates also appear before abbreviations.

For simple entries, use the `[abbreviations]` table:

```toml
[abbreviations]
gs = "git status"
```

For a description or explicit cursor placement, use `[[abbreviation]]`:

```toml
[[abbreviation]]
trigger = "gcm"
expansion = "git commit -m '{{cursor}}'"
description = "commit with a message"
```

Shell fragments can use the same format:

```toml
[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
description = "discard stdout"
when.position = "any"
```

`when.position = "any"` also offers the abbreviation at argument positions. Typing
`cargo test <Tab>` can then show `null`, and `cargo test null<Tab>` can replace
`null` with `>/dev/null`. The supported positions are `command` (the default)
`argument`, and `any`.

Use `when.command` to restrict an abbreviation to the current simple-command
context. It accepts one glob or a list of alternative globs:

```toml
[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
description = "discard stdout"
when.position = "argument"
when.command = ["cargo *", "cross *", "git add *"]
```

The shell normalizes parsed words to one space and matches only the simple
command containing the cursor, so `echo ok | cargo test null` is matched as
`cargo test null`. Matching is case-sensitive and anchored to the entire
context. Patterns support `*`, `?`, and character classes such as `[abc]`.
They are compiled when configuration is loaded and are never expanded or
executed by the shell. Position and command conditions are combined with AND;
entries in the command list are combined with OR.

`{{cursor}}` is removed during insertion and places the cursor at that position.
Only one cursor marker is supported per expansion.

The popup description always starts with the literal expansion preview. When a
custom `description` is configured, it is shown after the preview.

When an abbreviation is selected in the popup:

- Enter expands it and accepts the command line.
- Space expands it, appends a space, and keeps editing the command line.
- Cancel leaves the command line unchanged.

Defining abbreviations does not install key bindings, widgets, hooks, aliases,
functions, or traps. The shell adapter remains responsible for presenting and
applying completion results.

Both the daemon and subprocess paths read the same configuration. A running
daemon checks the file's modification time and size on existing requests and
applies changes to subsequent completion sessions. An interactive popup retains
the settings it started with. There is no file watcher, additional shell hook,
or per-keystroke configuration request.

If the file contains invalid TOML, cannot be read, or temporarily disappears while
being saved, the daemon keeps the last successfully loaded settings. Invalid TOML
is logged once per file version and retried when the file changes; I/O failures
are retried on subsequent requests. Without a successfully loaded file, defaults
are used. To restore defaults, save an empty file. Removing the file keeps the
running daemon's current settings until it restarts or a valid file reappears.

Abbreviations are added to native candidates using the current configuration on
each request, including requests that reuse cached native candidates. Reloading
does not change their matching or ranking rules. The Zsh initialization setting
for optional Enter expansion below still requires a plugin reload.

### Optional expansion on Enter (Zsh)

To expand an exact command abbreviation when accepting a line, add this to
`config.toml`:

```toml
[completion]
expand_abbreviation_on_enter = true
```

This replaces the former `ZACRS_ABBR_EXPAND_ON_ENTER` shell variable, which is
no longer read. Reload the plugin after changing this setting. Configuration is
received through the daemon's startup handshake. When disabled, no abbreviation
editing session is registered and Enter sends no abbreviation request.

It is disabled by default. With `gs = "git status"`, typing `gs` then Enter
rewrites the input to `git status` and executes it; `gs -sb` becomes
`git status -sb`. History records the expanded command. A same-named executable
or alias is overridden by the configured abbreviation in this mode. Space
keeps its ordinary behavior.

The Zsh adapter obtains a command-word range from the native completion parser.
It offers that range only when the parser's command words cover the complete
lexical input and native literal quoting leaves the word unchanged. Native
reserved words are excluded. Partial triggers, quoted/escaped words, command
words containing substitutions or patterns, assignment prefixes, command lists,
function declarations, and continuation input are left as typed. Variables in
arguments do not prevent expansion. A path such as `./gs`
does not match the trigger `gs`. Redirections are currently left unchanged
because this query cannot verify their full context.
Command-position and `when.command` conditions still apply;
argument-position abbreviations continue to require popup selection.

An expansion with `{{cursor}}` replaces the command word and leaves the line in
the editor at the marker instead of executing it. Arguments and surrounding
whitespace are preserved. Editing the resulting arguments and pressing Enter
executes that prepared command without expanding it again, even if its name is
also an abbreviation trigger (for example, `ssh = "ssh user@{{cursor}}"`).
Expansion happens once per Enter, and a candidate
explicitly selected in the popup is accepted without a second expansion.
This also applies after selecting with Space and editing arguments. The choice
is compared with the command word when selecting another candidate or accepting
the line. Returning to the same selected word during the same input line keeps
its protection; intermediate keystrokes are not tracked. A different final word
or a new input line clears the exemption.
Closing the popup with Space after typing a filter without selecting a candidate
keeps that typed command eligible for Enter expansion.

When enabled, the daemon owns the editing session and selection protection.
Zsh sends facts only when applying a candidate or accepting a line, and resets the
session when ZLE starts a new input line. It holds no abbreviation selection state
or edit journal and sends no per-redraw abbreviation observations. These requests
use the existing Unix socket and launch no helper processes. The Zsh adapter uses
the existing completion widget in context-only mode to verify the target and supplies
its literal word and character range. Rust validates the range and performs
exact lookup and editing without parsing shell syntax. This widget runs only
when Enter expansion is enabled and calls no candidate providers. No extra
completion widget is registered. The query temporarily places the cursor at the
end of the command word, then restores the editor state before applying any edit.
Other shell adapters must establish their own native context capability; there is no
shared Zsh grammar heuristic.

Enter expansion requires a compatible running daemon. If communication fails,
the original input is accepted and expansion is disabled until the plugin is
reloaded. If the daemon restarts or loses the editing session, the current input
is left unchanged; a fresh line starts a new session. Literal shell expressions
in expansions are evaluated by Zsh only when the resulting line is executed.

### Daemon updates

Installers should run the newly installed executable with `daemon retire` after
replacing the binary. This stops an obsolete daemon, leaves a matching daemon
alone, and does nothing if none is running. It never starts a daemon in the
installer's environment; the next shell starts one with the user's environment.
For Home Manager this belongs in activation after package installation, rather
than the package build. Other package managers can use their post-upgrade hook.

At plugin initialization, Zsh's builtin `zstat` reads the installed executable's
device, inode, size and timestamps. The existing `ping shell` reply carries the
running daemon's executable identity. Matching identities reuse the daemon with
no helper process. A missing daemon or a mismatch invokes `daemon ensure`, which
also covers updates outside the installer, including local builds and symlink
changes. Neither the file check nor a helper process runs on normal keystrokes.

Build identity additionally includes the Rust and shell sources, dependency
lockfile and build settings. `daemon ensure` uses both build and executable
identity so switching between copies of one build also converges on the installed
executable. If `zstat` is unavailable, initialization uses the startup helper.

Concurrent shell initializations share a lock, so only one replaces the daemon.
During replacement, the old process releases its listening socket and lets open
completion sessions finish while the new process accepts connections. Daemons
from releases before this handoff support may close an open popup on replacement.
If the installed CLI predates `daemon ensure`, a locally sourced newer plugin
falls back to the existing daemon connection behavior.

See [the shell adapter boundary](shell-adapters.md) for the context contract.
