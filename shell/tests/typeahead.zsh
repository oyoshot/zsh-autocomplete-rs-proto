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

PROMPT="READY> "
RPROMPT=""
' > "$tmp_dir/.zshrc"

wait_for_marker() {
    local expected="$1"
    local i
    for (( i = 0; i < 200; i++ )); do
        [[ -f "$ZACRS_TYPEAHEAD_MARKER" ]] \
            && grep -Fqx -- "$expected" "$ZACRS_TYPEAHEAD_MARKER" \
            && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: timed out waiting for ${(qqq)expected}"
    [[ -f "$ZACRS_TYPEAHEAD_MARKER" ]] && sed 's/^/marker: /' "$ZACRS_TYPEAHEAD_MARKER" >&2
    return 1
}

start_shell() {
    : > "$ZACRS_TYPEAHEAD_MARKER"
    ZDOTDIR="$tmp_dir" zpty -b zacrs_typeahead zsh -d
    local output=""
    local i
    for (( i = 0; i < 200; i++ )); do
        local chunk=""
        zpty -r -t zacrs_typeahead chunk 2>/dev/null && output+="$chunk"
        [[ "$output" == *"READY> "* ]] && return 0
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

typeset output="" chunk=""
while zpty -r -t zacrs_typeahead chunk 2>/dev/null; do
    output+="$chunk"
done
if [[ "$output" == *"No such widget"* ]]; then
    print -u2 -r -- "not ok: type-ahead Tab left a broken widget"
    return 1
fi
stop_shell
print -r -- "ok: batched Tab restores self-insert for the next command"
