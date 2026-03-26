#!/usr/bin/env zsh
# zsh-autocomplete-rs: Rust-powered autocomplete with popup UI
# Usage: source this file in your .zshrc

# === Initialization ===

ZACRS_BIN="${ZACRS_BIN:-zsh-autocomplete-rs}"

# Source helpers
_zacrs_dir="${0:A:h}"
source "${_zacrs_dir}/_zacrs_util.zsh"
source "${_zacrs_dir}/_zacrs_gather.zsh"
source "${_zacrs_dir}/_zacrs_compsys.zsh"

# Internal state
typeset -g _zacrs_prev_lbuffer=""
typeset -g _zacrs_suppressed=0
typeset -g _zacrs_popup_visible=0
typeset -g _zacrs_popup_row=0
typeset -g _zacrs_popup_height=0
typeset -g _zacrs_popup_cursor_row=0
typeset -g _zacrs_cached_candidates=""
typeset -g _zacrs_cached_lbase=""
typeset -gi _zacrs_chain_retry=0
typeset -g _zacrs_daemon_available=0
typeset -g _zacrs_daemon_started=0
typeset -g _zacrs_daemon_next_retry=0
typeset -g _zacrs_socket_path=""
typeset -g _zacrs_popup_snapshot_lbuffer=""
typeset -g _zacrs_popup_snapshot_prefix=""
typeset -gi _zacrs_popup_snapshot_prefix_len=0
typeset -g _zacrs_popup_snapshot_candidates=""
typeset -gi _zacrs_popup_snapshot_cursor_row=0
typeset -gi _zacrs_popup_snapshot_cursor_col=0
typeset -g _zacrs_popup_snapshot_reuse_token=""
typeset -gi _zacrs_popup_snapshot_from_gather=0
typeset -gi _zacrs_popup_snapshot_columns=0
typeset -gi _zacrs_popup_snapshot_lines=0
typeset -gi _zacrs_cached_from_gather=0

# Render header parse results (set by _zacrs_parse_render_header)
typeset -g  _zacrs_parsed_reuse_token=""
typeset -gi _zacrs_parsed_tty_len=0

# Cycle mode state
typeset -gi _zacrs_cycle_active=0
typeset -gi _zacrs_cycle_index=0
typeset -g  _zacrs_cycle_original_lbuffer=""
typeset -g  _zacrs_cycle_prefix=""
typeset -gi _zacrs_cycle_prefix_len=0
typeset -g  _zacrs_cycle_candidates=""
typeset -g  _zacrs_cycle_prev_keymap=""
typeset -gi _zacrs_cycle_filtered_count=0
typeset -gi _zacrs_cycle_selected_original_idx=-1
typeset -gi _zacrs_cycle_cursor_row=0
typeset -gi _zacrs_cycle_cursor_col=0

_zacrs_reset_popup_snapshot() {
    _zacrs_popup_snapshot_lbuffer=""
    _zacrs_popup_snapshot_prefix=""
    _zacrs_popup_snapshot_prefix_len=0
    _zacrs_popup_snapshot_candidates=""
    _zacrs_popup_snapshot_cursor_row=0
    _zacrs_popup_snapshot_cursor_col=0
    _zacrs_popup_snapshot_reuse_token=""
    _zacrs_popup_snapshot_from_gather=0
    _zacrs_popup_snapshot_columns=0
    _zacrs_popup_snapshot_lines=0
}

_zacrs_reset_cache() {
    _zacrs_cached_candidates=""
    _zacrs_cached_from_gather=0
    _zacrs_cached_lbase=""
}

_zacrs_record_popup_snapshot() {
    local prefix="$1" prefix_len="$2" candidates_str="$3" cursor_col="$4" reuse_token="$5" from_gather="${6:-0}"
    _zacrs_popup_snapshot_lbuffer="$LBUFFER"
    _zacrs_popup_snapshot_prefix="$prefix"
    _zacrs_popup_snapshot_prefix_len=$prefix_len
    _zacrs_popup_snapshot_candidates="$candidates_str"
    _zacrs_popup_snapshot_cursor_row=$_zacrs_popup_cursor_row
    _zacrs_popup_snapshot_cursor_col=$cursor_col
    _zacrs_popup_snapshot_reuse_token="$reuse_token"
    _zacrs_popup_snapshot_from_gather=$from_gather
    _zacrs_popup_snapshot_columns=$COLUMNS
    _zacrs_popup_snapshot_lines=$LINES
}

# === Daemon lifecycle ===

# Try to load zsh/system for sysread (used by daemon complete)
zmodload zsh/system 2>/dev/null

