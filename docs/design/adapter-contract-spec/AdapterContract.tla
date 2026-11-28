---------------------------- MODULE AdapterContract ----------------------------
(* The exchange between a shell adapter and the completion engine.            *)
(*                                                                            *)
(* A snapshot is <<ctx, env, prefix>>.                                        *)
(*   prefix  the typed part of the word being completed, as text              *)
(*   env     the lookup environment: working directory, home, options         *)
(*   ctx     everything else about the edit state: the other words, the       *)
(*           cursor and the text after it. Any change to those is a change    *)
(*           of ctx; the model does not look inside it.                       *)
(*                                                                            *)
(* Assumptions the results depend on:                                         *)
(*   - The shell handles no key while a job runs. Keys typed meanwhile are    *)
(*     queued and seen afterwards.                                            *)
(*   - A widget that changes env is followed by the redraw hook.              *)
(*   - With InShellChanges, env may also change inside the shell where no     *)
(*     hook follows: from a trap or a file-descriptor handler while the line  *)
(*     is idle, from a completer during a gather, and from a trap that runs   *)
(*     between the picker closing and the edit being applied. Each happens at *)
(*     most once per job. Code that changes env and restores it is treated as *)
(*     not having changed it.                                                 *)
(*                                                                            *)
(* Ghost values, not visible to either side: `genv` is the environment a      *)
(* stored set was really gathered in, and `really` says whether the set truly *)
(* does not depend on the typed prefix.                                       *)
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS
    MaxP,                \* longest typed prefix explored
    EngineCompares,      \* rule 8a: a popup is reused only for the same snapshot
    PostGather,          \* rule 8b: a gather result carries the snapshot seen after it
    EditCheck,           \* rule 8c: an edit is applied only if the snapshot is unchanged
    InShellChanges,      \* env may change inside the shell with no hook following
    External,            \* the world may change while the shell is busy
    EstimateMayBeWrong   \* "does not depend on the prefix" may be claimed wrongly

VARIABLES ctx, env, prefix, cache, popup, job, bad
vars == <<ctx, env, prefix, cache, popup, job, bad>>

Letters == {0, 1}
Snap    == <<ctx, env, prefix>>
Idle    == [kind |-> "idle"]
NoPopup == [shown |-> FALSE, known |-> FALSE]

IsPrefix(s, t) == Len(s) <= Len(t) /\ SubSeq(t, 1, Len(s)) = s

\* The reuse rule: the identical prefix, or an extension of it when the set was
\* gathered with a typed prefix and is claimed not to depend on it.
Covers(e, p) == e.src = p \/ (e.claimed /\ Len(e.src) > 0 /\ IsPrefix(e.src, p))

Flag(ok, name) == IF ok \/ bad # "none" THEN bad ELSE name

Init ==
    /\ ctx = 0 /\ env = 0 /\ prefix = <<>>
    /\ cache = {} /\ popup = NoPopup /\ job = Idle /\ bad = "none"

\* A widget ran cd or setopt. The redraw hook follows every widget.
EnvChangeInShell ==
    /\ job = Idle
    /\ env' = 1 - env /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<ctx, prefix, cache, popup, bad>>

\* Something outside the shell changed the environment, at any moment.
EnvChangeOutside ==
    /\ External
    /\ env' = 1 - env
    /\ UNCHANGED <<ctx, prefix, cache, popup, job, bad>>

\* The engine forgets everything; what is on the screen stays there.
DaemonRestart ==
    /\ cache' = {}
    /\ popup' = [popup EXCEPT !.known = FALSE]
    /\ UNCHANGED <<ctx, env, prefix, job, bad>>

Type ==
    /\ job = Idle /\ Len(prefix) < MaxP
    /\ \E c \in Letters : prefix' = Append(prefix, c)
    /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<ctx, env, cache, popup, bad>>

Erase ==
    /\ job = Idle /\ Len(prefix) > 0
    /\ prefix' = SubSeq(prefix, 1, Len(prefix) - 1)
    /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<ctx, env, cache, popup, bad>>

OtherEdit ==
    /\ job = Idle
    /\ ctx' = 1 - ctx /\ prefix' = <<>> /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<env, cache, popup, bad>>

PressTab ==
    /\ job = Idle
    /\ job' = [kind |-> "tab"]
    /\ UNCHANGED <<ctx, env, prefix, cache, popup, bad>>

\* The adapter sends its snapshot; the engine answers from the cache or says gather.
Hits == {e \in cache : e.ctx = ctx /\ e.env = env /\ Covers(e, prefix)}

DrawFromCache ==
    /\ job.kind = "redraw"
    /\ \E e \in Hits :
        /\ popup' = [shown |-> TRUE, known |-> TRUE, snap |-> Snap, genv |-> e.genv]
        /\ bad' = IF e.genv # env THEN Flag(FALSE, "S1: popup from another environment")
                  ELSE Flag(e.src = prefix \/ e.really, "S3: popup misses candidates")
    /\ job' = Idle
    /\ UNCHANGED <<ctx, env, prefix, cache>>

AskToGather ==
    /\ job.kind = "redraw" /\ Hits = {}
    /\ job' = [kind |-> "gather", snap |-> Snap, why |-> "redraw", changed |-> FALSE]
    /\ UNCHANGED <<ctx, env, prefix, cache, popup, bad>>

