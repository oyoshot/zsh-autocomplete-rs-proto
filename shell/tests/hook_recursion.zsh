#!/usr/bin/env zsh

set -eu
zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-hook-recursion.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT
export ZACRS_RECURSION_PLUGIN="${test_dir:h}/zsh-autocomplete-rs.plugin.zsh"
export ZACRS_RECURSION_MARKER="$tmp_dir/marker"

cat > "$tmp_dir/.zshrc" <<'ZSH'
HISTFILE="$ZACRS_RECURSION_MARKER.history"
ZACRS_BIN=false
source "$ZACRS_RECURSION_PLUGIN"
FUNCNEST=100

# Keep real ZLE and production hook registration/dispatch, replacing only the
# completion callback so this lifecycle regression needs no daemon or terminal
# cursor-query responses.
_zacrs_line_pre_redraw() {
    print -r -- "dispatch:$ZLE_RECURSIVE:$BUFFER:$CURSOR" >> "$ZACRS_RECURSION_MARKER"
}
_zacrs_test_init_marker() {
    print -r -- "init:$ZLE_RECURSIVE" >> "$ZACRS_RECURSION_MARKER"
}
_zacrs_test_recurse() {
    local saved_buffer="$BUFFER" saved_cursor=$CURSOR
    local -i recursive_result=0
    _zacrs_test_recursed=1
    BUFFER=""
    CURSOR=0
    zle recursive-edit || recursive_result=$?
    BUFFER="$saved_buffer"
    CURSOR=$saved_cursor
    print -r -- "returned:$recursive_result:$BUFFER:$CURSOR" >> "$ZACRS_RECURSION_MARKER"
    return 0
}
_zacrs_test_plain_init() {
    if [[ $ZACRS_RECURSION_MODE == line-init* ]] && (( ! ZLE_RECURSIVE && ! _zacrs_test_recursed )); then
        _zacrs_test_recurse
    fi
    return 0
}
_zacrs_test_plain_redraw() { return 0 }
# Stale dispatcher aliases require a fallback around both plain replacements.
add-zle-hook-widget line-init _zacrs_test_init_marker
zle -A zle-line-init _zacrs_test_stale_init
zle -N zle-line-init _zacrs_test_plain_init
_zacrs_add_zle_hook line-init _zacrs_test_init_marker
zle -A zle-line-pre-redraw _zacrs_test_stale_redraw
zle -N zle-line-pre-redraw _zacrs_test_plain_redraw
_zacrs_add_zle_hook line-pre-redraw _zacrs_line_pre_redraw

# A further plugin wrapper must share the observation for this invocation,
# while recursive editing and its successive redraws each need fresh state.
_zacrs_test_outer_redraw() {
    zle _zacrs_test_saved_fallback -f nolast
}
zle -A zle-line-pre-redraw _zacrs_test_saved_fallback
zle -N zle-line-pre-redraw _zacrs_test_outer_redraw
_zacrs_add_zle_hook line-pre-redraw _zacrs_line_pre_redraw

# Model a reload retaining observer styles written by the previous version.
for hook in zle-line-init zle-line-pre-redraw; do
    zstyle -a "$hook" widgets entries
    entries=("${(@)entries/:user:_zacrs_observe_zle_hook/:_zacrs_observe_zle_hook}")
    zstyle "$hook" widgets "${entries[@]}"
done
zle -N _zacrs_observe_zle_hook
source "$ZACRS_RECURSION_PLUGIN"
_zacrs_line_pre_redraw() {
    print -r -- "dispatch:$ZLE_RECURSIVE:$BUFFER:$CURSOR" >> "$ZACRS_RECURSION_MARKER"
}
# A native redraw during a wrapped line-init must not count as line-init.
if [[ $ZACRS_RECURSION_MODE == line-init-native ]]; then
    _zacrs_publish_zle_hook zle-line-pre-redraw "${functions[_zacrs_standard_zle_line-pre-redraw]}"
fi

# Exercise the same hook at the inner depth while the outer line-init call is
# suspended. Its observation must not suppress the outer fallback on return.
_zacrs_test_inner_init() {
    zle zle-line-init -w -f nolast
}
zle -N _zacrs_test_inner_init
bindkey '^X' _zacrs_test_inner_init
bindkey '^M' .accept-line
bindkey '^C' .send-break
PROMPT='READY> '
RPROMPT=''
ZSH

typeset pty_output=""
drain_pty() {
    local chunk
    while zpty -r -t zacrs_recursion chunk 2>/dev/null; do
        pty_output+="$chunk"
    done
    return 0
}
wait_for_marker() {
    local expected="$1" i
    for (( i=0; i<200; i++ )); do
        drain_pty
        if grep -Fqx -- "$expected" "$ZACRS_RECURSION_MARKER"; then
            return 0
        fi
        sleep 0.01
    done
    print -u2 -r -- "not ok: timed out waiting for ${(qqq)expected} ($ZACRS_RECURSION_MODE)"
    cat "$ZACRS_RECURSION_MARKER" >&2
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    return 1
}
assert_once() {
    local expected="$1"
    [[ $(grep -Fxc -- "$expected" "$ZACRS_RECURSION_MARKER") == 1 ]] || {
        print -u2 -r -- "not ok: expected one invocation of ${(qqq)expected}"
        cat "$ZACRS_RECURSION_MARKER" >&2
        return 1
    }
}

start_shell() {
    ZDOTDIR="$tmp_dir" zpty -b zacrs_recursion zsh -d
}

typeset mode finish expected_result
for mode in line-init line-init-native; do
    export ZACRS_RECURSION_MODE="$mode"
    for finish in accept cancel; do
        : > "$ZACRS_RECURSION_MARKER"
        pty_output=""
        start_shell
        wait_for_marker 'dispatch:1::0'
        zpty -w -n zacrs_recursion a
        wait_for_marker 'dispatch:1:a:1'
        zpty -w -n zacrs_recursion b
        wait_for_marker 'dispatch:1:ab:2'
        # Accepting the line may legitimately invoke a final redraw of ab.
        # Check once-per-event before sending the distinct terminating event.
        assert_once 'dispatch:1:a:1'
        assert_once 'dispatch:1:ab:2'
        zpty -w -n zacrs_recursion $'\x18'
        wait_for_marker 'init:1'
        if [[ $finish == accept ]]; then
            zpty -w -n zacrs_recursion $'\r'
            expected_result=0
        else
            zpty -w -n zacrs_recursion $'\x03'
            expected_result=1
        fi
        wait_for_marker "returned:$expected_result::0"
        wait_for_marker 'dispatch:0::0'
        wait_for_marker 'init:0'
        zpty -w -n zacrs_recursion c
        wait_for_marker 'dispatch:0:c:1'
        zpty -w -n zacrs_recursion d
        wait_for_marker 'dispatch:0:cd:2'
        assert_once 'dispatch:0:c:1'
        assert_once 'dispatch:0:cd:2'
        zpty -d zacrs_recursion
        print -r -- "ok: $mode recursive editing dispatches each redraw and restores state after $finish"
    done
done
