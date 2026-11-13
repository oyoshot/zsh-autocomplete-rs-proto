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
autoload -Uz add-zle-hook-widget
_test_plain() {
    print -r -- "plain:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
    [[ $ZACRS_REGISTRATION_MODE != failure ]]
}
_test_outer() {
    print -r -- "before:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
    local result=0
    zle _test_saved_dispatcher -f nolast || result=$?
    print -r -- "after:$BUFFER:$result" >> "$ZACRS_REGISTRATION_MARKER"
    return $result
}
zle -N zle-line-pre-redraw _test_plain
if [[ $ZACRS_REGISTRATION_MODE == prewrapped ]]; then
    _test_existing() { :; }
    add-zle-hook-widget line-pre-redraw _test_existing
    zle -A zle-line-pre-redraw _test_saved_dispatcher
    zle -N zle-line-pre-redraw _test_outer
fi
if [[ $ZACRS_REGISTRATION_MODE == legacy ]]; then
    _zacrs_self_insert_wrapped=0
    _test_legacy_public=${widgets[zle-line-pre-redraw]}
    _test_legacy_result=0
    source "$ZACRS_REGISTRATION_PLUGIN" >/dev/null 2>&1 || _test_legacy_result=$?
    if (( _test_legacy_result != 1 || ${+_zacrs_integration_version} ||
          ${+functions[_zacrs_wrap_self_insert]} )) ||
        [[ ${widgets[zle-line-pre-redraw]} != $_test_legacy_public ]]; then
        print -r -- ownership:changed >> "$ZACRS_REGISTRATION_MARKER"
    fi
    unset _zacrs_self_insert_wrapped
fi
source "$ZACRS_REGISTRATION_PLUGIN"
_test_late() { print -r -- "late:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"; }
# Standard registration works while the public entry is the dispatcher.
if [[ $ZACRS_REGISTRATION_MODE != prewrapped ]]; then
    add-zle-hook-widget line-pre-redraw _test_late
    zle -A zle-line-pre-redraw _test_saved_dispatcher
    zle -N zle-line-pre-redraw _test_outer
fi
if [[ $ZACRS_REGISTRATION_MODE == replaced ]]; then
    zle -N zle-line-pre-redraw _test_plain
elif [[ $ZACRS_REGISTRATION_MODE == removed ]]; then
    add-zle-hook-widget -d line-pre-redraw _zacrs_line_pre_redraw
fi
# Also preserve user key choices and wrappers of accept/cancel on reload.
zle -A accept-line _test_accept
_test_accept_outer() { zle _test_accept; }
zle -N accept-line _test_accept_outer
zle -A send-break _test_break
_test_break_outer() { zle _test_break; }
zle -N send-break _test_break_outer
bindkey '^I' expand-or-complete
_test_public=${widgets[zle-line-pre-redraw]}
_test_saved=${widgets[_test_saved_dispatcher]}
_test_body=${functions[azhw:zle-line-pre-redraw]}
zstyle -a zle-line-pre-redraw widgets _test_callbacks
for iteration in {1..10}; do
    source "$ZACRS_REGISTRATION_PLUGIN"
done
zstyle -a zle-line-pre-redraw widgets _test_after
if [[ $widgets[zle-line-pre-redraw] != $_test_public ||
      $widgets[_test_saved_dispatcher] != $_test_saved ||
      $functions[azhw:zle-line-pre-redraw] != $_test_body ||
      "${(j:,:)_test_callbacks}" != "${(j:,:)_test_after}" ||
      $widgets[accept-line] != user:_test_accept_outer ||
      $widgets[send-break] != user:_test_break_outer ||
      $(bindkey '^I') != *' expand-or-complete' ]]; then
    print -r -- 'ownership:changed' >> "$ZACRS_REGISTRATION_MARKER"
fi
_zacrs_line_pre_redraw() {
    print -r -- "complete:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
_test_ready() { print -r -- "ready:$(( ++_test_ready_count ))" >> "$ZACRS_REGISTRATION_MARKER"; }
add-zle-hook-widget line-init _test_ready
zle -A zle-line-pre-redraw _test_finished_delegate
_test_finished() {
    zle _test_finished_delegate -f nolast
    print -r -- "done:$BUFFER" >> "$ZACRS_REGISTRATION_MARKER"
}
zle -N zle-line-pre-redraw _test_finished
PROMPT='READY> '
RPROMPT=''
RC

typeset pty_output=''
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
        grep -Fqx -- "$expected" "$ZACRS_REGISTRATION_MARKER" && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: waiting for $expected ($ZACRS_REGISTRATION_MODE)"
    cat "$ZACRS_REGISTRATION_MARKER" >&2
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    return 1
}
assert_event() {
    local buffer="$1" expected
    case $ZACRS_REGISTRATION_MODE in
        wrapped|legacy) expected="before:$buffer,plain:$buffer,complete:$buffer,late:$buffer,after:$buffer:0,done:$buffer" ;;
        removed) expected="before:$buffer,plain:$buffer,late:$buffer,after:$buffer:0,done:$buffer" ;;
        replaced) expected="plain:$buffer,done:$buffer" ;;
        prewrapped|legacy) expected="before:$buffer,plain:$buffer,complete:$buffer,after:$buffer:0,done:$buffer" ;;
        failure) expected="before:$buffer,plain:$buffer,after:$buffer:1,done:$buffer" ;;
    esac
    local -a entries
    entries=("${(@f)$(grep -E "^(before|plain|complete|late|after|done):$buffer(:[01])?$" "$ZACRS_REGISTRATION_MARKER")}")
    [[ "${(j:,:)entries}" == "$expected" ]] || {
        print -u2 -r -- "not ok: $ZACRS_REGISTRATION_MODE order: ${(j:,:)entries}; expected $expected"
        return 1
    }
}
start_shell() {
    ZDOTDIR="$tmp_dir" zpty -b zacrs_registration zsh -d
}
for mode in wrapped removed replaced prewrapped failure legacy; do
    export ZACRS_REGISTRATION_MODE=$mode
    : > "$ZACRS_REGISTRATION_MARKER"
    pty_output=''
    start_shell
    wait_for_marker ready:1
    zpty -w -n zacrs_registration q
    wait_for_marker done:q
    assert_event q
    zpty -w -n zacrs_registration $'\x03'
    wait_for_marker ready:2
    zpty -w -n zacrs_registration r
    wait_for_marker done:r
    assert_event r
    ! grep -Fqx ownership:changed "$ZACRS_REGISTRATION_MARKER"
    zpty -d zacrs_registration
    print -r -- "ok: $mode preserves hook ownership, order and cancellation after reload"
done
