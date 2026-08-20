#!/usr/bin/env zsh

set -eu

zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-compsys-state.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT

export ZACRS_STATE_COMPSYS="${test_dir:h}/_zacrs_compsys.zsh"
export ZACRS_STATE_UTIL="${test_dir:h}/_zacrs_util.zsh"
export ZACRS_STATE_MARKER="$tmp_dir/marker"

print -r -- '
autoload -Uz compinit
compinit -C
source "$ZACRS_STATE_UTIL"
source "$ZACRS_STATE_COMPSYS"

_zacrs_state_test_completion() {
    local -i before=$compstate[nmatches] compadd_status=0 after
    local -a candidates

    case "$ZACRS_STATE_CASE" in
        zero) candidates=(zzz) ;;
        one) candidates=(xone) ;;
        many) candidates=(x{001..200}) ;;
    esac

    compadd -- "${candidates[@]}" || compadd_status=$?
    after=$compstate[nmatches]
    print -r -- "$ZACRS_STATE_CASE status=$compadd_status delta=$(( after - before )) captured=${#_zacrs_captured} first=${_zacrs_captured[1]-}" > "$ZACRS_STATE_MARKER"
    return $compadd_status
}

compdef _zacrs_state_test_completion zacrs-state-test
bindkey -M emacs "^T" _zacrs_compsys
bindkey -M viins "^T" _zacrs_compsys
PROMPT="READY> "
RPROMPT=""
' > "$tmp_dir/.zshrc"

run_case() {
    local case_name=$1 expected=$2
    local saved_zdotdir=${ZDOTDIR-}
    local -i had_zdotdir=$+ZDOTDIR
    : > "$ZACRS_STATE_MARKER"
    export ZACRS_STATE_CASE=$case_name
    export ZDOTDIR="$tmp_dir"
    zpty -b zacrs_compsys_state zsh -d
    if (( had_zdotdir )); then
        export ZDOTDIR=$saved_zdotdir
    else
        unset ZDOTDIR
    fi

    local output="" chunk="" i
    for (( i = 0; i < 200; i++ )); do
        zpty -r -t zacrs_compsys_state chunk 2>/dev/null && output+="$chunk"
        [[ "$output" == *"READY> "* ]] && break
        sleep 0.01
    done
    if [[ "$output" != *"READY> "* ]]; then
        print -u2 -r -- "not ok: child zsh did not become ready for $case_name"
        print -u2 -r -- "pty: ${(qqq)output}"
        return 1
    fi

    zpty -w -n zacrs_compsys_state $'zacrs-state-test x\x14'
    for (( i = 0; i < 200; i++ )); do
        [[ "$(<"$ZACRS_STATE_MARKER")" == "$expected" ]] && break
        sleep 0.01
    done

    local actual="$(<"$ZACRS_STATE_MARKER")"
    while zpty -r -t zacrs_compsys_state chunk 2>/dev/null; do
        output+="$chunk"
    done
    zpty -d zacrs_compsys_state 2>/dev/null || true
    if [[ "$actual" != "$expected" ]]; then
        print -u2 -r -- "not ok: $case_name (expected=${(qqq)expected}, actual=${(qqq)actual})"
        print -u2 -r -- "pty: ${(qqq)output}"
        return 1
    fi
    print -r -- "ok: $case_name preserves compadd status and match state"
}

run_case zero 'zero status=1 delta=0 captured=0 first='
run_case one $'one status=0 delta=1 captured=1 first=xone\t\t'
run_case many $'many status=0 delta=101 captured=200 first=x001\t\t'