# Try to load zsh/net/socket (preferred) or zsh/net/unix for zsocket support
if zmodload zsh/net/socket 2>/dev/null || zmodload zsh/net/unix 2>/dev/null; then
    _zacrs_socket_path="${XDG_RUNTIME_DIR:-/tmp}/zacrs.sock"
    [[ -z "$XDG_RUNTIME_DIR" ]] && _zacrs_socket_path="/tmp/zacrs-${USER:-unknown}.sock"

    _zacrs_mark_daemon_unavailable() {
        _zacrs_daemon_available=0
        _zacrs_daemon_next_retry=$(( SECONDS + 1 ))
    }

    _zacrs_ensure_daemon() {
        # Already confirmed available this session
        (( _zacrs_daemon_available )) && return 0
        # Check if daemon is running via zsocket ping
        local fd
        if zsocket "$_zacrs_socket_path" 2>/dev/null; then
            fd=$REPLY
            print -u $fd "ping"
            local resp
            read -r -u $fd resp 2>/dev/null
            exec {fd}<&-
            if [[ "$resp" == OK* ]]; then
                _zacrs_daemon_available=1
                _zacrs_daemon_started=0
                _zacrs_daemon_next_retry=0
                return 0
            fi
        fi
        # Start daemon
        "$ZACRS_BIN" daemon start &!
        # Wait briefly for socket to appear
        local i
        for (( i = 0; i < 10; i++ )); do
            [[ -S "$_zacrs_socket_path" ]] && break
            sleep 0.02
        done
        if [[ -S "$_zacrs_socket_path" ]]; then
            _zacrs_daemon_available=1
            _zacrs_daemon_started=1
            _zacrs_daemon_next_retry=0
            return 0
        fi

        _zacrs_mark_daemon_unavailable
        return 1
    }

    _zacrs_maybe_retry_daemon() {
        (( _zacrs_daemon_available )) && return 0
        (( SECONDS < _zacrs_daemon_next_retry )) && return 1
        _zacrs_ensure_daemon
    }
    _zacrs_ensure_daemon
fi

# === Render header parsing (shared by daemon + subprocess + cycle paths) ===

# Parse an "OK key=val ... <tty_len>" header into globals.
# Sets: _zacrs_popup_row, _zacrs_popup_height, _zacrs_popup_cursor_row,
#        _zacrs_cycle_filtered_count, _zacrs_cycle_selected_original_idx,
#        _zacrs_parsed_reuse_token, _zacrs_parsed_tty_len
_zacrs_parse_render_header() {
    local header="$1"
    _zacrs_parsed_reuse_token=""
    _zacrs_parsed_tty_len=0
    _zacrs_cycle_filtered_count=0
    _zacrs_cycle_selected_original_idx=-1
    local token
    for token in ${(s: :)header}; do
        local key="${token%%=*}" val="${token#*=}"
        case "$key" in
            OK)                     ;; # daemon header prefix — skip
            popup_row)              _zacrs_popup_row=$val ;;
            popup_height)           _zacrs_popup_height=$val ;;
            cursor_row)             _zacrs_popup_cursor_row=$val ;;
            reuse_token)            _zacrs_parsed_reuse_token=$val ;;
            filtered_count)         _zacrs_cycle_filtered_count=$val ;;
            selected_original_idx)  _zacrs_cycle_selected_original_idx=$val ;;
        esac
        [[ "$token" != *=* && "$token" != OK ]] && _zacrs_parsed_tty_len=$token
    done
}

# === Non-blocking render (auto-trigger) ===

_zacrs_render() {
    local prefix="$1" prefix_len="$2" candidates_str="$3" from_gather="${4:-0}" selected="${5:-}"
    local cursor_row=0 cursor_col=0
    _zacrs_get_cursor_pos
    _zacrs_cursor_stale=""  # auto-trigger: PENDING guards prevent stale bytes

    if (( !_zacrs_daemon_available )) && (( ${+functions[_zacrs_maybe_retry_daemon]} )); then
        _zacrs_maybe_retry_daemon
    fi

    # Try zsocket daemon path (no subprocess spawn)
    if (( _zacrs_daemon_available )); then
        local fd
        if zsocket "$_zacrs_socket_path" 2>/dev/null; then
            fd=$REPLY
            local cols rows
            cols=$COLUMNS rows=$LINES
            # Send request: header + prefix line + candidates + END marker
            local render_cmd="render $cursor_row $cursor_col $cols $rows"
            [[ -n "$selected" ]] && render_cmd+=" selected=$selected"
            print -u $fd -- "$render_cmd"
            printf '%s\n' "$prefix" >&$fd
            printf '%s\n' "$candidates_str" >&$fd
            print -u $fd "END"
            # Read response header
            local header
            IFS= read -r -u $fd header
            if [[ "$header" == OK* ]]; then
                local _prev_vis=$_zacrs_popup_visible _prev_row=$_zacrs_popup_row _prev_height=$_zacrs_popup_height
                _zacrs_parse_render_header "$header"
                local tty_len=$_zacrs_parsed_tty_len reuse_token="$_zacrs_parsed_reuse_token"
                # Atomic: clear all old rows + draw new popup (cursor hidden)
                local tty_ok=1
                {
                    printf '\e[?25l'
                    if (( _prev_vis )); then
                        printf '\e7'
                        local _oi
                        for (( _oi = 0; _oi < _prev_height; _oi++ )); do
                            printf '\e[%d;1H\e[2K' $(( _prev_row + _oi + 1 ))
                        done
                        printf '\e8'
                    fi
                    if (( tty_len > 0 )); then
                        if ! sysread -i $fd -o 1 -c $tty_len; then
                            tty_ok=0
                        fi
                    fi
                } > /dev/tty
                if (( tty_ok )); then
                    _zacrs_popup_visible=1
                    _zacrs_record_popup_snapshot "$prefix" "$prefix_len" "$candidates_str" "$cursor_col" "$reuse_token" "$from_gather"
                else
                    _zacrs_clear_popup
                    _zacrs_mark_daemon_unavailable
                fi
                exec {fd}<&-
                return
            elif [[ "$header" == EMPTY ]]; then
                _zacrs_clear_popup
                exec {fd}<&-
                return
            elif [[ "$header" == ERROR* ]]; then
                _zacrs_clear_popup
                _zacrs_mark_daemon_unavailable
            fi
            exec {fd}<&-
        fi
        # Socket connect failed, daemon may have died
        _zacrs_clear_popup
        _zacrs_mark_daemon_unavailable
    fi

    # Fallback: subprocess (clear stale popup before spawning)
    _zacrs_clear_popup
    local -a render_args
    render_args=(render --prefix "$prefix" --cursor-row "$cursor_row" --cursor-col "$cursor_col")
    [[ -n "$selected" ]] && render_args+=(--selected "$selected")
    local output
    output=$(printf '%s' "$candidates_str" | "$ZACRS_BIN" "${render_args[@]}")
    local exit_code=$?

    if [[ $exit_code -eq 0 && -n "$output" ]]; then
        _zacrs_popup_visible=1
        _zacrs_parse_render_header "$output"
        _zacrs_record_popup_snapshot "$prefix" "$prefix_len" "$candidates_str" "$cursor_col" "" "$from_gather"
    else
        _zacrs_reset_popup_snapshot
    fi
}

