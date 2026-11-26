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
mkdir -p "$HOME/files/directory" "$HOME/new-cwd"
: > "$HOME/files/NAME"
: > "$HOME/files/Report"
: > "$HOME/files/Repair"
mkdir -p "$HOME/files/Folder" "$HOME/files/dots"
: > "$HOME/files/Folder/Name"
: > "$HOME/files/dots/.hidden"
: > "$HOME/files/a~bee"
: > "$HOME/files/abcdef"
mkdir -p "$HOME/files/Repeat" "$HOME/cwd-original" "$HOME/cwd-retargeted"
: > "$HOME/files/Repeat/fuzzy.rs"
: > "$HOME/files/Repeat/future.rs"
: > "$HOME/cwd-original/match-original"
: > "$HOME/cwd-retargeted/match-wrong"
ln -s "$HOME/cwd-original" "$HOME/logical-cwd"
printf '%s\n' '#!/bin/sh' 'printf "%s\n" "$1" > "$HOME/argv"' > "$HOME/files/local"
chmod +x "$HOME/files/local"
printf '%s\n' '#!/bin/sh' 'printf "%s\n" "$1" > "$HOME/argv"' > "$HOME/files/tool.py"
chmod +x "$HOME/files/tool.py"
: > "$HOME/files/far"
: > "$HOME/files/foobar"
: > "$HOME/new-cwd/space new-cwd"
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
        [[ $_zacrs_filesystem_requested == 1 && "$_zacrs_filesystem_base" == "$HOME" && "$_zacrs_filesystem_lookup" == files/spa ]] || { print -r -- mutable-home-retry > "$_test_root/state"; return; }
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
    elif (( _test_index == 23 )); then
        local prefix='' prefix_len=0 candidates_str=''
        BUFFER='for fo'; CURSOR=${#BUFFER}
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 0 && "$BUFFER" == 'for fo' ]] || { print -r -- loop-variable-query > "$_test_root/state"; return; }
        BUFFER='~/files/dire'; CURSOR=${#BUFFER}
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 1 && "$_zacrs_filesystem_base" == "$HOME" && "$_zacrs_filesystem_lookup" == files/dire ]] || { print -r -- home-command-path > "$_test_root/state"; return; }
        BUFFER='~/'; CURSOR=2
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 1 && "$_zacrs_filesystem_base" == "$HOME" && -z "$_zacrs_filesystem_lookup" ]] || { print -r -- bare-home-command-path > "$_test_root/state"; return; }
    elif (( _test_index == 24 )); then
        local saved_provider="$functions[_main_complete]"
        _test_glob_provider=0
        _main_complete() { _test_glob_provider=1; return 0; }
        BUFFER='probe space no-match-*'; CURSOR=11
        () { setopt localoptions globsubst; zle _zacrs_compsys; }
        functions[_main_complete]="$saved_provider"
        (( _test_glob_provider )) || { print -r -- glob-blocked-native > "$_test_root/state"; return; }
    elif (( _test_index == 25 )); then
        local saved_bin="$ZACRS_BIN" saved_cursor="$functions[_zacrs_get_cursor_pos]" saved_retry="$functions[_zacrs_maybe_retry_daemon]"
        local _zacrs_daemon_available=0
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        _zacrs_maybe_retry_daemon() { return 0; }
        _test_native_render_bin() { print -r -- called > "$_test_root/native-render"; }
        ZACRS_BIN=_test_native_render_bin
        _zacrs_filesystem_requested=0
        BUFFER='probe fil'; CURSOR=${#BUFFER}
        () { setopt localoptions errreturn; _zacrs_render fil 3 $'file\tfile\tfile'; }
        ZACRS_BIN="$saved_bin"
        functions[_zacrs_get_cursor_pos]="$saved_cursor"; functions[_zacrs_maybe_retry_daemon]="$saved_retry"
        [[ -f "$_test_root/native-render" ]] || { print -r -- empty-filesystem-args-blocked-render > "$_test_root/state"; return; }
    elif (( _test_index == 26 )); then
        alias -g NAME=expanded 'directory/=expanded'
        probe() { print -r -- "$@" > "$_test_root/argv"; }
        local prefix='' prefix_len=0 candidates_str=''
        local typed expected_arg
        for typed expected_arg in NA NAME dire directory/ NA ''; do
            [[ -z "$expected_arg" ]] && alias -g "'NAME'=expanded"
            BUFFER="probe $typed"; CURSOR=${#BUFFER}
            _zacrs_collect_candidates
            if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
                _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 alias-path 1 1
            else
                _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 1
            fi
            if [[ -n "$expected_arg" ]]; then
                eval "$BUFFER"
                [[ "$(<"$_test_root/argv")" == "$expected_arg" ]] || { print -r -- alias-expanded-literal > "$_test_root/state"; return; }
            else
                [[ "$BUFFER" == 'probe NA' ]] || { print -r -- alias-quoted-literal > "$_test_root/state"; return; }
            fi
        done
        unalias NAME 'directory/' "'NAME'"
        unfunction probe
    elif (( _test_index == 27 )); then
        local old_cwd="$PWD" saved_provider="$functions[_main_complete]" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        _test_retry_provider_calls=0
        _main_complete() { (( ++_test_retry_provider_calls )); cd "$_test_root/new-cwd"; return 0; }
        BUFFER='probe spa'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str=''
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 0 && "$PWD" == "$_test_root/new-cwd" ]] || { print -r -- mutable-cwd-query > "$_test_root/state"; return; }
        cd "$old_cwd"
        _test_retry_provider_calls=0
        _zacrs_prev_lbuffer=''; _zacrs_prev_buffer=''
        _zacrs_chain_retry=1
        _zacrs_invalidate_candidate_cache
        functions[_test_actual_redraw]="$_test_production_redraw"
        _test_actual_redraw
        [[ $_test_retry_provider_calls == 2 && $_zacrs_filesystem_requested == 1 && "$_zacrs_filesystem_cwd" == "$PWD" && "$_zacrs_popup_snapshot_cwd" == "$PWD" && "$_zacrs_prev_cwd" == "$PWD" ]] || { print -r -- retry-cwd-query > "$_test_root/state"; return; }
        functions[_main_complete]="$saved_provider"
        cd "$old_cwd"
        _test_actual_redraw
        _zacrs_complete_popup
        local inserted="${BUFFER#probe }"
        inserted="${inserted% }"
        [[ "${(Q)inserted}" == 'space name' ]] || { print -r -- retry-cwd-poisoned-cache > "$_test_root/state"; return; }
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
    elif (( _test_index == 28 && ZACRS_TEST_DAEMON )); then
        local saved_cursor="$functions[_zacrs_get_cursor_pos]" saved_daemon="$functions[_zacrs_invoke_daemon]" saved_cli="$functions[_zacrs_invoke]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        _zacrs_invalidate_candidate_cache
        functions[_test_actual_redraw]="$_test_production_redraw"
        BUFFER='probe f'; CURSOR=${#BUFFER}
        _test_actual_redraw
        BUFFER='probe fo'; CURSOR=${#BUFFER}
        _test_actual_redraw
        BUFFER='probe f'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 && "$_zacrs_filesystem_lookup" == fo ]] || { print -r -- cache-handoff-not-seeded > "$_test_root/state"; return; }
        _zacrs_invoke_daemon() { _test_daemon_reuse=$6; return 1; }
        _zacrs_invoke() { _test_cli_query="$_zacrs_filesystem_requested:$_zacrs_filesystem_lookup:$1"; return 0; }
        _zacrs_complete_popup
        functions[_zacrs_invoke_daemon]="$saved_daemon"; functions[_zacrs_invoke]="$saved_cli"
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ $_test_daemon_reuse == 1 && "$_test_cli_query" == 1:f:f ]] || { print -r -- stale-cli-query > "$_test_root/state"; return; }
    elif (( _test_index == 29 )); then
        local saved_bin="$ZACRS_BIN" before='probe space' code=0
        _test_no_edit_bin() { print -r -- 'DONE 5 '; print -r -- 'APPLY chain=0 execute=0 restore_hex='; }
        ZACRS_BIN=_test_no_edit_bin
        BUFFER="$before"; CURSOR=${#BUFFER}
        _zacrs_filesystem_requested=0
        _zacrs_invoke space 5 '' 2 1 1 || code=$?
        ZACRS_BIN="$saved_bin"
        [[ $code == 5 && "$BUFFER" == "$before" ]] || { print -r -- no-edit-status-lost > "$_test_root/state"; return; }
    elif (( _test_index == 30 )); then
        setopt localoptions
        alias './local=print expanded' 'probe_alias=probe ' 'NAME=expanded'
        probe() { print -r -- "$@" > "$_test_root/argv"; }
        local prefix='' prefix_len=0 candidates_str='' typed arg
        for typed arg in ./loc selected 'probe_alias NA' NAME; do
            [[ "$typed" == 'probe_alias NA' ]] && setopt complete_aliases
            BUFFER="$typed"; CURSOR=${#BUFFER}
            _zacrs_collect_candidates
            [[ $_zacrs_filesystem_requested == 1 ]] || { print -r -- "alias-no-query:$typed" > "$_test_root/state"; return; }
            if (( ZACRS_TEST_DAEMON )); then
                _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 ordinary-alias-path 1 1
            else
                _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 1
            fi
            if [[ "$typed" == ./loc ]]; then
                eval "$BUFFER selected"
            else
                eval "$BUFFER"
            fi
            [[ "$(<"$_test_root/argv")" == "$arg" ]] || { print -r -- "ordinary-alias-expanded:$BUFFER" > "$_test_root/state"; return; }
        done
        unalias ./local probe_alias NAME
        unfunction probe
    elif (( _test_index == 31 )); then
        alias -g NAME=expanded
        BUFFER='probe \NA'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str='' before="$BUFFER" before_cursor=$CURSOR
        _zacrs_collect_candidates
        unalias NAME
        [[ $_zacrs_filesystem_requested == 0 && "$BUFFER" == "$before" && $CURSOR == $before_cursor ]] || { print -r -- escaped-word-query > "$_test_root/state"; return; }
    elif (( _test_index == 32 )); then
        setopt localoptions no_case_glob
        local saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        functions[_test_actual_redraw]="$_test_production_redraw"
        _zacrs_invalidate_candidate_cache
        BUFFER='probe rEPo'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 && "$_zacrs_popup_snapshot_file_options" == off:* ]] || { print -r -- insensitive-preview-missing > "$_test_root/state"; return; }
        _zacrs_current_context_key; local insensitive_key="$REPLY"
        setopt case_glob
        _zacrs_current_context_key
        [[ "$REPLY" != "$insensitive_key" ]] || { print -r -- reused-case-key > "$_test_root/state"; return; }
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 0 && "$BUFFER" == 'probe rEPo' ]] || { print -r -- insensitive-preview-reused > "$_test_root/state"; return; }
        unsetopt case_glob
        _test_actual_redraw
        _zacrs_complete_popup
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ "$BUFFER" == 'probe Report ' && $CURSOR == ${#BUFFER} ]] || { print -r -- "case-selection:$BUFFER" > "$_test_root/state"; return; }
    elif (( _test_index == 33 )); then
        local saved_provider="$functions[_main_complete]"
        local saved_context="$functions[_zacrs_filesystem_context]"
        local prefix='' prefix_len=0 candidates_str=''
        _test_context_calls=0
        functions[_test_context]="$saved_context"
        _zacrs_filesystem_context() { (( ++_test_context_calls )); _test_context "$@"; }
        _main_complete() { compset -p 1; compadd -U -- native-row; return 0; }
        BUFFER='probe space'; CURSOR=${#BUFFER}
        _zacrs_collect_candidates
        [[ "$candidates_str" == *native-row* && $_test_context_calls == 0 ]] || { print -r -- native-paid-for-file-context > "$_test_root/state"; return; }
        _main_complete() { compset -p 1; return 0; }
        _zacrs_collect_candidates
        functions[_main_complete]="$saved_provider"
        functions[_zacrs_filesystem_context]="$saved_context"
        [[ $_test_context_calls == 1 && $_zacrs_filesystem_requested == 1 && "$prefix" == space && $prefix_len == 5 && "$BUFFER" == 'probe space' && $CURSOR == 11 ]] || { print -r -- native-prefix-observation-mutated > "$_test_root/state"; return; }
    elif (( _test_index == 34 )); then
        alias -s 'py=print suffix-alias-ran'
        BUFFER='./to'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str=''
        _zacrs_collect_candidates
        if (( ZACRS_TEST_DAEMON )); then
            _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 suffix-alias-path 1 1
        else
            _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 1
        fi
        eval "$BUFFER suffix-protected"
        unalias -s py
        [[ "$(<"$_test_root/argv")" == suffix-protected ]] || { print -r -- "suffix-alias-expanded:$BUFFER" > "$_test_root/state"; return; }
    elif (( _test_index == 35 )); then
        setopt localoptions no_case_glob
        BUFFER='probe rEP'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str=''
        _zacrs_collect_candidates
        _zacrs_cursor_stale=$'o\t\r'
        if (( ZACRS_TEST_DAEMON )); then
            _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 narrow-folded-path 1 1
        else
            _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 1
        fi
        [[ "$BUFFER" == 'probe Report ' && $CURSOR == ${#BUFFER} ]] || { print -r -- "case-narrowing:$BUFFER" > "$_test_root/state"; return; }
    elif (( _test_index == 36 )); then
        setopt localoptions no_case_glob no_case_paths
        local saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        local typed
        for typed in 'folder/na' '~/files/folder/na'; do
            BUFFER="probe $typed"; CURSOR=${#BUFFER}
            _test_actual_redraw
            [[ $_zacrs_popup_visible == 1 ]] || { print -r -- "missing-folded-parent:$typed" > "$_test_root/state"; return; }
            if [[ "$typed" == folder/* && ! -d folder ]]; then
                # Same line and cursor must not reuse the folded-parent snapshot.
                setopt case_paths
                _test_actual_redraw
                [[ $_zacrs_popup_visible == 0 ]] || { print -r -- stale-parent-preview > "$_test_root/state"; return; }
                unsetopt case_paths
                _test_actual_redraw
            fi
            _zacrs_complete_popup
            if [[ "$typed" == folder/* ]]; then
                [[ "$BUFFER" == 'probe Folder/Name ' ]] || { print -r -- "folded-parent-selection:$BUFFER" > "$_test_root/state"; return; }
            else
                [[ "$BUFFER" == "probe $HOME/files/Folder/Name " ]] || { print -r -- "folded-home-selection:$BUFFER" > "$_test_root/state"; return; }
            fi
        done
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
    elif (( _test_index == 37 )); then
        setopt localoptions glob_dots
        local saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        BUFFER='probe dots/'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- hidden-preview-missing > "$_test_root/state"; return; }
        unsetopt glob_dots
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 0 ]] || { print -r -- stale-hidden-preview > "$_test_root/state"; return; }
        setopt glob_dots
        _test_actual_redraw
        _zacrs_complete_popup
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ "$BUFFER" == 'probe dots/.hidden ' ]] || { print -r -- "hidden-selection:$BUFFER" > "$_test_root/state"; return; }
    elif (( _test_index == 38 && ZACRS_TEST_DAEMON )); then
        local saved_ping="$functions[_zacrs_ping_daemon]" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        local saved_send="$functions[_zacrs_daemon_send_render]" saved_complete="$functions[_zacrs_invoke_daemon]"
        local _zacrs_enter_expansion=0 _zacrs_space_enabled=0 _zacrs_edit_transport_state=0
        _zacrs_ping_daemon() { REPLY='OK enter_expansion=1 edit_context=2 space_expansion=0'; }
        _zacrs_initialize_daemon
        functions[_zacrs_ping_daemon]="$saved_ping"
        [[ $_zacrs_daemon_filesystem == 0 && $_zacrs_edit_transport_state == 1 ]] || { print -r -- legacy-capabilities > "$_test_root/state"; return; }
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        functions[_test_actual_send]="$saved_send"
        _test_legacy_file_request=0
        _zacrs_daemon_send_render() {
            if (( ${8:-0} && _zacrs_filesystem_requested )); then
                _test_legacy_file_request=1
                _zacrs_mark_daemon_unavailable
                return 2
            fi
            _test_actual_send "$@"
        }
        _zacrs_invoke_daemon() { _test_legacy_file_request=1; _zacrs_mark_daemon_unavailable; return 1; }
        BUFFER='probe space'; CURSOR=${#BUFFER}
        _zacrs_invalidate_candidate_cache
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 && $_zacrs_popup_snapshot_daemon == 0 ]] || { print -r -- legacy-subprocess-preview > "$_test_root/state"; return; }
        _zacrs_complete_popup
        [[ "$BUFFER" == 'probe space\ name ' && $_test_legacy_file_request == 0 && $_zacrs_daemon_available == 1 ]] || { print -r -- legacy-subprocess-tab > "$_test_root/state"; return; }
        _zacrs_abbreviation_request reset || { print -r -- legacy-abbreviation-transport > "$_test_root/state"; return; }
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        functions[_zacrs_daemon_send_render]="$saved_send"
        functions[_zacrs_invoke_daemon]="$saved_complete"
        _zacrs_initialize_daemon
    elif (( _test_index == 39 )); then
        setopt localoptions no_extended_glob
        local saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        BUFFER='probe a~b'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- syntax-preview-missing > "$_test_root/state"; return; }
        _zacrs_current_context_key; local original_key="$REPLY"
        setopt extended_glob
        _zacrs_current_context_key
        [[ "$REPLY" != "$original_key" ]] || { print -r -- syntax-cache-key-reused > "$_test_root/state"; return; }
        _zacrs_complete_popup
        [[ "$BUFFER" == 'probe a~b' && $_zacrs_filesystem_requested == 0 ]] || { print -r -- syntax-tab-reused > "$_test_root/state"; return; }
        # Also exercise cache-first redraw when the buffer is identical.
        unsetopt extended_glob
        _zacrs_prev_buffer=''
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- syntax-preview-not-restored > "$_test_root/state"; return; }
        setopt extended_glob
        _test_actual_redraw
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ $_zacrs_popup_visible == 0 ]] || { print -r -- syntax-redraw-reused > "$_test_root/state"; return; }
    elif (( _test_index == 40 )); then
        setopt localoptions no_chase_links
        local old_cwd="$PWD" saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        cd -- "$_test_root/logical-cwd"
        [[ "$PWD" == "$_test_root/logical-cwd" ]] || { print -r -- cwd-fixture-not-logical > "$_test_root/state"; return; }
        BUFFER='probe match'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- logical-cwd-preview-missing > "$_test_root/state"; return; }
        ln -sfn "$_test_root/cwd-retargeted" "$_test_root/logical-cwd"
        [[ -f match-original && -f "$PWD/match-wrong" ]] || { print -r -- cwd-fixture-not-retargeted > "$_test_root/state"; return; }
        _zacrs_current_context_key
        [[ -z "$REPLY" ]] || { print -r -- retargeted-cwd-key > "$_test_root/state"; return; }
        _zacrs_complete_popup
        [[ "$BUFFER" == 'probe match' && $_zacrs_filesystem_requested == 0 ]] || { print -r -- retargeted-cwd-selection > "$_test_root/state"; return; }
        cd -- "$old_cwd"
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
    elif (( _test_index == 41 )); then
        setopt localoptions no_case_glob no_case_paths
        BUFFER='probe rEpeat//./fu'; CURSOR=${#BUFFER}
        local prefix='' prefix_len=0 candidates_str=''
        _zacrs_collect_candidates
        _zacrs_cursor_stale=$'z\t\r'
        if (( ZACRS_TEST_DAEMON )); then
            _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" 2 1 0 narrow-preserved-spelling 1 1
        else
            _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" 2 1 1
        fi
        [[ "$BUFFER" == 'probe Repeat/fuzzy.rs ' && $CURSOR == ${#BUFFER} ]] || { print -r -- "corrected-path-narrowing:$BUFFER" > "$_test_root/state"; return; }
    elif (( _test_index == 42 )); then
        local saved_bin="$ZACRS_BIN" code=0
        _test_truncated_bin() {
            print -r -- 'DONE 0 space name'
            print -rn -- 'APPLY chain=1 execute=1'
        }
        ZACRS_BIN=_test_truncated_bin
        BUFFER='probe space'; CURSOR=${#BUFFER}
        _zacrs_invoke space 5 '' 2 1 1 || code=$?
        [[ $code == 5 && "$BUFFER" == 'probe space' && $CURSOR == 11 ]] || { print -r -- truncated-subprocess-applied > "$_test_root/state"; return; }
        code=0
        _zacrs_invoke_daemon space 5 '' 2 1 0 truncated-reply 1 1 || code=$?
        ZACRS_BIN="$saved_bin"
        [[ $code == 5 && "$BUFFER" == 'probe space' && $CURSOR == 11 ]] || { print -r -- truncated-daemon-applied > "$_test_root/state"; return; }
    elif (( _test_index == 43 )); then
        setopt localoptions no_interactive_comments
        local saved_cursor="$functions[_zacrs_get_cursor_pos]"
        _zacrs_get_cursor_pos() { cursor_row=2; cursor_col=1; }
        BUFFER='probe #com abc'; CURSOR=${#BUFFER}
        _test_actual_redraw
        [[ $_zacrs_popup_visible == 1 ]] || { print -r -- literal-comment-preview-missing > "$_test_root/state"; return; }
        _zacrs_current_context_key; local original_key="$REPLY"
        setopt interactive_comments
        _zacrs_current_context_key
        [[ "$REPLY" != "$original_key" ]] || { print -r -- comment-cache-key-reused > "$_test_root/state"; return; }
        _zacrs_complete_popup
        functions[_zacrs_get_cursor_pos]="$saved_cursor"
        [[ "$BUFFER" == 'probe #com abc' && $_zacrs_filesystem_requested == 0 ]] || { print -r -- comment-tab-reused > "$_test_root/state"; return; }
    elif (( _test_index == 44 )); then
        local prefix='' prefix_len=0 candidates_str='' line
        for line in 'exec loc' 'command loc' 'command -p loc' 'noglob loc' 'builtin loc' 'nocorrect loc' '- loc'; do
            BUFFER="$line"; CURSOR=${#BUFFER}
            _zacrs_collect_candidates
            [[ $_zacrs_filesystem_requested == 0 && "$BUFFER" == "$line" && $CURSOR == ${#line} ]] || {
                print -r -- "precommand-file-query:$line" > "$_test_root/state"; return
            }
        done
        BUFFER='probe loc'; CURSOR=${#BUFFER}
        _zacrs_collect_candidates
        [[ $_zacrs_filesystem_requested == 1 ]] || { print -r -- expected-file-argument > "$_test_root/state"; return; }
    elif (( _test_index == 45 )); then
        # Restore the fields of a primary preview to exercise the reuse gate
        # independently of normal prompt-transition cleanup.
        [[ "$CONTEXT" != start || -n "$PREBUFFER" ]] || { print -r -- expected-continuation-editor > "$_test_root/state"; return; }
        BUFFER='probe f'; CURSOR=${#BUFFER}
        _zacrs_popup_visible=1
        _zacrs_popup_snapshot_buffer="$BUFFER"; _zacrs_popup_snapshot_cursor=$CURSOR
        _zacrs_popup_snapshot_cwd="$PWD"; _zacrs_popup_snapshot_home="$HOME"
        _zacrs_popup_snapshot_daemon=1
        local _zacrs_file_option_key
        _zacrs_file_options
        _zacrs_popup_snapshot_file_options="$_zacrs_file_option_key"
        _zacrs_popup_snapshot_columns=$COLUMNS; _zacrs_popup_snapshot_lines=$LINES
        _zacrs_current_context_key
        [[ -z "$REPLY" ]] || { print -r -- continuation-cache-key > "$_test_root/state"; return; }
        local saved_invoke="$functions[_zacrs_invoke_daemon]"
        _zacrs_daemon_available=1
        _zacrs_invoke_daemon() { _test_cont_result="$_zacrs_filesystem_requested:$6:$7"; return 0; }
        _zacrs_complete_popup
        functions[_zacrs_invoke_daemon]="$saved_invoke"
        [[ "$_test_cont_result" == '0:0:' && "$BUFFER" == 'probe f' ]] || { print -r -- continuation-reused-primary > "$_test_root/state"; return; }
        # An identical line must also miss the redraw early return on transition.
        _zacrs_prev_lbuffer="$LBUFFER"; _zacrs_prev_buffer="$BUFFER"; _zacrs_prev_cursor=$CURSOR
        _zacrs_prev_cwd="$PWD"; _zacrs_prev_home="$HOME"; _zacrs_prev_primary_editor=1
        local saved_send="$functions[_zacrs_daemon_send_render]" saved_render="$functions[_zacrs_render]"
        _test_cont_result=unchanged
        _zacrs_daemon_send_render() { _test_cont_result=stale-cache; return 3; }
        _zacrs_render() { [[ "$_test_cont_result" == unchanged ]] && _test_cont_result="$_zacrs_filesystem_requested:$6"; }
        _test_actual_redraw
        functions[_zacrs_daemon_send_render]="$saved_send"; functions[_zacrs_render]="$saved_render"
        [[ "$_test_cont_result" == '0:' && "$BUFFER" == 'probe f' ]] || { print -r -- continuation-redraw-reused-primary > "$_test_root/state"; return; }
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
_test_start_continuation() { BUFFER='probe "'; CURSOR=${#BUFFER}; zle .accept-line; }
zle -N _test_start_continuation
bindkey '^G' _test_start_continuation
PS2='CONT> '
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
for case_index in {1..45}; do
    if (( case_index == 45 )); then
        zpty -w -n files $'\x07'
        for attempt in {1..300}; do
            zpty -r -t files chunk 2>/dev/null && output+="$chunk"
            [[ "$output" == *'CONT> '* ]] && break
            sleep .01
        done
        [[ "$output" == *'CONT> '* ]] || { print -u2 -r -- 'continuation prompt missing'; exit 1; }
    fi
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
