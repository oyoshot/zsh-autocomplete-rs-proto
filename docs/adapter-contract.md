# Adapter and engine contract

This is the rule for deciding where new behavior goes. It states the target,
then lists where the current Zsh adapter and protocol differ from it.
[`shell-adapters.md`](shell-adapters.md) records how individual features were
built; this page is what a new feature is checked against.

## Roles

**The adapter reports facts and carries out instructions.** A fact is something
only the shell can know: the edit buffer and cursor, the shell's own lexing of
the line, what native completion offers, the working directory and options, and
what the terminal is doing. Carrying out means applying an edit the engine
returned, writing the bytes it rendered, and running native completion when
told to.

**The engine decides and remembers.** It chooses which candidates to offer and
from which source, whether a stored set may be reused, how they are filtered and
ordered, what is too large or too slow to show, and what an edit is. It holds
the candidate cache and the popup session.

A check for any change: if the adapter contains an `if` whose inputs the engine
could have, the decision belongs in the engine and the adapter sends the inputs.

## What the adapter sends

| Fact | Source in Zsh | Notes |
|---|---|---|
| Edit state | `BUFFER`, `CURSOR`, editor kind | One snapshot per request |
| Word being completed | Native completion (`IPREFIX`, `PREFIX`, `words`, `CURRENT`) | The adapter does not split on spaces itself |
| Command context | Native lexer | Command words through the cursor, and whether the cursor is on the command word |
| Lookup environment | `PWD`, `HOME`, the options that change completion | Sent as values, not folded into a key the engine cannot read |
| Candidates | Native completion, shell tables | Text, description, kind, plus how each set was produced |
| Terminal | Size, cursor position | |
| Pending input | `PENDING` | So the engine's answer can be skipped, not so the adapter decides |

Facts about how candidates were produced are part of the candidates. The first
one this contract adds: whether native completion passed more words than
matched the typed prefix. That tells the engine the set does not depend on the
prefix and may serve longer prefixes (see "First use").

## What the engine returns

Each request gets exactly one of these, on every path:

| Instruction | Adapter action |
|---|---|
| Draw | Write the given bytes; remember the reported popup position |
| Clear | Remove the popup |
| Gather | Run native completion for this position and send the result |
| Edit | Replace the stated range with the stated text; optionally accept the line |
| Use native | Hand the key to the shell's own widget |
| Nothing | Leave everything as it is |

"Gather" is what `CACHE_MISS` already means. Naming it lets the engine also
answer "Nothing" where gathering is not wanted, without a new reply word for
each reason.

## Rules

1. **One definition per concept.** The word being completed and the command
   position come from native facts. Every request uses the same ones.
2. **Identifiers are opaque and minted by one side.** The engine does not parse
   an adapter key, and the adapter does not interpret an engine token.
3. **Limits and thresholds live in the engine.** The adapter holds no size
   limit, length threshold or timeout that changes what the user sees.
   Transport timeouts are mechanics and stay in the adapter.
4. **The same condition gets the same answer on every path.** Socket and
   subprocess differ in transport only.
5. **Every request gets a reply line.** A daemon that cannot serve a request
   says so; it does not close the connection silently.
6. **Unknown tokens are ignored; new replies are opt-in.** A sender states
   which replies it understands. The handshake names capabilities.
7. **No shell code crosses the boundary.** The engine never returns text for the
   adapter to evaluate, and the adapter never evaluates candidate text.

## Budgets on the input path

| Situation | Daemon round trips | Processes started |
|---|---|---|
| Buffer unchanged | 0 | 0 |
| Buffer changed, engine can answer | 1 | 0 |
| Buffer changed, gather needed | 2 | 0, apart from what native completion starts |

Without a daemon the adapter starts one process per render or completion. That
path keeps behavior, not speed.

## Other shells

A second adapter implements the "sends" table with its own shell's facilities
and the "returns" table as written. Nothing in either table names Zsh syntax.
Whether another shell can supply each fact has not been checked.

## Where the code differs today

From a read of the adapter and protocol on 2026-11-28. Items marked "read" were
found by reading and not confirmed by running.

| Rule | Difference | Status |
|---|---|---|
| Roles | Source order (native, then files, then command tables), the three-character threshold for typo rescue, the blank-buffer skip and "always accept a single candidate on Tab" are decided in the adapter | read |
| Roles | The auto-trigger and Tab each implement candidate collection; only the auto-trigger retries an empty native result | read |
| 1 | Command position has four definitions that disagree, for example on `ls & gi` | read |
| 1 | The typed word is the native one after a gather and "text after the last space" for the cache probe and for popup reuse | read |
| 2 | The cache key is built by the adapter from the working directory, buffer, home and options, with a hand-kept list of characters to escape so the engine's header parser does not split it | confirmed |
| 2 | The lookup environment is stored three times in the adapter and once more inside the key | read |
| 2 | The engine computes a reuse token; the adapter discards it and sends the constant `1` | read |
| 4 | An oversized set, a failed file lookup and a malformed request each produce different replies on the socket, the subprocess render and Tab | read |
| 4 | Terminal size comes from `COLUMNS`/`LINES` on the socket and from the terminal on the subprocess | read |
| 5 | An unknown request closes the connection with no reply | read |
| Budgets | Each daemon render starts `od` and `tr` to hex-encode the command context | confirmed |
| Budgets | The cursor position is asked of the terminal on every changed key; the reuse of the last position appears unreachable | read |
| — | A candidate whose text is `END` with empty description and kind is taken as the end of the candidate list | read |

The last row is a defect, not a design gap.

## First use: reusing a gathered set for longer prefixes

Today the cache serves a stored set only for the identical prefix, so every new
character runs native completion again. Under this contract:

- **Fact (adapter).** For each native `compadd` call, whether more words were
  passed than matched. The adapter already intercepts these calls.
- **Decision (engine).** A set gathered with a non-empty prefix, in which every
  call passed more words than matched, may serve any longer prefix in the same
  context. The engine filters it. Otherwise it answers "Gather", as now.

Measured on one machine over the 468 installed commands that have a completer
(about 20,000 inputs, 35,799 prefix pairs): this rule would reuse 4,041 pairs.
Three of them missed a candidate, each because the system changed between the
two runs (a process appeared; another completer defined a function). The 251
pairs where a longer prefix produced new candidates, including `chmod a` to
`chmod ag` and `tar A` to `tar Af`, all fell on the "Gather" side. The rule
applies to about a quarter of typed-prefix gathers; file-name completion is not
among them.
