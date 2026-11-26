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

# Exercise the production render writer with a regular-file transport. EOF is
# the deliberate response; the captured outbound header must still be one line.
typeset capture="$(mktemp "${TMPDIR:-/tmp}/zacrs-header.XXXXXX")"
trap 'rm -f -- "$capture"' EXIT
zsocket() { exec {REPLY}<> "$capture"; }
COLUMNS=80 LINES=24
check_context_header() {
    local HOME=$'/home/with\vvertical\xc2\xa0space' LBUFFER='probe x'
    local RBUFFER=$'literal\\n\\c\t\n\v\f\r \xc2\x85\xc2\xa0\xe1\x9a\x80\xe2\x80\x80\xe2\x80\x81\xe2\x80\x82\xe2\x80\x83\xe2\x80\x84\xe2\x80\x85\xe2\x80\x86\xe2\x80\x87\xe2\x80\x88\xe2\x80\x89\xe2\x80\x8a\xe2\x80\xa8\xe2\x80\xa9\xe2\x80\xaf\xe2\x81\x9f\xe3\x80\x80'
    local CURSOR=7 header='' key code=0 whitespace
    _zacrs_current_context_key
    key="$REPLY"
    [[ "$key" == *'literal\n\c'* && "$key" == *'%0B'* && "$key" == *'%C2%A0'* ]] || return 1
    for whitespace in $'\t' $'\n' $'\v' $'\f' $'\r' ' ' \
        $'\xc2\x85' $'\xc2\xa0' $'\xe1\x9a\x80' \
        $'\xe2\x80\x80' $'\xe2\x80\x81' $'\xe2\x80\x82' $'\xe2\x80\x83' \
        $'\xe2\x80\x84' $'\xe2\x80\x85' $'\xe2\x80\x86' $'\xe2\x80\x87' \
        $'\xe2\x80\x88' $'\xe2\x80\x89' $'\xe2\x80\x8a' $'\xe2\x80\xa8' \
        $'\xe2\x80\xa9' $'\xe2\x80\xaf' $'\xe2\x81\x9f' $'\xe3\x80\x80'; do
        [[ "$key" != *"$whitespace"* ]] || return 1
    done
    : > "$capture"
    _zacrs_daemon_send_render 2 1 x '' '' "$key" 0 0 || code=$?
    [[ $code == 2 ]] || return 1
    IFS= read -r header < "$capture"
    [[ "$header" == "render 2 1 80 24 context_key=$key popup_key=$$" ]] || return 1
    local old_key="$key"
    HOME=/another/home
    _zacrs_current_context_key
    [[ "$REPLY" != "$old_key" ]] || return 1
}
for context_locale in C C.UTF-8; do
    if ! LC_ALL="$context_locale" check_context_header; then
        print -u2 -r -- "not ok: context key/header encoding LC_ALL=$context_locale"
        return 1
    fi
    print -r -- "ok: raw context header and Unicode whitespace LC_ALL=$context_locale"
done
