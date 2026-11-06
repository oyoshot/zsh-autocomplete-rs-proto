# Shell adapter boundary for Enter edits

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
tracking and per-redraw requests. Line-init is the sole new lifecycle hook; it
also resets state after terminal interruption or send-break. An empty known
identity denotes no command word. Selection application and acceptance include the current identity, so separate observations are not
needed immediately before them. Placeholder edits are followed by a selection
observation of the prepared command, whose identity only the adapter can supply.

Malformed observations, including invalid UTF-8, return an explicit error and
invalidate the daemon session. The adapter disables Enter expansion until reload
after a communication error. Missing sessions after restart or eviction preserve
the input. The default remains disabled and adds no Enter-time abbreviation work.
