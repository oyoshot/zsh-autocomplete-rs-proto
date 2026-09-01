#!/usr/bin/env zsh

set -eu

typeset test_dir="${0:A:h}"

# The behavior under test is the cache generation changed by the accept-line
# wrapper. Stub shell integration so the test does not depend on a terminal or
# contact an already-running daemon.
zle() { :; }
bindkey() { :; }
zmodload() { return 1; }
autoload() { :; }
add-zle-hook-widget() { :; }
ZACRS_BIN=false
source "${test_dir:h}/zsh-autocomplete-rs.plugin.zsh" || true
_zacrs_clear_popup() { :; }

LBUFFER='git add '
_zacrs_current_context_key
typeset first_key="$REPLY"

_zacrs_accept_line

LBUFFER='git add '
_zacrs_current_context_key
typeset second_key="$REPLY"

if [[ "$first_key" == "$second_key" ]]; then
    print -u2 -r -- "not ok: command boundary reused candidate cache key ($first_key)"
    return 1
fi

typeset first_context="${first_key#*:*:}"
typeset second_context="${second_key#*:*:}"
if [[ "$first_context" != "$second_context" ]]; then
    print -u2 -r -- "not ok: stable PWD/lbase context changed"
    return 1
fi

print -r -- 'ok: command boundary changes candidate cache key'
