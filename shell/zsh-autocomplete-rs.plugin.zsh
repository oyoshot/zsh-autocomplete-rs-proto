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
typeset -gi _zacrs_last_render_cursor_row=0
typeset -gi _zacrs_last_render_cursor_col=0
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
typeset -gi _zacrs_cached_prefix_len=0
typeset -gF _zacrs_debounce_until=0.0

# Render header parse results (set by _zacrs_parse_render_header)
typeset -g  _zacrs_parsed_reuse_token=""
typeset -gi _zacrs_parsed_tty_len=0
# Daemon send helper (set by _zacrs_daemon_send_render on OK)
typeset -gi _zacrs_send_render_fd=0

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
    _zacrs_cached_prefix_len=0
    _zacrs_cached_lbase=""
    _zacrs_debounce_until=0.0
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
# Try to load zsh/datetime for EPOCHREALTIME (debounce timestamps)
zmodload zsh/datetime 2>/dev/null

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

# === Render header parsing (shared by daemon + subprocess paths) ===

# Parse an "OK key=val ... <tty_len>" header into globals.
# Sets: _zacrs_popup_row, _zacrs_popup_height, _zacrs_popup_cursor_row,
#        _zacrs_parsed_reuse_token, _zacrs_parsed_tty_len
_zacrs_parse_render_header() {
    local header="$1"
    _zacrs_parsed_reuse_token=""
    _zacrs_parsed_tty_len=0
    local token
    for token in ${(s: :)header}; do
        local key="${token%%=*}" val="${token#*=}"
        case "$key" in
            OK)                     ;; # daemon header prefix — skip
            popup_row)              _zacrs_popup_row=$val ;;
            popup_height)           _zacrs_popup_height=$val ;;
            cursor_row)             _zacrs_popup_cursor_row=$val ;;
            reuse_token)            _zacrs_parsed_reuse_token=$val ;;
            tty_len)                _zacrs_parsed_tty_len=$val ;;
        esac
    done
}

# Connect to daemon, send a render request, and parse the response header.
# Args: $1=cursor_row $2=cursor_col $3=prefix $4=candidates $5=selected (optional)
# On OK  (return 0): _zacrs_send_render_fd holds open fd; caller must sysread + close.
# On EMPTY (return 1): fd already closed.
# On ERROR/connect failure (return 2): fd already closed, daemon marked unavailable.
_zacrs_daemon_send_render() {
    local _cr="$1" _cc="$2" _pfx="$3" _cands="$4" _sel="$5"
    local fd
    if ! zsocket "$_zacrs_socket_path" 2>/dev/null; then
        _zacrs_mark_daemon_unavailable
        return 2
    fi
    fd=$REPLY
    local render_cmd="render $_cr $_cc $COLUMNS $LINES"
    [[ -n "$_sel" ]] && render_cmd+=" selected=$_sel"
    print -u $fd -- "$render_cmd"
    printf '%s\n' "$_pfx" >&$fd
    printf '%s\n' "$_cands" >&$fd
    print -u $fd "END"
    local header
    IFS= read -r -u $fd header
    if [[ "$header" == OK* ]]; then
        _zacrs_parse_render_header "$header"
        _zacrs_send_render_fd=$fd
        return 0
    elif [[ "$header" == EMPTY ]]; then
        exec {fd}<&-
        return 1
    else
        exec {fd}<&-
        _zacrs_mark_daemon_unavailable
        return 2
    fi
}

