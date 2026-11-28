---------------------------- MODULE AdapterContract ----------------------------
(* The exchange between a shell adapter and the completion engine.            *)
(*                                                                            *)
(* A snapshot is <<ctx, env, plen>>: which word is being completed, the       *)
(* lookup environment (working directory, options), and how many characters   *)
(* of the word are typed. `genv` on stored candidates is a ghost value: the   *)
(* environment they were really gathered in. `bad` records the first time a   *)
(* popup is drawn (S1) or an edit applied (S2) from candidates that do not    *)
(* belong to the adapter's current word and environment.                      *)
EXTENDS Naturals, FiniteSets

CONSTANTS
    MaxP,            \* longest typed prefix explored
    EngineCompares,  \* rule 8: the engine reuses a popup only for the same snapshot
    Recheck,         \* the adapter re-reads the snapshot after a gather
    Echo,            \* the adapter drops an edit whose snapshot is no longer current
    External         \* the world may change while the shell is busy

VARIABLES ctx, env, plen, cache, popup, job, bad
vars == <<ctx, env, plen, cache, popup, job, bad>>

Snap    == <<ctx, env, plen>>
Idle    == [kind |-> "idle"]
NoPopup == [shown |-> FALSE]

\* The reuse rule: the identical prefix, or a longer one when the set was
\* gathered with a typed prefix and is known not to depend on it.
Covers(src, complete, p) == src = p \/ (complete /\ src > 0 /\ src <= p)

Init ==
    /\ ctx = 0 /\ env = 0 /\ plen = 0
    /\ cache = {} /\ popup = NoPopup /\ job = Idle /\ bad = "none"

\* cd and setopt happen between jobs. External also allows a change while the
\* shell is busy, such as a directory replaced at the same path.
EnvChange ==
    /\ job = Idle \/ External
    /\ env' = 1 - env
    /\ UNCHANGED <<ctx, plen, cache, popup, job, bad>>

DaemonRestart ==
    /\ cache' = {}
    /\ popup' = IF job = Idle THEN NoPopup ELSE popup
    /\ UNCHANGED <<ctx, env, plen, job, bad>>

Type ==
    /\ job = Idle /\ plen < MaxP
    /\ plen' = plen + 1 /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<ctx, env, cache, popup, bad>>

Erase ==
    /\ job = Idle /\ plen > 0
    /\ plen' = plen - 1 /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<ctx, env, cache, popup, bad>>

NextWord ==
    /\ job = Idle
    /\ ctx' = 1 - ctx /\ plen' = 0 /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<env, cache, popup, bad>>

PressTab ==
    /\ job = Idle
    /\ job' = [kind |-> "tab"]
    /\ UNCHANGED <<ctx, env, plen, cache, popup, bad>>

\* The adapter sends its snapshot; the engine answers from the cache or says gather.
Hits == {e \in cache : e.ctx = ctx /\ e.env = env /\ Covers(e.src, e.complete, plen)}

DrawFromCache ==
    /\ job.kind = "redraw"
    /\ \E e \in Hits :
        /\ popup' = [shown |-> TRUE, snap |-> Snap, genv |-> e.genv]
        /\ bad' = IF e.genv = env \/ bad # "none" THEN bad ELSE "S1"
    /\ job' = Idle
    /\ UNCHANGED <<ctx, env, plen, cache>>

AskToGather ==
    /\ job.kind = "redraw" /\ Hits = {}
    /\ job' = [kind |-> "gather", snap |-> Snap, why |-> "redraw"]
    /\ UNCHANGED <<ctx, env, plen, cache, popup, bad>>

\* Native completion runs in whatever environment is current when it finishes.
GatherDone ==
    /\ job.kind = "gather"
    /\ \E complete \in BOOLEAN :
        job' = [kind |-> "gathered", snap |-> job.snap, why |-> job.why,
                genv |-> env, complete |-> complete]
    /\ UNCHANGED <<ctx, env, plen, cache, popup, bad>>

DropStaleGather ==
    /\ job.kind = "gathered" /\ Recheck /\ job.snap # Snap
    /\ job' = Idle /\ popup' = NoPopup
    /\ UNCHANGED <<ctx, env, plen, cache, bad>>

\* The engine stores the set under the snapshot it was asked for. The cache
\* holds at most two sets; which one is evicted is left open.
Store ==
    /\ job.kind = "gathered" /\ ~(Recheck /\ job.snap # Snap)
    /\ LET entry == [ctx |-> job.snap[1], env |-> job.snap[2], src |-> job.snap[3],
                     complete |-> job.complete, genv |-> job.genv]
       IN \E keep \in SUBSET cache :
            /\ Cardinality(keep) <= 1
            /\ cache' = keep \cup {entry}
    /\ IF job.why = "redraw"
       THEN /\ popup' = [shown |-> TRUE, snap |-> job.snap, genv |-> job.genv]
            /\ job' = Idle
            /\ bad' = IF (job.snap[1] = ctx /\ job.snap[3] = plen /\ job.genv = env) \/ bad # "none"
                      THEN bad ELSE "S1"
       ELSE /\ popup' = NoPopup
            /\ job' = [kind |-> "picking", snap |-> job.snap, genv |-> job.genv]
            /\ bad' = bad
    /\ UNCHANGED <<ctx, env, plen>>

\* Tab may select from the popup already on screen.
Reusable ==
    /\ popup.shown
    /\ IF EngineCompares THEN popup.snap = Snap
       ELSE popup.snap[1] = ctx /\ popup.snap[3] = plen

PickFromVisible ==
    /\ job.kind = "tab" /\ Reusable
    /\ job' = [kind |-> "picking", snap |-> popup.snap, genv |-> popup.genv]
    /\ UNCHANGED <<ctx, env, plen, cache, popup, bad>>

GatherForTab ==
    /\ job.kind = "tab" /\ ~Reusable
    /\ job' = [kind |-> "gather", snap |-> Snap, why |-> "tab"]
    /\ UNCHANGED <<ctx, env, plen, cache, popup, bad>>

\* The picker blocks the shell until the user chooses.
DropStaleEdit ==
    /\ job.kind = "picking" /\ Echo /\ job.snap # Snap
    /\ job' = Idle /\ popup' = NoPopup
    /\ UNCHANGED <<ctx, env, plen, cache, bad>>

ApplyEdit ==
    /\ job.kind = "picking" /\ ~(Echo /\ job.snap # Snap)
    /\ bad' = IF (job.genv = env /\ job.snap[1] = ctx) \/ bad # "none" THEN bad ELSE "S2"
    /\ ctx' = 1 - ctx /\ plen' = 0 /\ popup' = NoPopup
    /\ job' = [kind |-> "redraw"]
    /\ UNCHANGED <<env, cache>>

Next ==
    \/ EnvChange \/ DaemonRestart
    \/ Type \/ Erase \/ NextWord \/ PressTab
    \/ DrawFromCache \/ AskToGather \/ GatherDone \/ DropStaleGather \/ Store
    \/ PickFromVisible \/ GatherForTab \/ DropStaleEdit \/ ApplyEdit

Spec == Init /\ [][Next]_vars

\* Every popup drawn and every edit applied belongs to the current word and environment.
Safe == bad = "none"
================================================================================
