#!/usr/bin/env zsh

setopt errexit nounset pipefail

typeset -r test_dir=${0:A:h}
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

(( failures == 0 ))
