#!/usr/bin/env zsh

set -eu

zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-cache-context.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT

mkdir -p "$tmp_dir/runtime"
export XDG_RUNTIME_DIR="$tmp_dir/runtime"
export ZACRS_CACHE_CONTEXT_PLUGIN="${test_dir:h}/zsh-autocomplete-rs.plugin.zsh"
export ZACRS_CACHE_CONTEXT_MARKER="$tmp_dir/marker"

print -r -- '
ZACRS_BIN=false
source "$ZACRS_CACHE_CONTEXT_PLUGIN"

# Keep this test focused on the line-init cache generation. The redraw path is
# covered separately and would otherwise try to query a terminal cursor.
_zacrs_line_pre_redraw() { :; }

record_context_key() {
    _zacrs_current_context_key
    print -r -- "$REPLY" >> "$ZACRS_CACHE_CONTEXT_MARKER"
    BUFFER=":"
    zle accept-line
}
zle -N record_context_key
bindkey -M emacs "^T" record_context_key
bindkey -M viins "^T" record_context_key

PROMPT="READY> "
RPROMPT=""
' > "$tmp_dir/.zshrc"

export ZDOTDIR="$tmp_dir"
zpty -b zacrs_cache_context zsh -d

wait_for_prompt() {
    local output="" chunk="" i
    for (( i = 0; i < 400; i++ )); do
        zpty -r -t zacrs_cache_context chunk 2>/dev/null && output+="$chunk"
        [[ "$output" == *"READY> "* ]] && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: child zsh did not become ready"
    print -u2 -r -- "pty: ${(qqq)output}"
    return 1
}

wait_for_line_count() {
    local expected=$1
    local i actual=0
    for (( i = 0; i < 200; i++ )); do
        [[ -f "$ZACRS_CACHE_CONTEXT_MARKER" ]] \
            && actual=$(wc -l < "$ZACRS_CACHE_CONTEXT_MARKER")
        (( actual >= expected )) && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: timed out waiting for $expected context keys"
    return 1
}

wait_for_prompt
zpty -w -n zacrs_cache_context $'git add \x14'
wait_for_line_count 1
wait_for_prompt
zpty -w -n zacrs_cache_context $'git add \x14'
wait_for_line_count 2

typeset -a keys
keys=("${(@f)$(<"$ZACRS_CACHE_CONTEXT_MARKER")}")

if [[ "${keys[1]}" == "${keys[2]}" ]]; then
    print -u2 -r -- "not ok: command boundary reused candidate cache key (${keys[1]})"
    return 1
fi

typeset first_context="${keys[1]#*:*:}"
typeset second_context="${keys[2]#*:*:}"
if [[ "$first_context" != "$second_context" ]]; then
    print -u2 -r -- "not ok: stable PWD/lbase context changed"
    return 1
fi

print -r -- 'ok: command boundary changes candidate cache key'