# Atomic clear+draw: hide cursor, clear stale rows, pipe tty bytes, show cursor.
# Args: $1=fd $2=tty_len $3=prev_vis $4=prev_row $5=prev_height $6=selective(0|1)
# When selective=1, only clears rows outside the *new* popup region
# (_zacrs_popup_row / _zacrs_popup_height must already be set by _zacrs_parse_render_header).
# Returns 0 on success, 1 on sysread failure.
_zacrs_daemon_draw_atomic() {
    local _da_fd=$1 _da_tty_len=$2 _da_prev_vis=$3 _da_prev_row=$4 _da_prev_height=$5 _da_selective=${6:-0}
    local _da_ok=0
    # Build the entire clear+draw sequence into a single buffer so that
    # it reaches the terminal as one write() — no intermediate frames.
    # Synchronized Output markers (\e[?2026h/l) are embedded inside the
    # same write to avoid nesting conflicts with ZSH's own sync regions.
    local _da_buf=$'\e[?2026h\e[?25l'
    if (( _da_prev_vis )); then
        _da_buf+=$'\e7'
        local _oi
        for (( _oi = 0; _oi < _da_prev_height; _oi++ )); do
            if (( _da_selective )); then
                local _row=$(( _da_prev_row + _oi ))
                if (( _row < _zacrs_popup_row || _row >= _zacrs_popup_row + _zacrs_popup_height )); then
                    _da_buf+=$'\e['"$(( _row + 1 ))"$';1H\e[2K'
                fi
            else
                _da_buf+=$'\e['"$(( _da_prev_row + _oi + 1 ))"$';1H\e[2K'
            fi
        done
        _da_buf+=$'\e8'
    fi
    if (( _da_tty_len > 0 )); then
        local _da_tty_data=""
        if sysread -i $_da_fd -c $_da_tty_len _da_tty_data; then
            _da_buf+="$_da_tty_data"
        else
            _da_ok=1
        fi
    fi
    _da_buf+=$'\e[?25h\e[?2026l'
    if (( _da_ok == 0 )); then
        printf '%s' "$_da_buf" > /dev/tty
    fi
    return $_da_ok
}

# === Non-blocking render (auto-trigger) ===

_zacrs_render() {
    local prefix="$1" prefix_len="$2" candidates_str="$3" from_gather="${4:-0}" selected="${5:-}"
    local cursor_row=0 cursor_col=0
    # When the popup is already on screen and the terminal hasn't resized,
    # reuse the previous cursor position instead of querying the terminal.
    # This eliminates the \e[6n round-trip (an extra /dev/tty write + read
    # loop) that can trigger a mid-render terminal flush on some platforms.
    if (( _zacrs_popup_visible
            && _zacrs_last_render_cursor_row > 0
            && COLUMNS == _zacrs_popup_snapshot_columns
            && LINES == _zacrs_popup_snapshot_lines )); then
        cursor_row=$_zacrs_last_render_cursor_row
        cursor_col=$_zacrs_last_render_cursor_col
    else
        _zacrs_get_cursor_pos
        _zacrs_cursor_stale=""  # auto-trigger: PENDING guards prevent stale bytes
    fi

    if (( !_zacrs_daemon_available )) && (( ${+functions[_zacrs_maybe_retry_daemon]} )); then
        _zacrs_maybe_retry_daemon
    fi

    # Try zsocket daemon path (no subprocess spawn)
    if (( _zacrs_daemon_available )); then
        local _prev_vis=$_zacrs_popup_visible _prev_row=$_zacrs_popup_row _prev_height=$_zacrs_popup_height
        _zacrs_daemon_send_render "$cursor_row" "$cursor_col" "$prefix" "$candidates_str" "$selected"
        local _send_rc=$?
        if (( _send_rc == 0 )); then
            local fd=$_zacrs_send_render_fd
            local tty_len=$_zacrs_parsed_tty_len reuse_token="$_zacrs_parsed_reuse_token"
            local tty_ok=1
            _zacrs_daemon_draw_atomic $fd $tty_len $_prev_vis $_prev_row $_prev_height 0 || tty_ok=0
            if (( tty_ok )); then
                _zacrs_popup_visible=1
                _zacrs_last_render_cursor_row=$cursor_row
                _zacrs_last_render_cursor_col=$cursor_col
                _zacrs_record_popup_snapshot "$prefix" "$prefix_len" "$candidates_str" "$cursor_col" "$reuse_token" "$from_gather"
            else
                _zacrs_clear_popup
                _zacrs_mark_daemon_unavailable
            fi
            exec {fd}<&-
            return
        elif (( _send_rc == 1 )); then
            _zacrs_clear_popup
            return
        fi
        # _send_rc == 2: daemon unavailable, fall through to subprocess
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
        _zacrs_last_render_cursor_row=$cursor_row
        _zacrs_last_render_cursor_col=$cursor_col
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
    local _cb=$'\e[?2026h\e[?25l\e7'
    local i
    for (( i = 0; i < _zacrs_popup_height; i++ )); do
        _cb+=$'\e['"$(( _zacrs_popup_row + i + 1 ))"$';1H\e[2K'
    done
    _cb+=$'\e8\e[?25h\e[?2026l'
    printf '%s' "$_cb" > /dev/tty
    _zacrs_popup_visible=0
    _zacrs_last_render_cursor_row=0
    _zacrs_last_render_cursor_col=0
    _zacrs_reset_popup_snapshot
}