\* A trap or a file-descriptor handler changes env while the line is idle.
SilentChange ==
    /\ InShellChanges /\ job = Idle
    /\ env' = 1 - env
    /\ UNCHANGED <<ctx, prefix, cache, popup, job, bad>>

\* A completer changes env during a gather, or a trap changes it after the
\* picker closes and before the edit is applied.
ChangeDuringJob ==
    /\ InShellChanges /\ job.kind \in {"gather", "picking"} /\ ~job.changed
    /\ env' = 1 - env /\ job' = [job EXCEPT !.changed = TRUE]
    /\ UNCHANGED <<ctx, prefix, cache, popup, bad>>

\* Native completion finishes in whatever environment is current.
GatherDone ==
    /\ job.kind = "gather"
    /\ \E really \in BOOLEAN :
       \E claimed \in (IF EstimateMayBeWrong THEN BOOLEAN ELSE {really}) :
        job' = [kind |-> "gathered", snap |-> job.snap, why |-> job.why, post |-> Snap,
                genv |-> env, really |-> really, claimed |-> claimed]
    /\ UNCHANGED <<ctx, env, prefix, cache, popup, bad>>

\* The engine refuses a result whose snapshot changed during the gather.
Changed == PostGather /\ job.post # job.snap

DiscardGather ==
    /\ job.kind = "gathered" /\ Changed
    /\ job' = Idle /\ popup' = NoPopup
    /\ UNCHANGED <<ctx, env, prefix, cache, bad>>

\* The engine stores the set under the snapshot it asked for. The cache holds
\* at most two sets; which one is evicted is left open.
Store ==
    /\ job.kind = "gathered" /\ ~Changed
    /\ LET entry == [ctx |-> job.snap[1], env |-> job.snap[2], src |-> job.snap[3],
                     claimed |-> job.claimed, really |-> job.really, genv |-> job.genv]
       IN \E keep \in SUBSET cache :
            /\ Cardinality(keep) <= 1
            /\ cache' = keep \cup {entry}
    /\ IF job.why = "redraw"
       THEN /\ popup' = [shown |-> TRUE, known |-> TRUE, snap |-> job.snap, genv |-> job.genv]
            /\ job' = Idle
            /\ bad' = Flag(job.snap = Snap /\ job.genv = env, "S1: popup for another snapshot")
       ELSE /\ popup' = NoPopup
            /\ job' = [kind |-> "picking", req |-> job.snap, from |-> job.snap,
                       genv |-> job.genv, changed |-> FALSE]
            /\ bad' = bad
    /\ UNCHANGED <<ctx, env, prefix>>

\* Tab may select from the popup already on screen.
Reusable ==
    /\ popup.shown /\ popup.known
    /\ IF EngineCompares THEN popup.snap = Snap
       ELSE popup.snap[1] = ctx /\ popup.snap[3] = prefix

PickFromVisible ==
    /\ job.kind = "tab" /\ Reusable
    /\ job' = [kind |-> "picking", req |-> Snap, from |-> popup.snap,
               genv |-> popup.genv, changed |-> FALSE]
    /\ UNCHANGED <<ctx, env, prefix, cache, popup, bad>>

GatherForTab ==
    /\ job.kind = "tab" /\ ~Reusable
    /\ job' = [kind |-> "gather", snap |-> Snap, why |-> "tab", changed |-> FALSE]
    /\ UNCHANGED <<ctx, env, prefix, cache, popup, bad>>

\* The picker blocks the shell until the user chooses. `req` is the snapshot
\* the Tab request was made with; `from` is the one the candidates belong to.
Stale == EditCheck /\ job.req # Snap

DropEdit ==
    /\ job.kind = "picking" /\ Stale
    /\ job' = [kind |-> "redraw"] /\ popup' = NoPopup
    /\ UNCHANGED <<ctx, env, prefix, cache, bad>>

ApplyEdit ==
    /\ job.kind = "picking" /\ ~Stale
    /\ bad' = Flag(job.from = Snap /\ job.genv = env, "S2: edit for another snapshot")
    /\ ctx' = 1 - ctx /\ prefix' = <<>> /\ popup' = NoPopup
    /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<env, cache>>

Next ==
    \/ EnvChangeInShell \/ EnvChangeOutside \/ DaemonRestart
    \/ Type \/ Erase \/ OtherEdit \/ PressTab
    \/ SilentChange \/ ChangeDuringJob
    \/ DrawFromCache \/ AskToGather \/ GatherDone
    \/ DiscardGather \/ Store
    \/ PickFromVisible \/ GatherForTab \/ DropEdit \/ ApplyEdit

Spec == Init /\ [][Next]_vars

\* No popup is drawn and no edit applied from candidates that belong to
\* another word, prefix or environment, or that lack candidates.
Safe == bad = "none"

\* Whenever the shell is waiting for a key, a popup on screen is for the
\* current snapshot. This cannot hold once env changes with no hook following.
PopupCurrent ==
    (job = Idle /\ popup.shown) => (popup.snap = Snap /\ popup.genv = env)
================================================================================
