#!/usr/bin/env zsh

set -eu

zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset plugin_path="${test_dir:h}/zsh-autocomplete-rs.plugin.zsh"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-typeahead.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT

export ZACRS_TYPEAHEAD_PLUGIN="$plugin_path"
export ZACRS_TYPEAHEAD_MARKER="$tmp_dir/marker"

print -r -- '
ZACRS_BIN=false
# Model a .zshrc that registers its own hook before sourcing zacrs, including
# when that same configuration is sourced again.
_zacrs_test_plain_pre_redraw() {
    print -r -- "plain:$BUFFER" >> "$ZACRS_TYPEAHEAD_MARKER"
}
zle -N zle-line-pre-redraw _zacrs_test_plain_pre_redraw
source "$ZACRS_TYPEAHEAD_PLUGIN"

# Keep the test focused on ZLE batching. Candidate generation, cursor queries,
# terminal drawing, and the Rust popup session are covered separately.
_zacrs_clear_popup() {
    _zacrs_popup_visible=0
    _zacrs_reset_popup_snapshot
}
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    _zacrs_captured=("abcdef\tcommand" "abcdefg\tcommand")
}
# The plugin registers this widget with _zacrs_compsys_func. Defining a shell
# function alone does not replace that registration.
zle -N _zacrs_compsys
_zacrs_gather() {
    print -r -- $'"'"'abcdef\tcommand\nabcdefg\tcommand'"'"'
}
_zacrs_should_render_candidates() { return 0 }
_zacrs_render() {
    print -r -- "render:$LBUFFER" >> "$ZACRS_TYPEAHEAD_MARKER"
    _zacrs_prev_lbuffer="$LBUFFER"
}
_zacrs_complete_popup() {
    print -r -- "tab:$LBUFFER" >> "$ZACRS_TYPEAHEAD_MARKER"
    BUFFER=""
    zle .accept-line
}
zle -N _zacrs_complete_popup

probe() {
    print -r -- "probe:ok" >> "$ZACRS_TYPEAHEAD_MARKER"
}

# A second plugin may wrap self-insert while zacrs has deferred a redraw.
# Its saved alias must remain callable after zacrs is sourced and deferred again.
_zacrs_test_outer_self_insert() {
    zle _zacrs_test_outer_original -w
}
_zacrs_test_outer_pre_redraw() {
    zle _zacrs_test_original_pre_redraw
}
_zacrs_test_outer_line_init() {
    zle _zacrs_test_original_line_init
}
_zacrs_test_wrap_and_reload() {
    _zacrs_defer_redraw
    zle -A self-insert _zacrs_test_outer_original
    zle -N self-insert _zacrs_test_outer_self_insert
    # Autosuggestions wraps the hook dispatcher after startup. Registering the
    # hook again must not save this wrapper as a child of that same dispatcher.
    zle -A zle-line-pre-redraw _zacrs_test_original_pre_redraw
    zle -N zle-line-pre-redraw _zacrs_test_outer_pre_redraw
    # Also cover removing the last line-init entry, then enabling it again.
    _zacrs_add_zle_hook line-init _zacrs_test_line_init
    zle -A zle-line-init _zacrs_test_original_line_init
    zle -N zle-line-init _zacrs_test_outer_line_init
    add-zle-hook-widget -d line-init _zacrs_test_line_init
    source "$ZACRS_TYPEAHEAD_PLUGIN"
    _zacrs_add_zle_hook line-init _zacrs_test_line_init
    _zacrs_test_line_init() {
        print -r -- "line-init:reloaded" >> "$ZACRS_TYPEAHEAD_MARKER"
    }
    [[ "$widgets[self-insert]" == user:_zacrs_test_outer_self_insert ]] \
        && print -r -- "wrapper:preserved" >> "$ZACRS_TYPEAHEAD_MARKER"
    [[ "$widgets[zle-line-pre-redraw]" == user:_zacrs_test_outer_pre_redraw ]] \
        && print -r -- "hook-wrapper:preserved" >> "$ZACRS_TYPEAHEAD_MARKER"
    [[ "$widgets[zle-line-init]" == user:_zacrs_test_outer_line_init ]] \
        && print -r -- "line-init-wrapper:preserved" >> "$ZACRS_TYPEAHEAD_MARKER"
    _zacrs_line_pre_redraw() {
        print -r -- "reloaded:$BUFFER" >> "$ZACRS_TYPEAHEAD_MARKER"
    }
    _zacrs_defer_redraw
}
zle -N _zacrs_test_wrap_and_reload
bindkey "^X" _zacrs_test_wrap_and_reload
_zacrs_test_replace_and_reload() {
    zle -N zle-line-pre-redraw _zacrs_test_plain_pre_redraw
    source "$ZACRS_TYPEAHEAD_PLUGIN"
    _zacrs_line_pre_redraw() {
        print -r -- "reattached:$BUFFER" >> "$ZACRS_TYPEAHEAD_MARKER"
    }
}
zle -N _zacrs_test_replace_and_reload
bindkey "^Y" _zacrs_test_replace_and_reload
FUNCNEST=100

