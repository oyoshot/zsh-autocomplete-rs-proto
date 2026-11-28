"""Explicit-state model of the adapter/engine exchange.

Checks two safety properties over every interleaving of user keys, external
environment changes (cwd, options), slow gathers, the Tab picker and daemon
restarts, for several variants of the contract.

  S1  every popup drawn shows candidates gathered for the adapter's current
      word position and environment, and covers the typed prefix
  S2  every edit applied was computed from such candidates

A snapshot is (ctx, env, plen): which word is being completed, the lookup
environment, and how many characters of the word are typed. `genv` on stored
candidates is a ghost value: the environment they really came from.
"""
import sys
from collections import deque

MAXP = 2

def covers(src_plen, complete, plen):
    return src_plen == plen or (complete and 0 < src_plen <= plen)

def step(state, V):
    ctx, env, plen, cache, popup, job = state
    snap = (ctx, env, plen)
    out = []   # (label, new_state, violation or None)
    def emit(label, **kw):
        s = dict(ctx=ctx, env=env, plen=plen, cache=cache, popup=popup, job=job); v = kw.pop('bad', None); s.update(kw)
        out.append((label, (s['ctx'], s['env'], s['plen'], s['cache'], s['popup'], s['job']), v))
    # The outside world can change at any time.
    # In-shell changes (cd, setopt) happen between jobs. `external` also lets the
    # world change while the shell is busy: a directory swapped at the same path.
    if job is None or V['external']:
        emit('env-change', env=1 - env)
    emit('daemon-restart', cache=frozenset(), popup=popup if job else None)
    if job is None:
        # User edits. Each is followed by the redraw hook as a separate step.
        if plen < MAXP: emit('type', plen=plen + 1, job=('redraw',))
        if plen > 0: emit('erase', plen=plen - 1, job=('redraw',))
        emit('next-word', ctx=1 - ctx, plen=0, job=('redraw',))
        emit('tab', job=('tab',))
        return out
    kind = job[0]
    if kind == 'redraw':
        # Adapter sends its snapshot; engine answers from the cache or says gather.
        hit = [e for e in cache if e[0] == ctx and e[1] == env and covers(e[2], e[3], plen)]
        if hit:
            for e in hit:
                bad = None if e[4] == env else 'S1: drew candidates from another environment'
                emit('draw-from-cache', popup=(snap, e[4]), job=None, bad=bad)
        else:
            emit('gather-start', job=('gather', snap, 'redraw'))
    elif kind == 'gather':
        _, s, why = job
        # Native completion runs now, in whatever environment is current.
        for complete in (0, 1):
            emit('gather-done', job=('gathered', s, why, env, complete))
    elif kind == 'gathered':
        _, s, why, genv, complete = job
        if V['recheck'] and s != snap:
            emit('drop-stale-gather', job=None, popup=None)      # adapter noticed the world moved
            return out
        entry = (s[0], s[1], s[2], complete, genv)
        newcache = frozenset(list(cache)[-1:] + [entry])
        if why == 'redraw':
            bad = None if (s[0], s[2]) == (ctx, plen) and genv == env else 'S1: drew a gather made for another snapshot'
            emit('store+draw', cache=newcache, popup=(s, genv), job=None, bad=bad)
        else:
            emit('store+pick', cache=newcache, popup=None, job=('picking', s, genv))
    elif kind == 'tab':
        # Engine may reuse the visible popup if it was drawn for this snapshot.
        if popup is not None and (popup[0] == snap if V['engine_compares'] else popup[0][0] == ctx and popup[0][2] == plen):
            emit('pick-visible', job=('picking', popup[0], popup[1]))
        else:
            emit('gather-start', job=('gather', snap, 'tab'))
    elif kind == 'picking':
        _, s, genv = job
        # The picker blocks the shell; the user chooses at some later time.
        if V['echo'] and s != snap:
            emit('drop-stale-edit', job=None, popup=None)
        else:
            bad = None if genv == env and s[0] == ctx else 'S2: applied an edit computed for another environment'
            emit('apply-edit', ctx=1 - ctx, plen=0, popup=None, job=('redraw',), bad=bad)
    return out

def check(V):
    init = (0, 0, 0, frozenset(), None, None)
    seen = {init: None}; q = deque([init]); bad = None
    while q:
        st = q.popleft()
        for label, nxt, v in step(st, V):
            if v and bad is None:
                trace = [label + '  <-- ' + v]; cur = st
                while seen[cur] is not None:
                    cur, lab = seen[cur]; trace.append(lab)
                bad = list(reversed(trace))
            if nxt not in seen:
                seen[nxt] = (st, label); q.append(nxt)
    return len(seen), bad

variants = [
    ('A: adapter sends facts; engine reuses a popup for the same word and prefix', dict(recheck=0, echo=0, engine_compares=0, external=0)),
    ('B: A + engine compares the full snapshot before reusing a popup', dict(recheck=0, echo=0, engine_compares=1, external=0)),
    ('C: B + adapter re-reads the snapshot after a gather', dict(recheck=1, echo=0, engine_compares=1, external=0)),
    ('D: C + instructions name their snapshot; adapter drops stale ones', dict(recheck=1, echo=1, engine_compares=1, external=0)),
    ('A+: A, with the world also changing while the shell is busy', dict(recheck=0, echo=0, engine_compares=0, external=1)),
    ('D+: D, with the world also changing while the shell is busy', dict(recheck=1, echo=1, engine_compares=1, external=1)),
]
for name, V in variants:
    n, bad = check(V)
    print(f"{name}\n  states={n}  {'SAFE' if bad is None else 'VIOLATION'}")
    if bad: print('  trace: ' + ' -> '.join(bad))
