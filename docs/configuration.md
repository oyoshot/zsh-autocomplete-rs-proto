# Configuration

The configuration file is read from `$XDG_CONFIG_HOME/zacrs/config.toml`, or
`~/.config/zacrs/config.toml` when `XDG_CONFIG_HOME` is not set.

## Abbreviations

Static abbreviations are portable completion candidates. By default they are
offered only at command position. Expansions are inserted literally without
running shell evaluation while candidates are being gathered or displayed.

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
