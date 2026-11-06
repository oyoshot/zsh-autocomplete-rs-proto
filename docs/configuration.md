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
daemon reloads the file when its modification time changes.

### Optional expansion on Enter (Zsh)

To expand an exact command abbreviation when accepting a line, add this to
`config.toml`:

```toml
[completion]
expand_abbreviation_on_enter = true
```

This replaces the former `ZACRS_ABBR_EXPAND_ON_ENTER` shell variable, which is
no longer read. Enable it before loading the plugin, or reload the plugin after
changing this setting. The shell reads the setting once at initialization;
when disabled, Enter does not tokenize input, launch a helper, or track selections.

It is disabled by default. With `gs = "git status"`, typing `gs` then Enter
rewrites the input to `git status` and executes it; `gs -sb` becomes
`git status -sb`. History records the expanded command. A same-named executable
or alias is overridden by the configured abbreviation in this mode. Space
keeps its ordinary behavior.

Only a bare command word in a single simple command is eligible. Partial
triggers, quoted/escaped words, paths such as `./gs`, assignment prefixes,
command modifiers such as `command gs`, and compound lines such as `gs | cat`
or `gs &! echo done`, and function declarations such as `gs() print hello`,
are left as typed. Continuation input (including heredoc
bodies and multiline quoted strings) is also left unchanged.
Command-position and `when.command` conditions still apply;
argument-position abbreviations continue to require popup selection.

An expansion with `{{cursor}}` replaces the command word and leaves the line in
the editor at the marker instead of executing it. Arguments and surrounding
whitespace are preserved. Editing the resulting arguments and pressing Enter
executes that prepared command without expanding it again, even if its name is
also an abbreviation trigger (for example, `ssh = "ssh user@{{cursor}}"`).
Expansion happens once per Enter, and a candidate
explicitly selected in the popup is accepted without a second expansion.
This also applies after selecting with Space and editing arguments; editing the
command word or starting a new input line clears that exemption.
Closing the popup with Space after typing a filter without selecting a candidate
keeps that typed command eligible for Enter expansion.

When enabled, the shell sends input and Zsh lexical tokens to Rust on Enter.
Rust decides whether to edit or accept the line. Rust also updates a bounded,
opaque selection state on popup application or command-word changes; argument
edits do not launch a state helper. Zsh holds no growing edit history.
It reads the current configuration independently of the daemon, so it also
works with the daemon stopped. A failed lookup accepts the original input.
Lookup only returns literal text; shell expressions in the resulting line are
evaluated by Zsh when the line is executed.
