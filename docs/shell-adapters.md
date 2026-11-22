# Shell adapter boundaries

The shared Rust engine owns configuration, exact abbreviation lookup, selection
protection, and literal edit results. It does not classify reserved words,
operators, declarations, quoting, or command boundaries from shell text.

Each adapter supplies facts from its own shell. A target is an unquoted command
word, a verified replacement range, and normalized command context. Range offsets
count Unicode characters, as cursor offsets already do. Rust checks that the
range is valid and contains the exact literal trigger before replacing it.
Unavailable context means no edit. An unknown word identity invalidates selection
tracking for the current editing session; it cannot silently restore protection
after later edits. A fresh line starts a fresh session.

Zsh uses its native lexer for word identity and reuses its existing completion
widget in context-only mode with the query cursor at the end of the command word.
The query position is independent of the editing cursor: inside function
parentheses, Zsh can otherwise report a declaration as plain command words.
The mode returns before candidate state initialization, `compinit`, or candidate
providers. During this query only, locally enabled `COMPLETE_ALIASES` keeps
literal alias names intact. Caller options are restored immediately afterward.
It compares the parser's command words with the complete lexical input and
requires native literal quoting to leave the command word unchanged. This
excludes quoted words, substitutions, and patterns independently of the cursor's
completion context. Native reserved words are also rejected. Variables in
arguments do not change the command-position query. Zsh's synthetic empty
current word at a whitespace insertion point is excluded from this comparison.
No extra completion widget is needed. The adapter restores the editing cursor
after the query and restores `BUFFER` if the native query changed it.
Redirections and any other context this query cannot establish are currently
unsupported for Enter expansion. Function declarations and command
lists do not pass the complete-word-coverage check. Continuation prompts never
offer targets. These checks belong to the Zsh adapter, not to the shared engine.

Bash, Fish, or another future adapter must obtain equivalent context from an
appropriate native facility. It must decline to offer a target when it cannot
establish that capability. Passing a different shell's raw tokens into a common
reserved-word or operator list is not supported. The portable candidate and edit
model continues to work independently of optional Enter expansion.

The daemon advertises `edit_context=2` in the startup handshake. The Zsh adapter
activates Enter integration only when the configuration opt-in and this protocol
capability are both present. Its framed observations carry these shell-neutral
facts:

| Operation | Fields |
| --- | --- |
| `reset` | No fields |
| `select` | Edit start, selection applied (`0`/`1`), identity known, word identity, word start |
| `accept` | Buffer, identity known, word identity, word start, word end, verified target available (`0`/`1`), normalized command context |

Word identity denotes the shell's literal first token, not the value after
expansion. Identity is checked only during selection and acceptance. If the user
edits away from a selected word and returns to it before either operation, that
word remains protected. This conservative same-line rule avoids per-keystroke
tracking and per-redraw requests. An optional line-init callback resets selection
state after terminal interruption or send-break, alongside the adapter's
unconditional popup-state reset. An empty known
identity denotes no command word. Selection application and acceptance include the current identity, so separate observations are not
needed immediately before them. Placeholder edits are followed by a selection
observation of the prepared command, whose identity only the adapter can supply.

Malformed observations, including invalid UTF-8, return an explicit error and
invalidate the daemon session. The adapter disables Enter expansion until reload
after a malformed response. Transport failure disables edits for the current line;
the existing line-init hook reconnects and resets the session at the next fresh
line, never while accepting or applying a selection. Missing sessions after restart or eviction preserve
the input. The default remains disabled and adds no Enter-time abbreviation work.

The Zsh adapter keeps Enter and Space opt-ins separate from their shared
`_zacrs_edit_transport_state`: disabled, ready, invalid response, or transport
unavailable. Session initialization is tracked separately across reloads so
reloading cannot discard protection for an explicitly selected command. The
former `_zacrs_enter_enabled` name is consulted only when identifying a reload
from an older adapter that lacks the session-initialization flag.

## Optional Space edits

