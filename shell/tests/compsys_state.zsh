#!/usr/bin/env zsh

set -eu

zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-compsys-state.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT

export ZACRS_STATE_COMPSYS="${test_dir:h}/_zacrs_compsys.zsh"
export ZACRS_STATE_UTIL="${test_dir:h}/_zacrs_util.zsh"
mkdir -p "$tmp_dir/home/space 日本語"
: > "$tmp_dir/home/file"
ln -s 'space 日本語' "$tmp_dir/home/link"
export ZACRS_STATE_HOME="$tmp_dir/home"
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

    if [[ "$ZACRS_STATE_CASE" == kinds ]]; then
        local HOME="$ZACRS_STATE_HOME"
        compadd -Uf -- "~" "~/" "~/file" "~/space 日本語" "~/link" "~/missing"
        print -rl -- "${_zacrs_captured[@]}" > "$ZACRS_STATE_MARKER"
        return 0
    fi
    case "$ZACRS_STATE_CASE" in
        plain|plain-debug)
            [[ "$ZACRS_STATE_CASE" == plain-debug ]] && local ZACRS_DEBUG=1 ZACRS_LOG="$ZACRS_STATE_MARKER.debug"
            compadd -U -- "alpha" "日本語 space" "a*b&c" "repo/package" "-option"
            ;;
        affixes) compadd -U -P "pre&" -S "*end" -- "alpha" "beta" ;;
        description) compadd -U -X "package entries" -- "alpha" "beta" ;;
        mapped-description)
            local -a displays=("alpha -- first entry" "beta -- second entry")
            compadd -U -d displays -- "alpha" "beta"
            ;;
        slash) compadd -U -- "alpha" "directory/" ;;
        # Compsys keeps these in the order they were added.
        unsorted) compadd -U -V group -- "beta" "alpha" ;;
        nosort) compadd -U -o nosort -- "beta" "alpha" ;;
    esac
    if [[ "$ZACRS_STATE_CASE" == (plain|plain-debug|affixes|description|mapped-description|slash|unsorted|nosort) ]]; then
        print -rl -- "${_zacrs_captured[@]}" > "$ZACRS_STATE_MARKER"
        return 0
    fi
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

# Like compsys, captured matches are sorted unless the call asks otherwise.
run_case kinds $'~\t\tdirectory\n~/\t\tdirectory\n~/file\t\tfile\n~/link\t\tdirectory\n~/missing\t\tfile\n~/space 日本語\t\tdirectory'

run_case plain $'-option\t\t\na*b&c\t\t\nalpha\t\t\nrepo/package\t\t\n日本語 space\t\t'
run_case plain-debug $'-option\t\t\na*b&c\t\t\nalpha\t\t\nrepo/package\t\t\n日本語 space\t\t'
run_case affixes $'pre&alpha*end\t\t\npre&beta*end\t\t'
run_case description $'alpha\tpackage entries\t\nbeta\tpackage entries\t'
run_case mapped-description $'alpha\tfirst entry\t\nbeta\tsecond entry\t'
run_case slash $'alpha\t\t\ndirectory/\t\tdirectory'
run_case unsorted $'beta\t\t\nalpha\t\t'
run_case nosort $'beta\t\t\nalpha\t\t'
