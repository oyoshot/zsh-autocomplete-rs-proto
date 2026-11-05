#!/usr/bin/env zsh
set -eu
source "${0:A:h:h}/_zacrs_util.zsh"

typeset -a reply
for input in '' './gs' '"gs"' '\gs' 'A=1 gs' 'command gs' \
    'gs | cat' 'gs &! echo done' 'gs &| echo done' 'gs; echo done' \
    'gs() print hello' 'gs () print hello' 'gs other() print hello' \
    'function gs print hello' 'gs() { print hello; }'; do
    if _zacrs_abbreviation_target "$input" ''; then
        print -u2 -r -- "not ok: unexpected target in ${(qqq)input}"
        exit 1
    fi
    (( ${#reply} == 0 ))
done
for prebuffer in $'cat <<\'EOF\'\n' $'print -r -- "\n'; do
    if _zacrs_abbreviation_target gs "$prebuffer"; then
        print -u2 -r -- 'not ok: continuation input has a target'
        exit 1
    fi
done

_zacrs_abbreviation_target $'  gs\t-sb "日本語 a"  ' ''
[[ "$reply[1]" == gs && "$reply[2]" == '  ' ]]
[[ "$reply[3]" == $'\t-sb "日本語 a"  ' && "$reply[4]" == 'gs -sb 日本語 a ' ]]
_zacrs_abbreviation_target 'gs "()"' ''
[[ "$reply[1]" == gs && "$reply[3]" == ' "()"' && "$reply[4]" == 'gs ()' ]]
_zacrs_abbreviation_target gs ''
[[ "$reply[1]" == gs && -z "$reply[2]" && -z "$reply[3]" && "$reply[4]" == gs ]]
print -r -- 'ok: abbreviation targets, ranges, and contexts without CLI or daemon'