# === Clear popup (zsh-native, no process spawn) ===

_zacrs_clear_popup() {
    if (( ! _zacrs_popup_visible )); then
        _zacrs_reset_popup_snapshot
        return 0
    fi
    {
        printf '\e[?25l\e7'
        local i
        for (( i = 0; i < _zacrs_popup_height; i++ )); do
            printf '\e[%d;1H\e[2K' $(( _zacrs_popup_row + i + 1 ))
        done
        printf '\e8\e[?25h'
    } > /dev/tty
    _zacrs_popup_visible=0
    _zacrs_reset_popup_snapshot
}

# === Apply completion result to LBUFFER ===

_zacrs_apply_result() {
    local prefix_len="$1" result_code="$2" result_text="$3"
    local base
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi

    if [[ $result_code -eq 0 && -n "$result_text" ]]; then
        LBUFFER="${base}${result_text}"
        _zacrs_suppressed=0
    elif [[ $result_code -eq 2 && -n "$result_text" ]]; then
        LBUFFER="${base}${result_text}"
        _zacrs_suppressed=1
    elif [[ $result_code -eq 1 && -n "$result_text" ]]; then
        LBUFFER="${base}${result_text}"
        _zacrs_suppressed=0
    elif [[ $result_code -eq 1 ]]; then
        _zacrs_suppressed=0
    fi

    # Confirm (code 0) で末尾がスペース/スラッシュなら
    # prev_lbuffer を更新せず line-pre-redraw にチェーンさせる
    if [[ $result_code -eq 0 && "$LBUFFER" == *[\ /] ]]; then
        _zacrs_prev_lbuffer="$base"
        _zacrs_chain_retry=1
    else
        _zacrs_prev_lbuffer="$LBUFFER"
    fi
}

# === Daemon-based interactive complete (blocking, for Tab) ===

# Parse FRAME header into _f_popup_row, _f_popup_height, _f_cursor_row, _f_tty_len
_zacrs_complete_parse_frame() {
    local header="$1"
    _f_popup_row=0 _f_popup_height=0 _f_cursor_row=0 _f_tty_len=0
    local token
    local last_token=""
    for token in ${(s: :)header}; do
        local key="${token%%=*}" val="${token#*=}"
        case "$key" in
            popup_row)    _f_popup_row=$val ;;
            popup_height) _f_popup_height=$val ;;
            cursor_row)   _f_cursor_row=$val ;;
        esac
        last_token="$token"
    done
    # Last token (no '=' sign) is tty_len
    [[ "$last_token" != *=* ]] && _f_tty_len=$last_token
}

# Handle a daemon response header in the interactive loop.
# Reads:  header, fd, tty_wfd from caller scope
# Writes: _f_resp (frame|done|none|unknown)
#         result_code, result_text (on done)
#         _zacrs_popup_row, _zacrs_popup_height (on frame)
#         _f_popup_row, _f_popup_height, _f_cursor_row, _f_tty_len (via parse_frame)
_zacrs_complete_handle_response() {
    case "$header" in
        FRAME*)
            _zacrs_complete_parse_frame "$header"
            if (( _f_tty_len > 0 )); then
                sysread -i $fd -o $tty_wfd -c $_f_tty_len
            fi
            _zacrs_popup_row=$_f_popup_row
            _zacrs_popup_height=$_f_popup_height
            _f_resp=frame
            ;;
        DONE*)
            result_code="${${(s: :)header}[2]}"
            result_text="${header#DONE [0-9]## }"
            [[ "$result_text" == "$header" ]] && result_text=""
            _f_resp=done
            ;;
        NONE)
            _f_resp=none
            ;;
        *)
            _f_resp=unknown
            ;;
    esac
}