# === Apply completion result to LBUFFER ===

_zacrs_apply_result() {
    local prefix_len="$1" result_code="$2" result_text="$3" execute_after_apply="${4:-0}"
    local base
    local new_lbuffer="$LBUFFER"
    local should_chain=0
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi

    # Passthrough (code 3): preserve POSTDISPLAY so zsh-autosuggestions
    # can still accept the suggestion when the re-injected key fires.
    [[ $result_code -ne 3 ]] && unset POSTDISPLAY

    if [[ $result_code -eq 0 && -n "$result_text" ]]; then
        new_lbuffer="${base}${result_text}"
        _zacrs_suppressed=0
    elif [[ $result_code -eq 2 && -n "$result_text" ]]; then
        new_lbuffer="${base}${result_text}"
        _zacrs_suppressed=0
    elif [[ $result_code -eq 3 ]]; then
        _zacrs_suppressed=0
    elif [[ $result_code -eq 1 && -n "$result_text" ]]; then
        new_lbuffer="${base}${result_text}"
        _zacrs_suppressed=0
    elif [[ $result_code -eq 1 ]]; then
        _zacrs_suppressed=0
    fi

    BUFFER="${new_lbuffer}${RBUFFER}"
    CURSOR=${#new_lbuffer}

    if [[ ( $result_code -eq 0 || $result_code -eq 2 ) && "$new_lbuffer" == *[\ /] ]]; then
        should_chain=1
    fi

    if (( execute_after_apply )) && [[ $result_code -eq 0 ]]; then
        _zacrs_prev_lbuffer="$new_lbuffer"
        _zacrs_chain_retry=0
        zle reset-prompt
        zle accept-line
        return
    fi

    # 補完適用後 (code 0/2) に末尾がスペース/スラッシュなら
    # prev_lbuffer を更新せず line-pre-redraw にチェーンさせる
    if (( should_chain )); then
        _zacrs_prev_lbuffer="$base"
        _zacrs_chain_retry=1
    else
        _zacrs_prev_lbuffer="$new_lbuffer"
    fi
}

# === Daemon-based popup session (Rust owns input after Tab) ===

# Parse FRAME header into _f_popup_row, _f_popup_height, _f_cursor_row, _f_tty_len, _f_common_prefix
_zacrs_complete_parse_frame() {
    local header="$1"
    _f_popup_row=0 _f_popup_height=0 _f_cursor_row=0 _f_tty_len=0 _f_common_prefix=""
    local token
    local last_token=""
    for token in ${(s: :)header}; do
        local key="${token%%=*}" val="${token#*=}"
        case "$key" in
            popup_row)      _f_popup_row=$val ;;
            popup_height)   _f_popup_height=$val ;;
            cursor_row)     _f_cursor_row=$val ;;
            common_prefix)  _f_common_prefix=$val ;;
        esac
        last_token="$token"
    done
    # Last token (no '=' sign) is tty_len
    [[ "$last_token" != *=* ]] && _f_tty_len=$last_token
}

# Handle a daemon response header in the popup-session loop.
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

