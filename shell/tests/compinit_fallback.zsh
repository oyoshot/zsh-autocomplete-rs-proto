#!/usr/bin/env zsh
# A configuration that never runs compinit leaves compsys to the adapter. Later
# shells must find completion functions added to $fpath after the first dump,
# and the adapter must leave the user's $fpath as it is.

set -eu

zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-compinit-fallback.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT

export ZACRS_FALLBACK_UTIL="${test_dir:h}/_zacrs_util.zsh"
export ZACRS_FALLBACK_COMPSYS="${test_dir:h}/_zacrs_compsys.zsh"
export ZACRS_FALLBACK_MARKER="$tmp_dir/marker"
export ZACRS_FALLBACK_FUNCTIONS="$tmp_dir/functions"
export ZACRS_FALLBACK_UNSAFE="$tmp_dir/unsafe"
export ZACRS_FALLBACK_WORK="$tmp_dir/work"
# The adapter's default dump path, not one the user chose.
export XDG_CACHE_HOME="$tmp_dir/cache"
unset ZSH_COMPDUMP
typeset dump="$XDG_CACHE_HOME/zsh/compdump"

mkdir -p "$tmp_dir/zdotdir" "$ZACRS_FALLBACK_FUNCTIONS" "$ZACRS_FALLBACK_WORK" \
    "$XDG_CACHE_HOME/zsh"

add_completion() {
    local dir=$1 command=$2
    print -rl -- "#compdef $command" "compadd -- $command-candidate" > "$dir/_$command"
}

cat > "$tmp_dir/zdotdir/.zshrc" <<'EOF'
fpath=("$ZACRS_FALLBACK_UNSAFE" "$ZACRS_FALLBACK_FUNCTIONS" $fpath)
typeset -g fallback_fpath="${(j.:.)fpath}"
cd "$ZACRS_FALLBACK_WORK"
source "$ZACRS_FALLBACK_UTIL"
source "$ZACRS_FALLBACK_COMPSYS"

# Collect the way the adapter does: from an ordinary widget, stderr discarded.
# A changed $fpath shows up in the result.
_zacrs_fallback_capture() {
    local fd tab=$'\t'
    exec {fd}>&2
    zle _zacrs_compsys 2>/dev/null
    exec 2>&$fd {fd}>&-
    local -a result=("${_zacrs_captured[@]%%$tab*}")
    [[ "${(j.:.)fpath}" == "$fallback_fpath" ]] || result+=("fpath changed: ${(j.:.)fpath}")
    print -rl -- "${result[@]}" > "$ZACRS_FALLBACK_MARKER"
}
zle -N _zacrs_fallback_capture
bindkey -M emacs "^T" _zacrs_fallback_capture
bindkey -M viins "^T" _zacrs_fallback_capture
PROMPT="READY> "
RPROMPT=""
EOF

# Complete `$command ` in a fresh shell and compare the captured candidates.
run_session() {
    local command=$1 expected=$2 description=$3
    : > "$ZACRS_FALLBACK_MARKER"
    ZDOTDIR="$tmp_dir/zdotdir" zpty -b zacrs_compinit_fallback zsh -d

    local output="" chunk="" i
    for (( i = 0; i < 500; i++ )); do
        zpty -r -t zacrs_compinit_fallback chunk 2>/dev/null && output+="$chunk"
        [[ "$output" == *"READY> "* ]] && break
        sleep 0.01
    done
    if [[ "$output" != *"READY> "* ]]; then
        print -u2 -r -- "not ok: child zsh did not become ready for $description"
        print -u2 -r -- "pty: ${(qqq)output}"
        return 1
    fi

    zpty -w -n zacrs_compinit_fallback "$command "$'\x14'
    for (( i = 0; i < 500; i++ )); do
        [[ "$(<"$ZACRS_FALLBACK_MARKER")" == "$expected" ]] && break
        sleep 0.01
    done

    local actual="$(<"$ZACRS_FALLBACK_MARKER")"
    while zpty -r -t zacrs_compinit_fallback chunk 2>/dev/null; do
        output+="$chunk"
    done
    zpty -d zacrs_compinit_fallback 2>/dev/null || true
    if [[ "$actual" != "$expected" ]]; then
        print -u2 -r -- "not ok: $description (expected=${(qqq)expected}, actual=${(qqq)actual})"
        print -u2 -r -- "pty: ${(qqq)output}"
        return 1
    fi
    print -r -- "ok: $description"
}

add_completion "$ZACRS_FALLBACK_FUNCTIONS" zacrsold
run_session zacrsold zacrsold-candidate 'adapter initializes compsys when compinit never ran'
[[ -f "$dump" ]] || { print -u2 -r -- 'not ok: first shell writes the dump'; exit 1 }

add_completion "$ZACRS_FALLBACK_FUNCTIONS" zacrsnew
run_session zacrsnew zacrsnew-candidate 'a later shell finds a function added after the dump'

# compaudit would prompt about a world-writable directory or remove it.
mkdir -m 777 "$ZACRS_FALLBACK_UNSAFE"
add_completion "$ZACRS_FALLBACK_UNSAFE" zacrsunsafe
run_session zacrsunsafe zacrsunsafe-candidate 'a world-writable directory stays in $fpath without a prompt'

# An up-to-date dump is loaded, not rewritten.
touch -t 200001010000 "$dump"
run_session zacrsold zacrsold-candidate 'a later shell loads an up-to-date dump'
typeset -a old_dump=($dump(Nmd+1))
(( ${#old_dump} )) || { print -u2 -r -- 'not ok: an up-to-date dump is not rewritten'; exit 1 }
print -r -- 'ok: an up-to-date dump is not rewritten'
