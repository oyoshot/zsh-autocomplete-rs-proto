#!/usr/bin/env zsh

set -eu

typeset test_dir="${0:A:h}"

# Stub shell integration so the test does not depend on a terminal or contact
# an already-running daemon.
zle() { :; }
bindkey() { :; }
zmodload() { return 1; }
autoload() { :; }
add-zle-hook-widget() { :; }
typeset -gi preexec_registered=0
add-zsh-hook() {
    [[ "$1:$2" == 'preexec:_zacrs_invalidate_candidate_cache' ]] \
        && preexec_registered=1
}
ZACRS_BIN=false
source "${test_dir:h}/zsh-autocomplete-rs.plugin.zsh" || true

if (( ! preexec_registered )); then
    print -u2 -r -- 'not ok: candidate cache invalidation is not registered for preexec'
    return 1
fi

LBUFFER='git add '
_zacrs_current_context_key
typeset first_key="$REPLY"

_zacrs_invalidate_candidate_cache

LBUFFER='git add '
_zacrs_current_context_key
typeset second_key="$REPLY"

if [[ "$first_key" == "$second_key" ]]; then
    print -u2 -r -- "not ok: preexec reused candidate cache key ($first_key)"
    return 1
fi

typeset first_context="${first_key#*:*:}"
typeset second_context="${second_key#*:*:}"
if [[ "$first_context" != "$second_context" ]]; then
    print -u2 -r -- "not ok: stable PWD/lbase context changed"
    return 1
fi

print -r -- 'ok: preexec changes candidate cache key'
