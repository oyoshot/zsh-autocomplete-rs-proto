#!/usr/bin/env zsh

setopt errexit nounset pipefail

typeset -r test_dir=${0:A:h}
source "${test_dir:h}/_zacrs_util.zsh"
source "${test_dir:h}/_zacrs_compsys.zsh"

typeset -i failures=0

assert_file_flag() {
    local expected=$1 description=$2
    shift 2

    local actual=0
    _zacrs_compadd_has_file_flag "$@" && actual=1
    if (( actual != expected )); then
        print -u2 -r -- "not ok: ${description} (expected=${expected}, actual=${actual})"
        (( ++failures ))
    else
        print -r -- "ok: ${description}"
    fi
}

assert_file_flag 1 'standalone file flag' -f candidate
assert_file_flag 1 'combined file flag' -Qf candidate
assert_file_flag 1 'multiple combined file flags' -QUf candidate
assert_file_flag 1 'file flag before attached option value' -fP/foo candidate
assert_file_flag 1 'file flag after array mode' -a -Qf array_name
assert_file_flag 1 'file flag after associative-array mode' -k -Qf array_name
assert_file_flag 1 'file flag after a separate option value' -i -foo -Qf candidate

assert_file_flag 0 'attached prefix value containing f' -P/foo candidate
assert_file_flag 0 'attached suffix value containing f' -Sfoo candidate
assert_file_flag 0 'separate ignored prefix containing f' -i -foo candidate
assert_file_flag 0 'separate ignored suffix containing f' -I -foo candidate
assert_file_flag 0 'value after combined suffix option containing f' -Qs -foo candidate
assert_file_flag 0 'candidate after option terminator' - -foo
assert_file_flag 0 'candidate after double option terminator' -- -foo

assert_array_output_flag() {
    local expected=$1 description=$2
    shift 2

    local actual=0
    _zacrs_compadd_has_array_output_flag "$@" && actual=1
    if (( actual != expected )); then
        print -u2 -r -- "not ok: ${description} (expected=${expected}, actual=${actual})"
        (( ++failures ))
    else
        print -r -- "ok: ${description}"
    fi
}

assert_array_output_flag 1 'standalone capture output flag' -O matches candidate
assert_array_output_flag 1 'combined capture output flag' -QO matches candidate
assert_array_output_flag 1 'standalone display output flag' -D displays candidate
assert_array_output_flag 1 'combined display output flag' -QD displays candidate
assert_array_output_flag 1 'standalone transformed output flag' -A matches candidate
assert_array_output_flag 1 'combined transformed output flag' -QA matches candidate
assert_array_output_flag 0 'output flag after option terminator is a candidate' - -O
assert_array_output_flag 0 'option value containing O is not an output flag' -P -Output candidate
assert_array_output_flag 0 'option value containing A is not an output flag' -P -Array candidate

assert_path_kind() {
    local expected=$1 path=$2 description=$3
    _zacrs_path_candidate_kind "$path"
    if [[ "$REPLY" != "$expected" ]]; then
        print -u2 -r -- "not ok: ${description} (expected=${expected}, actual=${REPLY})"
        (( ++failures ))
    else
        print -r -- "ok: ${description}"
    fi
}

assert_path_kind directory "$test_dir" 'directory candidate keeps directory kind'
assert_path_kind file "$0" 'file candidate keeps file kind'
HOME="$test_dir" assert_path_kind directory '~/' 'home-relative directory candidate keeps directory kind'
HOME="$test_dir" assert_path_kind file '~/compsys_options.zsh' 'home-relative file candidate keeps file kind'

assert_render_gate() {
    local expected=$1 candidates=$2 buffer=$3 prefix=$4 description=$5
    local actual=0
    _zacrs_should_render_candidates "$candidates" "$buffer" "$prefix" && actual=1
    if (( actual != expected )); then
        print -u2 -r -- "not ok: ${description} (expected=${expected}, actual=${actual})"
        (( ++failures ))
    else
        print -r -- "ok: ${description}"
    fi
}

assert_render_gate 1 '' gpo gpo 'command position reaches configured abbreviations without shell candidates'
assert_render_gate 1 '' 'git gpo' gpo 'argument position reaches configured abbreviations without shell candidates'
assert_render_gate 1 $'git\tcommand\tcommand' 'git gpo' gpo 'shell candidates render in argument position'

(( failures == 0 ))
