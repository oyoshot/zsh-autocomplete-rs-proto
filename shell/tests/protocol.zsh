#!/usr/bin/env zsh

setopt errexit nounset pipefail

typeset -r test_dir=${0:A:h}
source "${test_dir:h}/_zacrs_protocol.zsh"

typeset -i failures=0

assert_equal() {
    local expected="$1" actual="$2" description="$3"
    if [[ "$actual" != "$expected" ]]; then
        print -u2 -r -- "not ok: ${description} (expected=${(qqq)expected}, actual=${(qqq)actual})"
        (( ++failures ))
    else
        print -r -- "ok: ${description}"
    fi
}

result_text="legacy"
chain=0
execute=0
restore_text=""
cursor_offset=""
_zacrs_parse_apply_line \
    "APPLY chain=1 execute=0 cursor_offset=3 text_hex=666f6f0a62617220 restore_hex=62617a0a"

assert_equal $'foo\nbar ' "$result_text" 'text hex preserves newline and trailing space'
assert_equal $'baz\n' "$restore_text" 'restore hex preserves trailing newline'
assert_equal 1 "$chain" 'chain flag is parsed'
assert_equal 0 "$execute" 'execute flag is parsed'
assert_equal 3 "$cursor_offset" 'cursor offset is parsed'

result_text="legacy result"
_zacrs_parse_apply_line "APPLY chain=0 execute=1 restore=legacy restore"
assert_equal "legacy result" "$result_text" 'legacy result text remains unchanged'
assert_equal "legacy restore" "$restore_text" 'legacy restore field is parsed'

(( failures == 0 ))
