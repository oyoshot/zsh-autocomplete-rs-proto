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
# Register callbacks before an outer plugin captures the standard dispatcher.
# Reload must keep that dispatcher and its recursive invocation semantics.
add-zle-hook-widget line-init _zacrs_test_plain_init
add-zle-hook-widget line-init _zacrs_test_init_marker
_zacrs_test_outer_redraw() {
    zle _zacrs_test_saved_dispatcher -f nolast
}
if [[ $ZACRS_RECURSION_MODE != line-init-native ]]; then
    zle -A zle-line-pre-redraw _zacrs_test_saved_dispatcher
    zle -N zle-line-pre-redraw _zacrs_test_outer_redraw
fi
source "$ZACRS_RECURSION_PLUGIN"
_zacrs_line_pre_redraw() {
    print -r -- "dispatch:$ZLE_RECURSIVE:$BUFFER:$CURSOR" >> "$ZACRS_RECURSION_MARKER"
}

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
# interrupt_and_wait <marker>: send a terminal interrupt and wait for a marker.
# Zsh can take a terminal interrupt after its last check and before it blocks
# on the next key. It then acts on it only once more input arrives. A second
# interrupt reaches zsh while it is blocked and makes it act at once, also in
# a search minibuffer or a recursive edit, where an ordinary key would not.
# It is sent only after the normal wait has run out, because on a line that
# has already been aborted it would abort the next one too.
interrupt_and_wait() {
    local expected="$1" i
    zpty -w -n zacrs_recursion $'\x03'
    for (( i=0; i<200; i++ )); do
        drain_pty
        grep -Fqx -- "$expected" "$ZACRS_RECURSION_MARKER" 2>/dev/null && return 0
        sleep 0.01
    done
    print -u2 -r -- 'note: sent a second interrupt to release a pending one'
    zpty -w -n zacrs_recursion $'\x03'
    wait_for_marker "$expected"
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

typeset mode finish
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
            wait_for_marker 'returned:0::0'
        else
            interrupt_and_wait 'returned:1::0'
        fi
        # Returning from recursive-edit need not redraw the empty outer buffer
        # (notably on macOS). Synchronize with the outer line-init callback;
        # the following input checks depth-0 redraws and restored editor state.
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
