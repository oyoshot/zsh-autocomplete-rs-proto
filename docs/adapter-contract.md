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

| Fact | Required | Source in Zsh | Notes |
|---|---|---|---|
| Edit state | yes | `BUFFER`, `CURSOR`, editor kind | One snapshot per request |
| Word being completed | yes | Native completion (`IPREFIX`, `PREFIX`, `words`, `CURRENT`) | The adapter does not split on spaces itself |
| Candidates | yes | Native completion, shell tables | Text, and description and kind where the shell has them |
| Lookup environment | yes | `PWD`, `HOME`, the options that change completion | Sent as values, not folded into a key the engine cannot read |
| Command context | no | Native lexer | Command words through the cursor, and whether the cursor is on the command word |
| How candidates were produced | no | The intercepted `compadd` calls | See "First use" |
| Terminal cursor position | no | A terminal query | The engine asks the terminal itself when the adapter cannot |
| Pending input | no | `PENDING` | So the engine's answer can be skipped, not so the adapter decides |
| A call on every key | no | The `line-pre-redraw` hook | Without it the adapter serves Tab only |

A shell that cannot supply a required fact from its own facilities does not get
an adapter. When an optional fact is missing the engine takes the cautious
answer: without "how candidates were produced" it never reuses a set for a
longer prefix, and without a call on every key there is no automatic popup.

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
8. **Every request carries the whole snapshot, and the engine binds what it
   stores to it.** The snapshot is the edit state, the word being completed and
   the lookup environment. The engine serves a stored candidate set or a drawn
   popup only for the snapshot it came from, or for one the reuse rule allows.
   The adapter keeps no copy of the snapshot to compare.

## Not protected

A change made outside the shell while the shell is busy, such as a directory
replaced at the same path during a gather or while the picker is open. Comparing
values cannot detect a change that is undone before the comparison, and the
specification below finds that case even with every check enabled.

## Budgets on the input path

| Situation | Daemon round trips | Processes started |
|---|---|---|
| Buffer unchanged | 0 | 0 |
| Buffer changed, engine can answer | 1 | 0 |
| Buffer changed, gather needed | 2 | 0, apart from what native completion starts |

Without a daemon the adapter starts one process per render or completion. That
path keeps behavior, not speed.

## Checked against a model

[`design/adapter-contract-spec/AdapterContract.tla`](design/adapter-contract-spec/AdapterContract.tla)
is a TLA+ specification of the exchange. TLC explores every interleaving of
keys, in-shell environment changes, slow gathers, Tab and daemon restarts for
two word positions, two environments and prefixes up to two characters. The
invariant is that a drawn popup and an applied edit always come from candidates
gathered for the current word and environment.

| Configuration | Distinct states | Result |
|---|---|---|
| `WordAndPrefixOnly`: popup reuse matched on word and prefix | — | Violated: an edit from before a `cd` is applied after it |
| `Rule8`: the engine compares the whole snapshot | 18,364 | Holds |
| `Rule8AdapterRechecks`: the adapter also re-checks | 18,364 | Holds; the re-check adds nothing |
| `Rule8ExternalChanges`: changes from outside the shell | — | Violated, as "Not protected" says |

```sh
cd docs/design/adapter-contract-spec
tlc -config Rule8.cfg AdapterContract.tla
```

TLC is not run in CI. The specification checks the design, not the code: nothing
ties its actions to the Rust or Zsh sources.

## Other shells

The two tables above name no Zsh syntax. Six throwaway adapters each drove the
unchanged engine binary through Tab, a selection and the resulting edit in a
pseudo-terminal, on one Linux machine:

| Shell | Word and candidates from | Edit applied by | Gaps found |
|---|---|---|---|
| fish 4.9 | `commandline -ct`, `complete -C` | `commandline -rt` | Cursor position not available; no call on every key |
| bash 5.3 | `COMP_WORDS` and `COMPREPLY` inside programmable completion | Returning one `COMPREPLY` entry | Cursor position not available; no descriptions; no call on every key |
| PowerShell 7.6 | `CommandCompletion.CompleteInput` | `PSConsoleReadLine.Replace` | None for Tab |
| Nushell 0.116 | `commandline complete --detailed` | `commandline edit` | Cursor position not available |
| xonsh 0.25 | The prompt_toolkit completer | The prompt_toolkit buffer | Cursor position not available |
| Elvish 0.21 | `edit:wordify`, a completer the adapter chooses | `edit:current-command` | No single "complete this line" call, so command names were not covered |

Each ran three or four scenarios once, on the subprocess path only, so neither
the daemon nor an automatic popup was exercised, and none handled every reply
code. Five of the six needed a helper program to read the cursor position, which
is why that fact is optional and falls to the engine. The prototypes were not
kept: they were 20 to 40 lines each, untested, and depended on that helper.

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
| 8 | The adapter stores the lookup environment three times and compares it itself; the engine sees it only inside the key | read |
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