_zacrs_invoke_daemon() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}" reuse_visible="${6:-0}" reuse_token="${7:-}"
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
    fi

    local fd
    if ! zsocket "$_zacrs_socket_path" 2>/dev/null; then
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        return 1
    fi
    fd=$REPLY

    # Send complete request
    local req="complete $cursor_row $cursor_col $COLUMNS $LINES"
    (( reuse_visible )) && [[ -n "$reuse_token" ]] && req+=" reuse_token=$reuse_token"
    print -u $fd -- "$req"
    printf '%s\n' "$prefix" >&$fd
    printf '%s\n' "$candidates_str" >&$fd
    print -u $fd "END"

    local header
    IFS= read -r -u $fd header
    local result_code=1 result_text=""
    local have_initial_frame=0 initial_done=0
    case "$header" in
        FRAME*) have_initial_frame=1 ;;
        NONE)
            if (( ! reuse_visible )) || [[ -z "$reuse_token" ]]; then
                (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
                exec {fd}<&-
                [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
                _zacrs_cursor_stale=""
                return 1
            fi
            _zacrs_popup_visible=1
            ;;
        DONE*)
            local -a parts
            parts=( ${(s: :)header} )
            result_code="${parts[2]}"
            result_text="${header#DONE [0-9]## }"
            [[ "$result_text" == "$header" ]] && result_text=""
            initial_done=1
            ;;
        *)
            (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
            exec {fd}<&-
            [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
            _zacrs_cursor_stale=""
            return 1
            ;;
    esac

    if (( initial_done )); then
        exec {fd}<&-
        # No interactive loop to inject into; push stale bytes to ZLE
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        _zacrs_clear_popup
        _zacrs_apply_result "$prefix_len" "$result_code" "$result_text"
        zle reset-prompt
        return 0
    fi

    # Open /dev/tty fds
    local tty_rfd tty_wfd
    exec {tty_rfd}</dev/tty
    exec {tty_wfd}>/dev/tty

    local _f_popup_row _f_popup_height _f_cursor_row _f_tty_len
    if (( have_initial_frame )); then
        _zacrs_complete_parse_frame "$header"
        # Clear stale rows from previous popup that the new frame won't cover
        if (( _zacrs_popup_visible )); then
            printf '\e7' >&$tty_wfd
            local _si _row
            for (( _si = 0; _si < _zacrs_popup_height; _si++ )); do
                _row=$(( _zacrs_popup_row + _si ))
                if (( _row < _f_popup_row || _row >= _f_popup_row + _f_popup_height )); then
                    printf '\e[%d;1H\e[2K' $(( _row + 1 )) >&$tty_wfd
                fi
            done
            printf '\e8' >&$tty_wfd
        fi
        if (( _f_tty_len > 0 )); then
            sysread -i $fd -o $tty_wfd -c $_f_tty_len
        fi
        _zacrs_popup_visible=1
        _zacrs_popup_row=$_f_popup_row
        _zacrs_popup_height=$_f_popup_height
    fi

    # Enter raw mode
    local saved_stty
    saved_stty=$(stty -g < /dev/tty)
    stty raw -echo < /dev/tty

    {
        # Re-inject keystrokes that were consumed by the DSR query.
        # ESC-prefixed sequences (arrows, Home, End, Alt-...) are grouped
        # into a single KEY command, matching the main loop's behaviour.
        local _inject_done=0
        if [[ -n "$_zacrs_cursor_stale" ]]; then
            local _i _ch _key
            _i=1
            while (( _i <= ${#_zacrs_cursor_stale} )); do
                _ch="${_zacrs_cursor_stale[$_i]}"
                _key="$_ch"
                if [[ "$_ch" = $'\e' ]]; then
                    (( _i++ ))
                    while (( _i <= ${#_zacrs_cursor_stale} )); do
                        _ch="${_zacrs_cursor_stale[$_i]}"
                        _key+="$_ch"
                        (( _i++ ))
                        [[ "$_ch" =~ [A-Za-z~] ]] && break
                    done
                else
                    (( _i++ ))
                fi
                printf 'KEY %d\n%s' "${#_key}" "$_key" >&$fd
                IFS= read -r -u $fd header
                _zacrs_complete_handle_response
                case "$_f_resp" in
                    frame) ;;
                    done)  _inject_done=1; break ;;
                    none)  ;;
                    *)     _inject_done=1; break ;;
                esac
            done
            _zacrs_cursor_stale=""
        fi

        if (( ! _inject_done )); then
        while true; do
            # Read key bytes from /dev/tty
            local input=""
            sysread -i $tty_rfd -c 1 input || break
            if [[ "$input" = $'\e' ]]; then
                local extra=""
                while sysread -i $tty_rfd -c 1 -t 0 extra 2>/dev/null; do
                    input+="$extra"
                    extra=""
                done
            fi

            # Send to daemon
            printf 'KEY %d\n%s' "${#input}" "$input" >&$fd

            # Read response
            IFS= read -r -u $fd header
            _zacrs_complete_handle_response
            case "$_f_resp" in
                frame) ;;
                done)  break ;;
                none)  ;;
                *)     break ;;
            esac
        done
        fi # _inject_done
    } always {
        stty "$saved_stty" < /dev/tty
        exec {tty_rfd}<&- {tty_wfd}>&-
    }

    exec {fd}<&-
    _zacrs_clear_popup
    _zacrs_apply_result "$prefix_len" "$result_code" "$result_text"
    zle reset-prompt
    return 0
}

# === Core: invoke Rust binary (blocking, for Tab) ===

_zacrs_invoke() {
    local prefix="$1"
    local prefix_len="$2"
    local candidates_str="$3"

    local cursor_row="${4:-}" cursor_col="${5:-}"
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
        # Subprocess path has no interactive loop; push stale bytes
        # back to ZLE so they are processed after the widget returns.
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
    fi

    local output
    output=$(printf '%s' "$candidates_str" | \
        "$ZACRS_BIN" complete \
        --prefix "$prefix" \
        --cursor-row "$cursor_row" \
        --cursor-col "$cursor_col")
    local exit_code=$?

    unset POSTDISPLAY
    _zacrs_apply_result "$prefix_len" "$exit_code" "$output"
    zle reset-prompt
}

