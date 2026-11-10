#!/usr/bin/env zsh

set -eu
zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-hook-registration.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT
export ZACRS_REGISTRATION_PLUGIN="${ZACRS_REGISTRATION_PLUGIN:-${test_dir:h}/zsh-autocomplete-rs.plugin.zsh}"
export ZACRS_REGISTRATION_MARKER="$tmp_dir/marker"

cat > "$tmp_dir/.zshrc" <<'RC'
HISTFILE="$ZACRS_REGISTRATION_MARKER.history"
ZACRS_BIN=false
FUNCNEST=100
_zacrs_test_ready() {
    (( ++_zacrs_test_ready_count ))
    print -r -- "ready:$_zacrs_test_ready_count" >> "$ZACRS_REGISTRATION_MARKER"
}
_zacrs_test_plain() {
    print -r -- "plain:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
_aaa_plain() {
    _zacrs_test_plain
    return 7
}
if [[ $ZACRS_REGISTRATION_MODE == captured ]]; then
    zle -N zle-line-pre-redraw _zacrs_test_plain
elif [[ $ZACRS_REGISTRATION_MODE == masked-failure ]]; then
    # This captured index-0 hook sorts before an index-0 observer callback.
    zle -N zle-line-pre-redraw _aaa_plain
fi
source "$ZACRS_REGISTRATION_PLUGIN"
zle -A zle-line-pre-redraw _zacrs_test_saved_dispatcher

if [[ $ZACRS_REGISTRATION_MODE == captured ]]; then
    # Autosuggestions can wrap the alias captured by add-zle-hook-widget, then
    # .zshrc reinstalls the original public function while the alias survives.
    zle -A user:_zacrs_test_plain _zacrs_test_saved_plain
    _zacrs_test_wrapped_plain() {
        print -r -- "before:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
        zle _zacrs_test_saved_plain -f nolast
        print -r -- "after:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
    }
    zle -N user:_zacrs_test_plain _zacrs_test_wrapped_plain
    zle -N zle-line-pre-redraw _zacrs_test_plain
    source "$ZACRS_REGISTRATION_PLUGIN"
elif [[ $ZACRS_REGISTRATION_MODE == reset* && $ZACRS_REGISTRATION_MODE != reset-init* ]]; then
    # First install a fallback around an active dispatcher delegate, then
    # remove all registered entries, with and without another zacrs reload.
    _zacrs_test_outer() {
        print -r -- "before:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
        zle _zacrs_test_saved_dispatcher -f nolast
        print -r -- "after:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
    }
    zle -N zle-line-pre-redraw _zacrs_test_outer
    source "$ZACRS_REGISTRATION_PLUGIN"
    add-zle-hook-widget -D line-pre-redraw '*'
    if [[ $ZACRS_REGISTRATION_MODE == reset ]]; then
        source "$ZACRS_REGISTRATION_PLUGIN"
    fi
elif [[ $ZACRS_REGISTRATION_MODE == masked-failure ]]; then
    _zacrs_test_mask_failure() {
        print -r -- "before:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
        local -i result=0
        zle _zacrs_test_saved_dispatcher -f nolast || result=$?
        (( result == 7 )) || return 1
        print -r -- "after:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
    }
    zle -N zle-line-pre-redraw _zacrs_test_mask_failure
    source "$ZACRS_REGISTRATION_PLUGIN"
fi

if [[ $ZACRS_REGISTRATION_MODE == reset-init* ]]; then
    add-zle-hook-widget line-init _zacrs_test_ready
    zle -A zle-line-init _zacrs_test_saved_init
    _zacrs_test_outer_init() {
        print -r -- "init-before" >> "$ZACRS_REGISTRATION_MARKER"
        zle _zacrs_test_saved_init -f nolast
        print -r -- "init-after" >> "$ZACRS_REGISTRATION_MARKER"
    }
    zle -N zle-line-init _zacrs_test_outer_init
    _zacrs_add_zle_hook line-init _zacrs_test_ready
    add-zle-hook-widget -D line-init '*'
    add-zle-hook-widget line-init _zacrs_test_ready
    # Enter integration is disabled by ZACRS_BIN=false; its remaining fallback
    # must still observe calls to the competing line-init callback after reload.
    if [[ $ZACRS_REGISTRATION_MODE == reset-init ]]; then
        source "$ZACRS_REGISTRATION_PLUGIN"
    fi
fi

# Keep the production hook lifecycle but isolate dispatch from candidate
# generation and terminal cursor queries. A later plugin uses the standard API.
_zacrs_line_pre_redraw() {
    print -r -- "complete:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
_zacrs_test_late() {
    print -r -- "late:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
add-zle-hook-widget line-pre-redraw _zacrs_test_late

# A final representative wrapper marks the end of the entire redraw, so the
# parent cannot count callbacks before a duplicate fallback finishes running.
zle -A zle-line-pre-redraw _zacrs_test_final_dispatcher
_zacrs_test_finished() {
    zle _zacrs_test_final_dispatcher -f nolast
    print -r -- "done:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
zle -N zle-line-pre-redraw _zacrs_test_finished
add-zle-hook-widget line-init _zacrs_test_ready
bindkey '^C' .send-break
PROMPT='READY> '
RPROMPT=''
RC

typeset pty_output=""
drain_pty() {
    local chunk
    while zpty -r -t zacrs_registration chunk 2>/dev/null; do
        pty_output+="$chunk"
    done
    return 0
}
wait_for_marker() {
    local expected="$1" i
    for (( i=0; i<200; i++ )); do
        drain_pty
        if grep -Fqx -- "$expected" "$ZACRS_REGISTRATION_MARKER"; then
            return 0
        fi
        sleep 0.01
    done
    print -u2 -r -- "not ok: timed out waiting for ${(qqq)expected} ($ZACRS_REGISTRATION_MODE)"
    cat "$ZACRS_REGISTRATION_MARKER" >&2
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    return 1
}
assert_event() {
    local buffer="$1" expected actual
    if [[ $ZACRS_REGISTRATION_MODE == captured ]]; then
        expected="before:$buffer,plain:$buffer,after:$buffer,complete:$buffer,late:$buffer,done:$buffer"
    elif [[ $ZACRS_REGISTRATION_MODE == reset ]]; then
        expected="before:$buffer,complete:$buffer,late:$buffer,after:$buffer,done:$buffer"
    elif [[ $ZACRS_REGISTRATION_MODE == reset-live ]]; then
        # Standard removal intentionally disables zacrs until its next reload.
        expected="before:$buffer,late:$buffer,after:$buffer,done:$buffer"
    elif [[ $ZACRS_REGISTRATION_MODE == masked-failure ]]; then
        # A failed first callback aborts the list even when its wrapper succeeds.
        expected="before:$buffer,plain:$buffer,after:$buffer,done:$buffer"
    else
        expected="complete:$buffer,late:$buffer,done:$buffer"
    fi
    local -a entries
    entries=("${(@f)$(grep -E "^(before|plain|after|complete|late|done):$buffer$" "$ZACRS_REGISTRATION_MARKER")}")
    actual="${(j:,:)entries}"
    [[ "$actual" == "$expected" ]] || {
        print -u2 -r -- "not ok: $ZACRS_REGISTRATION_MODE must dispatch each hook once in order"
        print -u2 -r -- "expected: $expected"
        print -u2 -r -- "actual:   $actual"
        return 1
    }
}
start_shell() {
    ZDOTDIR="$tmp_dir" zpty -b zacrs_registration zsh -d
}

typeset -a modes
if (( $# )); then
    modes=("$@")
else
    modes=(captured reset reset-init reset-live reset-init-live masked-failure)
fi
typeset mode
for mode in "${modes[@]}"; do
    export ZACRS_REGISTRATION_MODE="$mode"
    : > "$ZACRS_REGISTRATION_MARKER"
    pty_output=""
    start_shell
    wait_for_marker 'ready:1'
    zpty -w -n zacrs_registration a
    wait_for_marker 'done:a'
    assert_event a
    zpty -w -n zacrs_registration b
    wait_for_marker 'done:ab'
    assert_event ab
    zpty -w -n zacrs_registration $'\x03'
    wait_for_marker 'ready:2'
    zpty -w -n zacrs_registration c
    wait_for_marker 'done:c'
    assert_event c
    # Exactly one init callback for startup and one for cancellation recovery.
    [[ $(grep -c '^ready:' "$ZACRS_REGISTRATION_MARKER") == 2 ]] || {
        print -u2 -r -- "not ok: line-init callbacks ran more than once per event"
        cat "$ZACRS_REGISTRATION_MARKER" >&2
        return 1
    }
    zpty -d zacrs_registration
    print -r -- "ok: $mode reload preserves once-per-event hooks and wrapper order"
done