The handshake also advertises `space_expansion=0|1`. Either expansion option
enables the shared editing session. The `space` operation carries the same
fields as `accept`, but preserves the session and never executes. Zsh offers a
target only when the editing cursor is at the end of the command word.
Recursive editors never offer abbreviation targets or update the outer editing
session: their input may be replacement text rather than a command line.
Zsh also requires its native `CONTEXT` to identify a primary command prompt;
`vared`, `select`, and continuation editors do not supply abbreviation targets.
Rust applies the verified edit and returns the insertion cursor plus
`insert_trigger=1` in APPLY metadata when the native separator should be inserted.
An explicit cursor marker suppresses that insertion. The adapter delegates the
separator to its saved insertion widget, preserving existing plugin behavior. The adapter then reports the prepared
command identity through `select` to prevent repeated expansion. The operation
is gated independently of Enter expansion; older adapters ignore the new flag.

Before a Space context query, Zsh checks its native `SUFFIX_ACTIVE` state.
Pending completion suffixes bypass abbreviation expansion and go directly to
the current insertion widget. Querying completion context would consume that
state even without an edit, so restoring only the buffer and cursor is
insufficient. Removal and retention remain the native widget's responsibility.

## Filesystem fallback provider

Rust owns literal filesystem enumeration and constructs structured candidates
without passing filenames through native completion's line-delimited TSV. The
existing selection policy remains native completion first, filesystem and shell
command-table fallback only after native completion produces no candidates.
PATH, aliases, builtins, functions and command rescue remain in the adapter in
this first slice. Arbitrary native completion, asynchronous scheduling and a
shared provider manager are later work.

The Zsh adapter captures native completion context before providers can change
it with `compset`. It supplies a literal path prefix, its insertion/display
spelling and the current working directory. It offers this capability only at
an unquoted literal word end, including an empty argument. Native lexical words
must cover the input. Quoted words, substitutions, patterns, continuation input
and cursors inside a word do not request filesystem fallback. Unlike the former
last-space/glob fallback, unknown context cannot produce an unsafe file edit.
Native completion continues to handle those contexts independently. Explicit
command paths such as `./` and home-relative paths such as `~/` are supported;
bare command words retain the command-table path. Home-relative candidates keep
that display spelling but insert a safely quoted expanded path.

Each request carries cwd; the shared daemon never uses its own cwd for lookup.
`render-files` and `complete-files` carry three length-framed UTF-8 fields before
the existing prefix/candidate payload. Older daemons reject the distinct verbs
instead of interpreting a structured body as TSV. The subprocess path accepts
the same query through arguments. Native candidate transport is unchanged.
The daemon retains structured filesystem snapshots with candidate and active
popup caches, so navigation and cached selection do not enumerate again. The
adapter scopes reuse to session, generation, cwd, HOME, cursor and right-side
input,
with the existing exact-prefix rule. Popup snapshots require the complete buffer
and cursor, cwd and HOME to match, so a right-side-only edit or changed home
directory cannot reuse stale file context. A provider changing HOME after native
context capture invalidates that captured filesystem expansion; a fresh redraw
can collect it in the new environment. If a retry establishes new context after
HOME changes, its cache key is refreshed before the query is submitted.

Literal file paths are separate from display text and configured suffixes.
Control characters and literal backslashes have distinct display escapes, while
exact UTF-8 filename bytes, including tabs and newlines, survive collection and
selection. APPLY metadata
carries the literal selected path and suffix separately; Zsh quotes the path
with its native quoting facility before applying the suffix. Unambiguous-prefix
extension is disabled for this source so cancellation cannot apply unquoted
display text. Non-UTF-8 names are skipped, never converted lossily. Hidden names
require an explicit dot prefix. Directory symlinks are classified as directories.
This step adds no hooks, widgets, shell option changes or helper subprocesses.

Filesystem collection currently runs synchronously in the shared daemon when a
fresh request is handled. A slow filesystem can therefore delay other requests;
this first slice does not provide asynchronous scheduling or cancellation of an
in-progress directory scan. Moving scheduling to Rust is a separate next step.