# Shared popup-session loop used by both daemon and subprocess (coproc) paths.
# Reads:  have_initial_frame, header from caller scope
# Writes: result_code, result_text, passthrough_input to caller scope
# Args:   $1 = read fd   $2 = write fd (may equal read fd for daemon socket)
_zacrs_popup_session_loop() {
    local read_fd=$1 write_fd=$2
    # Alias read_fd as 'fd' so _zacrs_complete_handle_response can use it
    local fd=$read_fd

    # Open /dev/tty fds
    local tty_rfd tty_wfd
    exec {tty_rfd}</dev/tty
    exec {tty_wfd}>/dev/tty

    local _f_popup_row _f_popup_height _f_cursor_row _f_tty_len _f_common_prefix
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
        # Auto-insert unambiguous prefix: update LBUFFER when common prefix extends prefix
        if [[ -n "$_f_common_prefix" && ${#_f_common_prefix} -gt ${#prefix} ]]; then
            local _aiu_base="${LBUFFER%${prefix}}"
            LBUFFER="${_aiu_base}${_f_common_prefix}"
            CURSOR=$#LBUFFER
        fi
    fi

    # Enter raw mode
    local saved_stty
    saved_stty=$(stty -g < /dev/tty)
    stty raw -echo < /dev/tty

    {
        # Re-inject keystrokes that were consumed by the DSR query.
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
                _zacrs_send_key_input $write_fd "$_key"
                IFS= read -r -u $read_fd header
                _zacrs_complete_handle_response
                case "$_f_resp" in
                    frame) ;;
                    done)
                        (( result_code == 3 )) && passthrough_input="$_key"
                        _inject_done=1
                        break
                        ;;
                    none)  ;;
                    *)     _inject_done=1; break ;;
                esac
            done
            _zacrs_cursor_stale=""
        fi

        if (( ! _inject_done )); then
        while true; do
            local input=""
            _zacrs_read_key_input $tty_rfd || break
            input="$REPLY"

            _zacrs_send_key_input $write_fd "$input"

            IFS= read -r -u $read_fd header
            _zacrs_complete_handle_response
            case "$_f_resp" in
                frame) ;;
                done)
                    (( result_code == 3 )) && passthrough_input="$input"
                    break
                    ;;
                none)  ;;
                *)     break ;;
            esac
        done
        fi # _inject_done
    } always {
        stty "$saved_stty" < /dev/tty
        exec {tty_rfd}<&- {tty_wfd}>&-
    }
}

_zacrs_invoke_daemon() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}" reuse_visible="${6:-0}" reuse_token="${7:-}"
    local passthrough_input=""
    local shift_tab_hex=""
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
    [[ -n "$terminfo[kcbt]" ]] && shift_tab_hex="$(_zacrs_encode_hex_input "$terminfo[kcbt]")"
    (( reuse_visible )) && [[ -n "$reuse_token" ]] && req+=" reuse_token=$reuse_token"
    [[ -n "$shift_tab_hex" ]] && req+=" shift_tab_hex=$shift_tab_hex"
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
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        _zacrs_clear_popup
        _zacrs_apply_result "$prefix_len" "$result_code" "$result_text"
        zle reset-prompt
        return 0
    fi

    _zacrs_popup_session_loop $fd $fd

    exec {fd}<&-
    _zacrs_clear_popup
    _zacrs_apply_result "$prefix_len" "$result_code" "$result_text" 1
    [[ $result_code -eq 3 && -n "$passthrough_input" ]] && zle -U "$passthrough_input"
    [[ $result_code -ne 0 ]] && zle reset-prompt
    return 0
}

# === Core: invoke Rust binary via coproc (blocking, for Tab) ===

