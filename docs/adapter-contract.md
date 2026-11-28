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
told to. Writing an edit in the shell's own syntax, quoting included, is part
of carrying it out: the engine says which literal text or path goes in, and the
adapter spells it for its shell.

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
| How candidates were produced | no | The intercepted `compadd` calls | An observation the engine may use as evidence; see "First use" |
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
| Edit | Put the stated literal text or path in the stated range, written in the shell's syntax; optionally accept the line |
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
8. **Results are tied to the snapshot they were computed for.** The snapshot is
   the edit state, the word being completed and the lookup environment.
   1. Every request carries it. The engine serves a stored candidate set or a
      drawn popup only for the snapshot it came from, or for one the reuse rule
      allows.
   2. A gather result carries the snapshot observed after the gather as well.
      The engine discards a result whose two snapshots differ, because a
      completer may change the working directory or an option.
   3. The adapter applies an edit only if its snapshot still equals the one the
      request was made with. This is a comparison of two values it already
      holds, not a judgement about the edit.

   The adapter may also remember the last snapshot it sent, in order not to
   send the same request twice. It does not use a remembered snapshot to decide
   whether a popup or a cached set is still good; that is 8.1.

## Not protected

A change made outside the shell while the shell is busy, such as a directory
replaced at the same path during a gather or while the picker is open. Comparing
values cannot detect a change that is undone before the comparison, and the
specification below finds a failure even with all of rule 8 in force.

## Budgets on the input path

| Situation | Daemon round trips | Processes started |
|---|---|---|
| Snapshot unchanged since the last request | 0 | 0 |
| Snapshot changed, engine can answer | 1 | 0 |
| Snapshot changed, gather needed | 2 | 0, apart from what native completion starts |

"Unchanged" means the whole snapshot. A `cd` leaves the buffer alone and still
changes it.

Without a daemon the adapter starts one process per render or completion. That
path keeps behavior, not speed.

## Not yet specified

The contract says who decides. It does not yet give numbers or rules for:

- **Latency.** No target for key-to-popup time, such as a 95th or 99th
  percentile. The round-trip budget above bounds the count, not the duration.
- **Memory.** The daemon keeps 8 candidate sets, 16 popups and 128 abbreviation
  sessions, each set at most 1 MiB. There is no stated total and no per-shell
  share.
- **Cancellation.** What happens to a request, or a slow gather, once the
  snapshot has moved on. Rule 8 makes a late answer harmless; it does not stop
  the work.
- **Isolation between shells.** Several shells share one daemon and one cache.
  One shell's slow request can delay another's: from a read of the code, the
  daemon accepts requests on a single thread and collects file candidates
  there.
- **Failure.** A daemon restart, a timeout or a reply cut short must leave the
  edit buffer and pending keys untouched. This is the intent of the current
  code and is not written as a rule.

## Checked against a model

[`design/adapter-contract-spec/AdapterContract.tla`](design/adapter-contract-spec/AdapterContract.tla)
is a TLA+ specification of the exchange. TLC explores every interleaving of
typing, erasing, other edits, environment changes, slow gathers, Tab and daemon
restarts for two word positions, two environments and prefixes of up to two
characters over two letters. A violation is a popup drawn, or an edit applied,
from candidates that belong to another word, prefix or environment, or that
lack candidates.

| Configuration | Environment may change | Result |
|---|---|---|
| `Rule8Quiet` | Only by a widget, which the redraw hook follows | Holds (72,385 states). A popup on screen also always matches the snapshot while the shell waits for a key |
| `Rule8` | Also with no hook following: a trap while idle, a completer during a gather, a trap before an edit is applied | Holds (860,940 states) |
| `WithoutEngineCompare` | As `Rule8`, without 8.1 for popups | Violated: a trap changes the directory, then Tab selects from the popup drawn before |
| `WithoutPostGather` | As `Rule8`, without 8.2 | Violated: a completer changes the directory and the result is drawn as if it had not |
| `WithoutEditCheck` | As `Rule8`, without 8.3 | Violated: the directory changes after the picker closes and the edit is still applied |
| `OutsideChanges` | Also from outside the shell | Violated, as "Not protected" says |
| `WrongEstimate` | As `Rule8`, and a set may be wrongly taken as independent of the prefix | Violated: a reused set lacks candidates |

Each part of rule 8 is therefore needed, and together they are enough, within
the model. `sh docs/design/adapter-contract-spec/check.sh` runs all seven and
compares them with this table. It needs `tlc`; CI does not run it.

What the results rest on:

- The shell handles no key while a hook, a gather or the picker runs. Keys
  typed meanwhile are queued. This holds for Zsh widgets and hooks.
- Code that changes the environment and restores it counts as not having
  changed it.
- The word position stands for everything else in the edit state: the other
  words, the cursor and the text after it. The model does not look inside it.
- Whether a set depends on the typed prefix is given to the model, not derived.
  `WrongEstimate` shows what follows when it is given wrongly.
- Nothing ties the specification's actions to the Rust or Zsh sources. It checks
  the design, not the code.

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
character runs native completion again. Two separate things change.

**The mechanism.** The engine may serve a stored set for a longer prefix in an
otherwise equal snapshot, filtering it itself. This is correct exactly when
gathering with the longer prefix would return the stored set filtered by that
prefix.

**The estimate.** Nothing the adapter can observe proves that. The engine uses
this evidence: the set was gathered with a non-empty prefix, and in every
native `compadd` call more words were passed than matched. A completer that
passes words which do not match the typed prefix is usually listing everything
and letting `compadd` filter.

The estimate can be wrong. A completer may choose its list from the prefix and
still pass words that do not match:

```zsh
_example() {
    if [[ $PREFIX == ab* ]]; then
        compadd abc abcd xyz
    else
        compadd apple apricot xyz
    fi
}
```

At `a` three words are passed and two match, so the set looks independent of
the prefix. At `ab` native completion offers `abc` and `abcd`; the stored set
has neither. The estimate is therefore engine policy with a way to turn it off,
not a guarantee, and a wrong estimate costs missing candidates until the next
gather.

Measured on one machine over the 468 installed commands that have a completer
(about 20,000 inputs, 35,799 pairs of a prefix and a longer one):

- The estimate would reuse 4,041 pairs, 11% of all pairs. Counted by input,
  3,703 of the 16,278 typed-prefix gathers that returned candidates qualify,
  23%. File-name completion is not among them.
- Three reused pairs missed a candidate, each because the system changed
  between the two runs (a process appeared; another completer defined a
  function).
- The 251 pairs where a longer prefix produced new candidates, including
  `chmod a` to `chmod ag` and `tar A` to `tar Af`, all fell on the "Gather"
  side.

No completer of the `_example` kind was among the 468. That is an observation
about one machine, not a bound.