# === Cycle mode helpers ===

# Exit cycle mode and restore normal keymap
_zacrs_cycle_exit() {
    _zacrs_cycle_active=0
    zle -K "$_zacrs_cycle_prev_keymap"
    _zacrs_clear_popup
    _zacrs_prev_lbuffer="$LBUFFER"
    # Release potentially large strings
    _zacrs_cycle_candidates=""
    _zacrs_cycle_prefix=""
    _zacrs_cycle_original_lbuffer=""
}

# Apply the selected candidate text to LBUFFER
_zacrs_cycle_apply_selected() {
    if (( _zacrs_cycle_selected_original_idx >= 0 )); then
        local -a cands
        cands=( ${(f)_zacrs_cycle_candidates} )
        cands=( ${cands:#} )
        local sel_line="${cands[$((_zacrs_cycle_selected_original_idx + 1))]}"
        local sel_text="${sel_line%%	*}"
        local base
        if (( _zacrs_cycle_prefix_len > 0 )); then
            base="${_zacrs_cycle_original_lbuffer[1,-(${_zacrs_cycle_prefix_len}+1)]}"
        else
            base="$_zacrs_cycle_original_lbuffer"
        fi
        LBUFFER="${base}${sel_text}"
    fi
}

# Render popup with selected index and update LBUFFER.
# Uses daemon zsocket path directly to clear old popup + draw new popup
# atomically (in one output group) — prevents ghost borders and flicker.
# NOTE: daemon protocol mirrors _zacrs_render — keep both in sync.
_zacrs_cycle_render_and_apply() {
    local cursor_row=$_zacrs_cycle_cursor_row cursor_col=$_zacrs_cycle_cursor_col

    local _prev_row=$_zacrs_popup_row _prev_height=$_zacrs_popup_height _prev_vis=$_zacrs_popup_visible

    if (( _zacrs_daemon_available )); then
        local fd
        if zsocket "$_zacrs_socket_path" 2>/dev/null; then
            fd=$REPLY
            local render_cmd="render $cursor_row $cursor_col $COLUMNS $LINES selected=$_zacrs_cycle_index"
            print -u $fd -- "$render_cmd"
            printf '%s\n' "$_zacrs_cycle_prefix" >&$fd
            printf '%s\n' "$_zacrs_cycle_candidates" >&$fd
            print -u $fd "END"

            local header
            IFS= read -r -u $fd header
            if [[ "$header" == OK* ]]; then
                _zacrs_parse_render_header "$header"
                local tty_len=$_zacrs_parsed_tty_len reuse_token="$_zacrs_parsed_reuse_token"

                # Atomic: hide cursor + selective-clear stale rows + draw new popup
                {
                    printf '\e[?25l'
                    if (( _prev_vis )); then
                        printf '\e7'
                        local _oi _row
                        for (( _oi = 0; _oi < _prev_height; _oi++ )); do
                            _row=$(( _prev_row + _oi ))
                            if (( _row < _zacrs_popup_row || _row >= _zacrs_popup_row + _zacrs_popup_height )); then
                                printf '\e[%d;1H\e[2K' $(( _row + 1 ))
                            fi
                        done
                        printf '\e8'
                    fi
                    local tty_ok=1
                    if (( tty_len > 0 )); then
                        if ! sysread -i $fd -o 1 -c $tty_len; then
                            tty_ok=0
                        fi
                    fi
                } > /dev/tty

                if (( tty_ok )); then
                    _zacrs_popup_visible=1
                    _zacrs_record_popup_snapshot "$_zacrs_cycle_prefix" "$_zacrs_cycle_prefix_len" \
                        "$_zacrs_cycle_candidates" "$cursor_col" "$reuse_token" "0"
                else
                    _zacrs_clear_popup
                    _zacrs_mark_daemon_unavailable
                fi
                exec {fd}<&-
                return
            elif [[ "$header" == EMPTY ]]; then
                # No candidates match — exit cycle immediately (no fallback needed)
                exec {fd}<&-
                _zacrs_cycle_exit
                return
            fi
            # ERROR — fall through to subprocess fallback
            exec {fd}<&-
        fi
    fi

    # Fallback: general render path (subprocess, may have slight flicker)
    _zacrs_clear_popup
    _zacrs_render "$_zacrs_cycle_prefix" "$_zacrs_cycle_prefix_len" \
        "$_zacrs_cycle_candidates" "0" "$_zacrs_cycle_index"
    if (( ! _zacrs_popup_visible )); then
        _zacrs_cycle_active=0
        zle -K "$_zacrs_cycle_prev_keymap"
    fi
}

# === Cycle mode widgets ===

_zacrs_cycle_next() {
    (( _zacrs_cycle_filtered_count <= 0 )) && return
    _zacrs_cycle_index=$(( (_zacrs_cycle_index + 1) % _zacrs_cycle_filtered_count ))
    _zacrs_cycle_render_and_apply
}

_zacrs_cycle_prev() {
    (( _zacrs_cycle_filtered_count <= 0 )) && return
    _zacrs_cycle_index=$(( (_zacrs_cycle_index - 1 + _zacrs_cycle_filtered_count) % _zacrs_cycle_filtered_count ))
    _zacrs_cycle_render_and_apply
}

_zacrs_cycle_accept() {
    _zacrs_cycle_apply_selected
    _zacrs_cycle_exit
    _zacrs_reset_cache
    zle .accept-line
}

_zacrs_cycle_accept_space() {
    _zacrs_cycle_apply_selected
    LBUFFER+=" "
    _zacrs_cycle_exit
    _zacrs_suppressed=1
    _zacrs_chain_retry=1
    zle reset-prompt
}

_zacrs_cycle_cancel() {
    LBUFFER="$_zacrs_cycle_original_lbuffer"
    _zacrs_cycle_exit
    zle reset-prompt
}

_zacrs_cycle_passthrough() {
    _zacrs_cycle_apply_selected
    _zacrs_cycle_exit
    LBUFFER+="$KEYS"
    _zacrs_prev_lbuffer=""
    zle reset-prompt
}

# === Shared completion helpers ===

# Collect candidates: compsys → gather → fuzzy cache fallback.
# Caller must declare: local prefix prefix_len candidates_str
# (zsh dynamic scoping lets us write to these from here)
_zacrs_collect_candidates() {
    _zacrs_captured=()
    local _zacrs_fd2
    exec {_zacrs_fd2}>&2
    zle _zacrs_compsys 2>/dev/null
    exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-

    if (( _zacrs_ctx_valid )); then
        prefix="$_zacrs_ctx_prefix"
        prefix_len=$_zacrs_ctx_prefix_len
    else
        prefix="${LBUFFER##* }"
        prefix_len=${#prefix}
    fi

    if (( ${#_zacrs_captured} > 0 )); then
        candidates_str="${(pj:\n:)_zacrs_captured}"
    fi
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
        if [[ -n "$candidates_str" ]]; then
            prefix="${LBUFFER##* }"
            prefix_len=${#prefix}
        fi
    fi
    # Fuzzy fallback from auto-trigger cache
    if [[ -z "$candidates_str" && -n "$prefix" ]]; then
        local lbase
        if [[ "$LBUFFER" == *" "* ]]; then
            lbase="${LBUFFER% *} "
        else
            lbase=""
        fi
        if [[ "$lbase" == "$_zacrs_cached_lbase" && -n "$_zacrs_cached_candidates" ]]; then
            candidates_str="$_zacrs_cached_candidates"
        fi
    fi
}

# Handle single-candidate immediate completion.
# Args: $1=prefix $2=prefix_len $3=candidate_line (tab-separated)
_zacrs_apply_single_candidate() {
    local prefix="$1" prefix_len="$2" cand_line="$3"
    _zacrs_clear_popup
    local text="${cand_line%%	*}"
    local kind="${cand_line##*	}"
    local base
    local is_cmd_pos=0
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi
    _zacrs_is_cmd_pos "$LBUFFER" "$prefix" && is_cmd_pos=1
    LBUFFER="${base}${text}"
    case "$kind" in
        directory) [[ "$text" != */ ]] && LBUFFER+="/" ;;
        command|alias|builtin|function|file) LBUFFER+=" " ;;
        "")
            if (( is_cmd_pos )) && [[ "$text" != */ && "$text" != */* ]]; then
                LBUFFER+=" "
            fi
            ;;
    esac
    unset POSTDISPLAY
    if [[ "$LBUFFER" == *[\ /] ]]; then
        _zacrs_prev_lbuffer="$base"
        _zacrs_chain_retry=1
    else
        _zacrs_prev_lbuffer="$LBUFFER"
    fi
    zle reset-prompt
}

# === Tab-cycle completion widget (new default) ===

_zacrs_complete_cycle() {
    if (( _zacrs_cycle_active )); then
        _zacrs_cycle_next
        return
    fi

    local prefix="" prefix_len=0 candidates_str=""

    # Try to reuse visible popup (same checks as interactive mode)
    local reuse_visible=0
    if (( _zacrs_popup_visible )) \
        && [[ "$_zacrs_popup_snapshot_lbuffer" == "$LBUFFER" ]] \
        && [[ -n "$_zacrs_popup_snapshot_candidates" ]] \
        && (( _zacrs_popup_snapshot_columns == COLUMNS )) \
        && (( _zacrs_popup_snapshot_lines == LINES )) \
        && (( ! _zacrs_popup_snapshot_from_gather )); then
        reuse_visible=1
        prefix="$_zacrs_popup_snapshot_prefix"
        prefix_len=$_zacrs_popup_snapshot_prefix_len
        candidates_str="$_zacrs_popup_snapshot_candidates"
    fi

    if (( ! reuse_visible )); then
        _zacrs_collect_candidates
    fi

    # No candidates → default zsh completion
    if [[ -z "$candidates_str" ]]; then
        _zacrs_clear_popup
        zle expand-or-complete
        return
    fi

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )

    # Single candidate → immediate completion (no cycle needed)
    if [[ ${#cands[@]} -eq 1 ]]; then
        _zacrs_apply_single_candidate "$prefix" "$prefix_len" "${cands[1]}"
        return
    fi

    # Initialize cycle mode
    _zacrs_suppressed=0
    _zacrs_cycle_active=1
    _zacrs_cycle_index=0
    _zacrs_cycle_original_lbuffer="$LBUFFER"
    _zacrs_cycle_prefix="$prefix"
    _zacrs_cycle_prefix_len=$prefix_len
    _zacrs_cycle_candidates="$candidates_str"
    _zacrs_cycle_prev_keymap="$KEYMAP"

    # Cache cursor position for all cycle renders (avoids DSR flicker on Tab)
    if (( _zacrs_popup_visible )); then
        _zacrs_cycle_cursor_row=$_zacrs_popup_snapshot_cursor_row
        _zacrs_cycle_cursor_col=$_zacrs_popup_snapshot_cursor_col
    else
        local cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
        _zacrs_cursor_stale=""
        _zacrs_cycle_cursor_row=$cursor_row
        _zacrs_cycle_cursor_col=$cursor_col
    fi

    zle -K _zacrs_cycle
    _zacrs_cycle_render_and_apply
}

# === Interactive completion widget (blocking mode, legacy) ===

_zacrs_complete_interactive() {
    local prefix="" prefix_len=0 candidates_str=""
    local cursor_row="" cursor_col=""
    local reuse_visible=0
    local reuse_token=""

    if (( _zacrs_popup_visible )) \
        && [[ "$_zacrs_popup_snapshot_lbuffer" == "$LBUFFER" ]] \
        && [[ -n "$_zacrs_popup_snapshot_candidates" ]] \
        && (( _zacrs_popup_snapshot_columns == COLUMNS )) \
        && (( _zacrs_popup_snapshot_lines == LINES )) \
        && (( ! _zacrs_popup_snapshot_from_gather )); then
        reuse_visible=1
        prefix="$_zacrs_popup_snapshot_prefix"
        prefix_len=$_zacrs_popup_snapshot_prefix_len
        candidates_str="$_zacrs_popup_snapshot_candidates"
        cursor_row=$_zacrs_popup_snapshot_cursor_row
        cursor_col=$_zacrs_popup_snapshot_cursor_col
        reuse_token="$_zacrs_popup_snapshot_reuse_token"
    fi

    if (( ! reuse_visible )); then
        _zacrs_collect_candidates
    fi

    # 候補なし → default zsh 補完にフォールバック
    if [[ -z "$candidates_str" ]]; then
        _zacrs_clear_popup
        zle expand-or-complete
        return
    fi

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )

    # 単一候補 → 即補完
    if [[ ${#cands[@]} -eq 1 ]]; then
        _zacrs_apply_single_candidate "$prefix" "$prefix_len" "${cands[1]}"
        return
    fi

    _zacrs_suppressed=0

    # Try daemon path first, fall back to subprocess
    if (( _zacrs_daemon_available )); then
        _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" \
            "$cursor_row" "$cursor_col" "$reuse_visible" "$reuse_token" && return
    fi
    if (( ! reuse_visible )); then
        _zacrs_clear_popup
    fi
    _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" "$cursor_row" "$cursor_col"
}

# === Auto-trigger via line-pre-redraw hook ===

_zacrs_line_pre_redraw() {
    # Cycle mode: render は widget 側で直接行うため auto-trigger は抑制
    if (( _zacrs_cycle_active )); then
        # 未処理キーで LBUFFER が変わった場合、サイクルモードを強制終了
        if [[ "$LBUFFER" != "$_zacrs_cycle_original_lbuffer" ]]; then
            _zacrs_cycle_exit
            # fall through to normal auto-trigger flow
        else
            return
        fi
    fi
    # LBUFFER が変わってなければスキップ
    if [[ "$LBUFFER" != "$_zacrs_prev_lbuffer" ]]; then
        # Defer popup clear — _zacrs_render will do selective clear to avoid flicker.
        # Just invalidate the snapshot so Tab reuse check uses fresh data.
        _zacrs_reset_popup_snapshot
    else
        return
    fi
    # Type-ahead detected: skip heavy work.  Do NOT update
    # _zacrs_prev_lbuffer so the next redraw retries this buffer.
    (( PENDING > 0 )) && return

    _zacrs_prev_lbuffer="$LBUFFER"

    # 空 or 空白のみ → コマンド未入力なのでスキップ
    [[ ! "$LBUFFER" =~ [^[:space:]] ]] && { _zacrs_clear_popup; return }

    # DismissWithSpace 後の抑制: 非空 prefix 入力で解除 (naive prefix で十分)
    local naive_prefix="${LBUFFER##* }"
    if (( _zacrs_suppressed )); then
        if [[ -n "$naive_prefix" ]]; then
            _zacrs_suppressed=0
        else
            _zacrs_clear_popup; return
        fi
    fi

    # lbase 計算: 最後のスペースより前の部分（コマンド＋引数の文脈）
    local lbase
    if [[ "$LBUFFER" == *" "* ]]; then
        lbase="${LBUFFER% *} "
    else
        lbase=""
    fi

    # lbase が変わったらキャッシュ無効化
    if [[ "$lbase" != "$_zacrs_cached_lbase" ]]; then
        _zacrs_reset_cache
        _zacrs_cached_lbase="$lbase"
    fi

    # 候補収集: compsys → gather fallback
    local candidates_str="" from_gather=0
    _zacrs_captured=()
    local _zacrs_fd2
    exec {_zacrs_fd2}>&2
    zle _zacrs_compsys 2>/dev/null
    exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-

    # compsys コンテキストから prefix 取得 (render 用、LBUFFER 置換なし)
    local prefix prefix_len
    if (( _zacrs_ctx_valid )); then
        prefix="$_zacrs_ctx_prefix"
        prefix_len=$_zacrs_ctx_prefix_len
    else
        prefix="$naive_prefix"
        prefix_len=${#naive_prefix}
    fi

    if (( ${#_zacrs_captured} > 0 )); then
        candidates_str="${(pj:\n:)_zacrs_captured}"
    fi
    # compsys 0 件 → 遅延ロード補完のためリトライ1回
    # チェーン時 or サブコマンド位置 (naive_prefix 空) で発動
    if (( ${#_zacrs_captured} == 0 )) && { (( _zacrs_chain_retry )) || [[ -z "$naive_prefix" ]]; }; then
        _zacrs_chain_retry=0
        _zacrs_captured=()
        exec {_zacrs_fd2}>&2
        zle _zacrs_compsys 2>/dev/null
        exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-
        if (( _zacrs_ctx_valid )); then
            prefix="$_zacrs_ctx_prefix"
            prefix_len=$_zacrs_ctx_prefix_len
        fi
        if (( ${#_zacrs_captured} > 0 )); then
            candidates_str="${(pj:\n:)_zacrs_captured}"
        fi
    fi
    _zacrs_chain_retry=0
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
        if [[ -n "$candidates_str" ]]; then
            prefix="$naive_prefix"
            prefix_len=${#naive_prefix}
            from_gather=1
        fi
    fi

    # キャッシュ更新: 同じ lbase で初めて候補が見つかった場合
    if [[ -n "$candidates_str" && -z "$_zacrs_cached_candidates" ]]; then
        _zacrs_cached_candidates="$candidates_str"
        _zacrs_cached_from_gather=$from_gather
    fi

    # Fuzzy fallback: 候補なし → キャッシュから再利用
    if [[ -z "$candidates_str" && -n "$_zacrs_cached_candidates" ]]; then
        candidates_str="$_zacrs_cached_candidates"
        from_gather=$_zacrs_cached_from_gather
        prefix="$naive_prefix"
        prefix_len=${#naive_prefix}
    fi

    [[ -z "$candidates_str" ]] && { _zacrs_clear_popup; return }

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )
    [[ ${#cands[@]} -eq 0 ]] && { _zacrs_clear_popup; return }

    # Type-ahead arrived during candidate gathering: skip render.
    # Reset prev_lbuffer so the next redraw retries this buffer.
    if (( PENDING > 0 )); then
        _zacrs_prev_lbuffer=""
        return
    fi

    _zacrs_render "$prefix" "$prefix_len" "$candidates_str" "$from_gather"
}

# === Widget wrappers: Enter/Ctrl-C でポップアップクリア ===

_zacrs_accept_line() {
    if (( _zacrs_cycle_active )); then
        _zacrs_cycle_exit
    else
        _zacrs_clear_popup
        _zacrs_prev_lbuffer="$LBUFFER"
    fi
    _zacrs_reset_cache
    zle .accept-line
}
zle -N accept-line _zacrs_accept_line

_zacrs_send_break() {
    if (( _zacrs_cycle_active )); then
        LBUFFER="$_zacrs_cycle_original_lbuffer"
        _zacrs_cycle_exit
    else
        _zacrs_clear_popup
    fi
    _zacrs_prev_lbuffer=""
    _zacrs_reset_cache
    zle .send-break
}
zle -N send-break _zacrs_send_break

# === ターミナルリサイズ対応 ===

TRAPWINCH() {
    if (( _zacrs_cycle_active )); then
        LBUFFER="$_zacrs_cycle_original_lbuffer"
        _zacrs_cycle_exit
    else
        _zacrs_clear_popup
    fi
}

# === Register widgets and keybindings ===

# Cycle mode widgets
zle -N _zacrs_complete_cycle
zle -N _zacrs_cycle_next
zle -N _zacrs_cycle_prev
zle -N _zacrs_cycle_accept
zle -N _zacrs_cycle_accept_space
zle -N _zacrs_cycle_cancel
zle -N _zacrs_cycle_passthrough

# Interactive mode widget (legacy blocking mode)
zle -N _zacrs_complete_interactive

# Default: cycle mode
bindkey '^I' _zacrs_complete_cycle

# Cycle mode keymap (active only during Tab cycling)
bindkey -N _zacrs_cycle main
bindkey -M _zacrs_cycle '^I'   _zacrs_cycle_next
bindkey -M _zacrs_cycle '^[[Z' _zacrs_cycle_prev       # Shift-Tab
bindkey -M _zacrs_cycle '^[[B' _zacrs_cycle_next       # Down arrow
bindkey -M _zacrs_cycle '^[[A' _zacrs_cycle_prev       # Up arrow
bindkey -M _zacrs_cycle '^M'   _zacrs_cycle_accept     # Enter
bindkey -M _zacrs_cycle ' '    _zacrs_cycle_accept_space
bindkey -M _zacrs_cycle '^['   _zacrs_cycle_cancel     # Escape
bindkey -M _zacrs_cycle '^C'   _zacrs_cycle_cancel
bindkey -M _zacrs_cycle '^?'   _zacrs_cycle_cancel     # DEL (Backspace)
bindkey -M _zacrs_cycle '^H'   _zacrs_cycle_cancel     # BS (Backspace alt)
# Printable ASCII: accept current completion and insert the character
bindkey -M _zacrs_cycle -R '!'-'~' _zacrs_cycle_passthrough
# Shift-Tab via terminfo (if available)
[[ -n "$terminfo[kcbt]" ]] && bindkey -M _zacrs_cycle "$terminfo[kcbt]" _zacrs_cycle_prev

# Register line-pre-redraw hook (auto-trigger without key rebinding)
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _zacrs_line_pre_redraw