_zacrs_invoke() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}"
    local passthrough_input=""
    local shift_tab_hex=""
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
    fi
    [[ -n "$terminfo[kcbt]" ]] && shift_tab_hex="$(_zacrs_encode_hex_input "$terminfo[kcbt]")"

    local -a complete_args
    complete_args=(
        complete
        --prefix "$prefix"
        --cursor-row "$cursor_row"
        --cursor-col "$cursor_col"
        --cols "$COLUMNS"
        --rows "$LINES"
    )
    [[ -n "$shift_tab_hex" ]] && complete_args+=(--shift-tab-hex "$shift_tab_hex")

    # Launch subprocess as coproc
    coproc { "$ZACRS_BIN" "${complete_args[@]}" }
    local coproc_pid=$!
    local coproc_rfd coproc_wfd
    exec {coproc_rfd}<&p {coproc_wfd}>&p

    # Send candidates + END marker to coproc stdin
    printf '%s\n' "$candidates_str" >&$coproc_wfd
    print -u $coproc_wfd "END"

    # Read initial response
    local header
    IFS= read -r -u $coproc_rfd header
    local result_code=1 result_text=""
    local have_initial_frame=0 initial_done=0
    case "$header" in
        FRAME*) have_initial_frame=1 ;;
        DONE*)
            local -a parts
            parts=( ${(s: :)header} )
            result_code="${parts[2]}"
            result_text="${header#DONE [0-9]## }"
            [[ "$result_text" == "$header" ]] && result_text=""
            initial_done=1
            ;;
        *)
            exec {coproc_rfd}<&- {coproc_wfd}>&-
            wait $coproc_pid 2>/dev/null
            [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
            _zacrs_cursor_stale=""
            return 1
            ;;
    esac

    if (( initial_done )); then
        exec {coproc_rfd}<&- {coproc_wfd}>&-
        wait $coproc_pid 2>/dev/null
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        _zacrs_clear_popup
        _zacrs_apply_result "$prefix_len" "$result_code" "$result_text"
        zle reset-prompt
        return 0
    fi

    _zacrs_popup_session_loop $coproc_rfd $coproc_wfd

    exec {coproc_rfd}<&- {coproc_wfd}>&-
    wait $coproc_pid 2>/dev/null
    _zacrs_clear_popup
    _zacrs_apply_result "$prefix_len" "$result_code" "$result_text" 1
    [[ $result_code -eq 3 && -n "$passthrough_input" ]] && zle -U "$passthrough_input"
    [[ $result_code -ne 0 ]] && zle reset-prompt
}

_zacrs_send_key_input() {
    local fd="$1" input="$2"
    local LC_ALL=C
    printf 'KEY %d\n%s' "${#input}" "$input" >&$fd
}

_zacrs_read_key_input() {
    local fd="$1"
    local input=""
    sysread -i $fd -c 1 input || return 1
    if [[ "$input" = $'\e' ]]; then
        local extra=""
        while sysread -i $fd -c 1 -t 0.05 extra 2>/dev/null; do
            input+="$extra"
            extra=""
        done
    else
        local -i expected_len=1 nbytes=1 ord=0
        LC_ALL=C printf -v ord '%d' "'$input" 2>/dev/null
        if (( ord >= 0xc0 && ord <= 0xdf )); then
            expected_len=2
        elif (( ord >= 0xe0 && ord <= 0xef )); then
            expected_len=3
        elif (( ord >= 0xf0 && ord <= 0xf7 )); then
            expected_len=4
        fi
        if (( expected_len > 1 )); then
            local extra=""
            while (( nbytes < expected_len )) && sysread -i $fd -c 1 -t 0.05 extra 2>/dev/null; do
                input+="$extra"
                (( nbytes++ ))
                extra=""
            done
        fi
    fi
    REPLY="$input"
}