# A printed prompt can precede ZLE entering raw mode on the PTY.
# Wait for the editor itself before sending a batch of input.
_zacrs_test_line_init() {
    print -r -- "editor:ready" >> "$ZACRS_TYPEAHEAD_MARKER"
}
add-zle-hook-widget line-init _zacrs_test_line_init

PROMPT="READY> "
RPROMPT=""
' > "$tmp_dir/.zshrc"

typeset pty_output=""

drain_pty() {
    local chunk=""
    while zpty -r -t zacrs_typeahead chunk 2>/dev/null; do
        pty_output+="$chunk"
    done
    return 0
}

wait_for_marker() {
    local expected="$1"
    local i
    for (( i = 0; i < 200; i++ )); do
        # Drain output while waiting: otherwise the child can block writing to
        # its PTY before it reaches the marker (notably on macOS).
        drain_pty
        [[ -f "$ZACRS_TYPEAHEAD_MARKER" ]] \
            && grep -Fqx -- "$expected" "$ZACRS_TYPEAHEAD_MARKER" \
            && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: timed out waiting for ${(qqq)expected}"
    [[ -f "$ZACRS_TYPEAHEAD_MARKER" ]] && sed 's/^/marker: /' "$ZACRS_TYPEAHEAD_MARKER" >&2
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    return 1
}

start_shell() {
    : > "$ZACRS_TYPEAHEAD_MARKER"
    pty_output=""
    ZDOTDIR="$tmp_dir" zpty -b zacrs_typeahead zsh -d
    local output=""
    local i
    for (( i = 0; i < 200; i++ )); do
        local chunk=""
        zpty -r -t zacrs_typeahead chunk 2>/dev/null && output+="$chunk"
        if [[ "$output" == *"READY> "* ]]; then
            wait_for_marker "editor:ready"
            return $?
        fi
        sleep 0.01
    done
    print -u2 -r -- "not ok: child zsh did not become ready"
    return 1
}

stop_shell() {
    zpty -d zacrs_typeahead 2>/dev/null || true
}

# One PTY write models a type-ahead (Shinsoku-typing) batch rather than
# human-paced keypresses.
start_shell
zpty -w -n zacrs_typeahead "abcdef"
wait_for_marker "render:abcdef"
stop_shell
print -r -- "ok: batched self-insert renders the final buffer"

# Tab is included in the same write. The following command proves that the
# temporary self-insert wrapper was restored and ordinary input still works.
start_shell
zpty -w -n zacrs_typeahead $'abcdef\tprobe\n'
wait_for_marker "tab:abcdef"
wait_for_marker "probe:ok"

drain_pty
if [[ "$pty_output" == *"No such widget"* ]]; then
    print -u2 -r -- "not ok: type-ahead Tab left a broken widget"
    return 1
fi
stop_shell
print -r -- "ok: batched Tab restores self-insert for the next command"

start_shell
zpty -w -n zacrs_typeahead $'\x19q'
wait_for_marker "reattached:q"
wait_for_marker "plain:q"
zpty -w -n zacrs_typeahead $'\x19r'
wait_for_marker "reattached:qr"
wait_for_marker "plain:qr"
stop_shell
print -r -- "ok: reload reattaches a replaced hook and preserves the plain widget"

start_shell
zpty -w -n zacrs_typeahead $'\x18q'
wait_for_marker "wrapper:preserved"
wait_for_marker "hook-wrapper:preserved"
wait_for_marker "line-init-wrapper:preserved"
wait_for_marker "reloaded:q"
zpty -w -n zacrs_typeahead r
wait_for_marker "reloaded:qr"
zpty -w -n zacrs_typeahead $'\x03'
wait_for_marker "line-init:reloaded"
drain_pty
if [[ "$pty_output" == *"maximum nested function level"* ]]; then
    print -u2 -r -- "not ok: reloading with another self-insert wrapper caused recursion"
    return 1
fi
stop_shell
print -r -- "ok: reload preserves another plugin wrapper without recursive self-insert"
print -r -- "ok: reload preserves a wrapped hook dispatcher without recursive redraw"
