set -eu
typeset _test_root="$HOME"
zmodload zsh/zpty
mkdir -p "$HOME/files"
: > "$HOME/files/space name"
: > "$HOME/files/日本語 file"
: > "$HOME/files/"$'tab\tname'
: > "$HOME/files/"$'line\nname'
: > "$HOME/files/dollar\$(false)"
: > "$HOME/files/trailing "
mkdir -p "$HOME/new-home/files"
: > "$HOME/new-home/files/space new-home"
mkdir -p "$HOME/files/directory"
cat > "$HOME/.zshrc" <<'RC'
unsetopt MONITOR
_test_root="$HOME"
stty rows 24 cols 100
bindkey -e
autoload -Uz compinit
compinit -C
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    zmodload() {
        [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
        builtin zmodload "$@"
    }
fi
source "$ZACRS_PLUGIN"
_test_production_redraw="$functions[_zacrs_line_pre_redraw]"
_zacrs_line_pre_redraw() { return 0; }
# Only the provider is empty; the native completion parser and capture widget run.
_main_complete() { return 0; }
cd "$HOME/files"
_test_index=0
_test_run() {
    (( ++_test_index ))
    local expected='' valid=1 select=1 accept_single=1
    case $_test_index in
        1) BUFFER='probe space'; expected='space name ' ;;
        2) BUFFER='probe 日本'; expected='日本語 file ' ;;
        3) BUFFER='probe tab'; expected=$'tab\tname ' ;;
        4) BUFFER='probe line'; expected=$'line\nname ' ;;
        5) BUFFER='probe dollar'; expected='dollar$(false) ' ;;
        6) BUFFER='probe dire'; expected='directory/' ;;
        7) BUFFER='probe ~/files/spa'; expected="$HOME/files/space name " ;;
        8) BUFFER='probe "space'; valid=0 ;;
        9) BUFFER='probe $HOME'; valid=0 ;;
        10) BUFFER='probe spa*'; valid=0 ;;
        11) BUFFER='probe space tail'; valid=0 ;;
        12) BUFFER='probe '; expected=''; select=0 ;;
        13) BUFFER='probe trail'; expected='trailing  ' ;;
        14) BUFFER='probe trail'; expected='trailing  '; accept_single=0; _zacrs_cursor_stale=' ' ;;
        15|16|17|18|19|20|21|22) select=0 ;;
    esac
    if (( _test_index == 15 )); then
        BUFFER='probe space'; CURSOR=${#BUFFER}
        local before="$BUFFER" saved_bin="$ZACRS_BIN" code=0
        _test_malformed_bin() { print -r -- 'DONE 0 changed'; print -r -- 'APPLY chain=1 execute=0 path_hex=61 suffix_hex=GG restore_hex='; }
        ZACRS_BIN=_test_malformed_bin
        _zacrs_invoke space 5 '' 2 1 1 || code=$?
        ZACRS_BIN="$saved_bin"
        [[ $code == 5 && "$BUFFER" == "$before" ]] || { print -r -- malformed-applied > "$_test_root/state"; return; }
    elif (( _test_index == 16 )); then
        BUFFER=gs; CURSOR=2
        local before="$BUFFER" saved_request="$functions[_zacrs_abbreviation_request]" code=0
        _zacrs_abbreviation_request() { _zacrs_abbreviation_header='DONE 0 changed'; _zacrs_abbreviation_apply='APPLY chain=0 execute=0 text_hex=GG restore_hex='; }
        _zacrs_expand_command_abbreviation || code=$?
        functions[_zacrs_abbreviation_request]="$saved_request"
        [[ $code == 1 && "$BUFFER" == "$before" ]] || { print -r -- malformed-abbreviation > "$_test_root/state"; return; }
    elif (( _test_index == 17 )); then
        BUFFER='probe space'; CURSOR=${#BUFFER}
        _zacrs_filesystem_requested=1
        _zacrs_popup_visible=1
        _zacrs_record_popup_snapshot 5
        _zacrs_prev_lbuffer="$LBUFFER"; _zacrs_prev_buffer="$BUFFER"; _zacrs_prev_cursor=$CURSOR
        _zacrs_current_context_key; local before_key="$REPLY"
        BUFFER='probe spaceXYZ'; CURSOR=11
        _zacrs_current_context_key
        [[ "$REPLY" != "$before_key" ]] || { print -r -- stale-key > "$_test_root/state"; return; }
        local saved_invoke="$functions[_zacrs_invoke_daemon]" original="$BUFFER"
        _zacrs_daemon_available=1
        _zacrs_invoke_daemon() { print -r -- "$_zacrs_filesystem_requested:$6" > "$HOME/reuse"; }
        _zacrs_complete_popup
        functions[_zacrs_invoke_daemon]="$saved_invoke"
        [[ "$(<"$HOME/reuse")" == '0:0' && "$BUFFER" == "$original" ]] || { print -r -- stale-snapshot > "$_test_root/state"; return; }
        local saved_render="$functions[_zacrs_render]" saved_retry="$functions[_zacrs_maybe_retry_daemon]" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_render() { print -r -- "$_zacrs_filesystem_requested" > "$HOME/redrawn"; }
        _zacrs_maybe_retry_daemon() { return 1; }
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        _zacrs_daemon_available=0
        functions[_test_actual_redraw]="$_test_production_redraw"
        _test_actual_redraw
        functions[_zacrs_render]="$saved_render"; functions[_zacrs_maybe_retry_daemon]="$saved_retry"; functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ "$(<"$HOME/redrawn")" == 0 && "$BUFFER" == "$original" ]] || { print -r -- stale-redraw > "$_test_root/state"; return; }
    elif (( _test_index == 18 )); then
        local HOME="$HOME" old_home="$HOME" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        BUFFER='probe ~/files/spa'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str=''
        _zacrs_collect_candidates
        _zacrs_current_context_key; local old_key="$REPLY"
        _zacrs_render "$prefix" "$prefix_len" "$candidates_str" 1 '' "$old_key"
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- home-seed-failed > "$_test_root/state"; return; }
        _zacrs_prev_lbuffer="$LBUFFER"; _zacrs_prev_buffer="$BUFFER"; _zacrs_prev_cursor=$CURSOR; _zacrs_prev_home="$HOME"
        HOME="$_test_root/new-home"
        _zacrs_current_context_key
        [[ "$REPLY" != "$old_key" ]] || { print -r -- stale-home-key > "$_test_root/state"; return; }
        functions[_test_actual_redraw]="$_test_production_redraw"
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 && "$_zacrs_popup_snapshot_home" == "$HOME" && "$_zacrs_prev_home" == "$HOME" ]] || { print -r -- stale-home-redraw > "$_test_root/state"; return; }
        _zacrs_complete_popup
        local inserted="${BUFFER#probe }" expected="$HOME/files/space new-home"
        inserted="${inserted% }"
        [[ "${(Q)inserted}" == "$expected" ]] || { print -r -- wrong-home-selection > "$_test_root/state"; return; }
        # Check Tab's snapshot guard independently of the cache key: seed an
        # old-home snapshot, then observe whether it requests fresh candidates.
        HOME="$old_home"; BUFFER='probe ~/files/spa'; CURSOR=${#BUFFER}
        _zacrs_popup_visible=1; _zacrs_record_popup_snapshot 11
        HOME="$_test_root/new-home"
        local saved_invoke="$functions[_zacrs_invoke_daemon]"
        functions[_test_home_invoke]="$saved_invoke"
        _zacrs_invoke_daemon() { _test_home_reuse=$6; _test_home_invoke "$@"; }
        _zacrs_complete_popup
        functions[_zacrs_invoke_daemon]="$saved_invoke"
        if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
            [[ $_test_home_reuse == 0 ]] || { print -r -- stale-home-tab > "$_test_root/state"; return; }
        fi
        inserted="${BUFFER#probe }"; inserted="${inserted% }"
        [[ "${(Q)inserted}" == "$expected" ]] || { print -r -- wrong-home-tab > "$_test_root/state"; return; }
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
    elif (( _test_index == 19 )); then
        # Keep legacy debug helper return statuses from masking the optional-query regression.
        local ZACRS_DEBUG=1 ZACRS_LOG="$_test_root/context-debug.log"
        local saved_provider="$functions[_main_complete]"
        _main_complete() { print -r -- reached > "$_test_root/native-reached"; return 0; }
        BUFFER=pro; CURSOR=3
        () { setopt localoptions errreturn; zle _zacrs_compsys; }
        functions[_main_complete]="$saved_provider"
        [[ -f "$_test_root/native-reached" ]] || { print -r -- declined-native-provider > "$_test_root/state"; return; }
    elif (( _test_index == 20 )); then
        local HOME="$HOME" saved_provider="$functions[_main_complete]" prefix='' prefix_len=0 candidates_str=''
        BUFFER='probe ~/files/spa'; CURSOR=${#BUFFER}
        _main_complete() { HOME="$_test_root/new-home"; return 0; }
        _zacrs_collect_candidates
        functions[_main_complete]="$saved_provider"
        [[ $_zacrs_filesystem_requested == 0 && "$HOME" == "$_test_root/new-home" ]] || { print -r -- mutable-home-query > "$_test_root/state"; return; }
        # The next native observation can safely use the changed home.
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 1 && "$_zacrs_filesystem_lookup" == "$HOME/files/spa" ]] || { print -r -- mutable-home-retry > "$_test_root/state"; return; }
    elif (( _test_index == 21 && ZACRS_TEST_DAEMON )); then
        BUFFER=$'probe space tail\\n\\c\v\xc2\xa0'; CURSOR=11
        _zacrs_current_context_key; local raw_key="$REPLY" code=0
        _zacrs_filesystem_requested=0
        _zacrs_daemon_send_render 2 1 space $'space-result\tcommand\tcommand' '' "$raw_key" 0 1 || code=$?
        [[ $code == 0 ]] || { print -r -- raw-render-header > "$_test_root/state"; return; }
        local fd=$_zacrs_send_render_fd
        _zacrs_daemon_draw_atomic $fd $_zacrs_parsed_tty_len 0 0 0 0
        exec {fd}<&-
        _zacrs_invoke_daemon space 5 '' 2 1 0 "$raw_key" 1 0 || code=$?
        [[ $code == 0 && "$BUFFER" == $'probe space-result  tail\\n\\c\v\xc2\xa0' ]] || { print -r -- raw-complete-header > "$_test_root/state"; return; }
    elif (( _test_index == 22 )); then
        local HOME="$HOME" old_home="$HOME" saved_provider="$functions[_main_complete]" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        _test_retry_provider_calls=0
        _main_complete() { (( ++_test_retry_provider_calls )); HOME="$_test_root/new-home"; return 0; }
        BUFFER='probe ~/files/spa'; CURSOR=${#BUFFER}
        _zacrs_prev_lbuffer=''; _zacrs_prev_buffer=''; _zacrs_prev_home="$HOME"
        _zacrs_chain_retry=1
        _zacrs_invalidate_candidate_cache
        functions[_test_actual_redraw]="$_test_production_redraw"
        _test_actual_redraw
        [[ $_test_retry_provider_calls == 2 && $_zacrs_filesystem_requested == 1 && "$_zacrs_popup_snapshot_home" == "$HOME" && "$_zacrs_prev_home" == "$HOME" ]] || { print -r -- retry-home-query > "$_test_root/state"; return; }
        functions[_main_complete]="$saved_provider"
        # Restoring HOME must not find the new home's snapshot under the old key.
        HOME="$old_home"
        _test_actual_redraw
        [[ "$_zacrs_popup_snapshot_home" == "$HOME" ]] || { print -r -- retry-home-restored-redraw > "$_test_root/state"; return; }
        _zacrs_complete_popup
        local inserted="${BUFFER#probe }" expected="$HOME/files/space name"
        inserted="${inserted% }"
        [[ "${(Q)inserted}" == "$expected" ]] || { print -r -- retry-home-poisoned-cache > "$_test_root/state"; return; }
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
    fi
    if (( _test_index >= 15 )); then
        (( ${+literal_path_known} + ${+literal_path} + ${+literal_suffix} == 0 )) || { print -r -- leaked-literal-state > "$_test_root/state"; return; }
        print -r -- "ok:$_test_index" > "$_test_root/state"
        return
    fi
    CURSOR=${#BUFFER}
    (( _test_index == 11 )) && CURSOR=9
    local original="$BUFFER" original_cursor=$CURSOR
    local prefix='' prefix_len=0 candidates_str=''
    _zacrs_collect_candidates
    if (( valid )); then
        [[ $_zacrs_filesystem_requested == 1 ]] || { print -r -- "missing-context:$_test_index" > "$_test_root/state"; return; }
        # Empty argument tests only context availability; other cases select one file.
        if (( select )); then
            if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
                _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 "files-$_test_index" "$accept_single" 1
            else
                _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 "$accept_single"
            fi
            local inserted="${BUFFER#probe }"
            # Separate the configured unquoted suffix before native dequoting.
            [[ "$expected" == *' ' ]] && inserted="${inserted% }"
            local decoded="${(Q)inserted}"
            [[ "$expected" == *' ' ]] && decoded+=' '
            [[ "$decoded" == "$expected" ]] || { print -r -- "wrong:$_test_index:${(qqq)BUFFER}:${(qqq)decoded}" > "$_test_root/state"; return; }
            [[ $CURSOR == ${#BUFFER} ]] || { print -r -- "cursor:$_test_index" > "$_test_root/state"; return; }
        fi
    else
        [[ $_zacrs_filesystem_requested == 0 && "$BUFFER" == "$original" && $CURSOR == $original_cursor ]] || { print -r -- "unsafe:$_test_index" > "$_test_root/state"; return; }
    fi
    (( ${+literal_path_known} + ${+literal_path} + ${+literal_suffix} == 0 )) || { print -r -- leaked-literal-state > "$_test_root/state"; return; }
    print -r -- "ok:$_test_index" > "$_test_root/state"
}
zle -N _test_run
bindkey '^T' _test_run
PROMPT='READY> '
RPROMPT=''
print -r -- ready > "$HOME/ready"
RC
cleanup() {
    zpty -d files 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
zpty -b files zsh -di
typeset output='' chunk=''
for attempt in {1..300}; do
    zpty -r -t files chunk 2>/dev/null && output+="$chunk"
    [[ "$output" == *'READY> '* ]] && break
    sleep .01
done
[[ "$output" == *'READY> '* ]] || { print -u2 -r -- "not ready: ${(qqq)output}"; exit 1; }
for case_index in {1..22}; do
    : > "$_test_root/state"
    zpty -w -n files $'\x14'
    for attempt in {1..300}; do
        zpty -r -t files chunk 2>/dev/null && output+="$chunk"
        [[ -s "$_test_root/state" ]] && break
        sleep .01
    done
    [[ "$(<"$_test_root/state")" == "ok:$case_index" ]] || { print -u2 -r -- "case $case_index: $(<"$_test_root/state") pty: ${(qqq)output}"; exit 1; }
done
print -r -- "ok: real filesystem selection, quoting and unknown-context preservation daemon=$ZACRS_TEST_DAEMON"