_zacrs_encode_hex_input() {
    local input="$1"
    [[ -z "$input" ]] && return 0
    print -rn -- "$input" | od -An -tx1 -v | tr -d ' \n'
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
    local new_lbuffer
    local is_cmd_pos=0
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi
    _zacrs_is_cmd_pos "$LBUFFER" "$prefix" && is_cmd_pos=1
    unset POSTDISPLAY
    new_lbuffer="${base}${text}"
    case "$kind" in
        directory) [[ "$text" != */ ]] && new_lbuffer+="/" ;;
        command|alias|builtin|function|file) new_lbuffer+=" " ;;
        "")
            if (( is_cmd_pos )) && [[ "$text" != */ && "$text" != */* ]]; then
                new_lbuffer+=" "
            fi
            ;;
    esac
    BUFFER="${new_lbuffer}${RBUFFER}"
    CURSOR=${#new_lbuffer}
    if [[ "$new_lbuffer" == *[\ /] ]]; then
        _zacrs_prev_lbuffer="$base"
        _zacrs_chain_retry=1
    else
        _zacrs_prev_lbuffer="$new_lbuffer"
    fi
    zle reset-prompt
}

# === Popup completion widget ===

_zacrs_complete_popup() {
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

    # 候補収集: cache-first → compsys → gather fallback
    local candidates_str="" from_gather=0
    local prefix prefix_len

    # Cache-first: lbase unchanged + cache available → skip compsys/gather.
    # Rust-side filtering handles the current prefix, so the full candidate
    # list from cache is sufficient.  This eliminates per-keystroke compsys
    # overhead during rapid Backspace (same command context, shrinking prefix).
    if [[ -n "$_zacrs_cached_candidates" ]] && (( ! _zacrs_chain_retry )) && (( ${#naive_prefix} >= _zacrs_cached_prefix_len )); then
        candidates_str="$_zacrs_cached_candidates"
        from_gather=$_zacrs_cached_from_gather
        prefix="$naive_prefix"
        prefix_len=${#naive_prefix}
        _zacrs_chain_retry=0
    else
        # Heavy path: compsys + gather (no cache available).
        # Debounce: when keystrokes arrive faster than compsys can complete,
        # skip this cycle and let the next line-pre-redraw retry.
        if (( ${+EPOCHREALTIME} )) && (( EPOCHREALTIME < _zacrs_debounce_until )); then
            _zacrs_clear_popup
            _zacrs_prev_lbuffer=""
            return
        fi

        _zacrs_captured=()
        local _zacrs_fd2
        exec {_zacrs_fd2}>&2
        zle _zacrs_compsys 2>/dev/null
        exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-

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

        # キャッシュ更新: heavy path で初めて候補が見つかった場合
        if [[ -n "$candidates_str" ]]; then
            _zacrs_cached_candidates="$candidates_str"
            _zacrs_cached_from_gather=$from_gather
            _zacrs_cached_prefix_len=$prefix_len
        fi

        # Fuzzy fallback: 候補なし → キャッシュから再利用
        if [[ -z "$candidates_str" && -n "$_zacrs_cached_candidates" ]]; then
            candidates_str="$_zacrs_cached_candidates"
            from_gather=$_zacrs_cached_from_gather
            prefix="$naive_prefix"
            prefix_len=${#naive_prefix}
        fi

        # Heavy path 完了後、デバウンスウィンドウを設定 (50ms)
        (( ${+EPOCHREALTIME} )) && _zacrs_debounce_until=$(( EPOCHREALTIME + 0.050 ))
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
    _zacrs_clear_popup
    _zacrs_prev_lbuffer="$LBUFFER"
    _zacrs_chain_retry=0
    _zacrs_reset_cache
    zle reset-prompt
    zle .accept-line
}
zle -N accept-line _zacrs_accept_line

_zacrs_send_break() {
    _zacrs_clear_popup
    _zacrs_prev_lbuffer=""
    _zacrs_reset_cache
    zle .send-break
}
zle -N send-break _zacrs_send_break

# === ターミナルリサイズ対応 ===

TRAPWINCH() {
    _zacrs_clear_popup
    _zacrs_reset_cache
}

# === Register widgets and keybindings ===

# Popup widget
zle -N _zacrs_complete_popup

# Default: Tab enters the Rust-owned popup session.
bindkey '^I' _zacrs_complete_popup

# Register line-pre-redraw hook (auto-trigger without key rebinding)
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _zacrs_line_pre_redraw
