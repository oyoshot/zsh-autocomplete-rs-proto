#!/usr/bin/env zsh
# zsh-autocomplete-rs: Rust-powered autocomplete with popup UI
# Usage: source this file in your .zshrc

# === Initialization ===

# Reload changes our implementation, never another plugin's saved widget.
# Old adapters used mutable delegate storage and foreign-alias repair. Mixing
# the two lifecycles in a live shell is unsafe; start a new shell to upgrade.
if (( ! ${+_zacrs_integration_version} && ${+_zacrs_self_insert_wrapped} )); then
    print -u2 -- 'zacrs: restart the shell to load the new widget lifecycle'
    return 1
fi
typeset -g _zacrs_integration_version=1

ZACRS_BIN="${ZACRS_BIN:-zsh-autocomplete-rs}"

# Packages keep the helpers beside this entry point, with optional adjacent .zwc
# files. Zsh selects compiled code when sourcing each file.
_zacrs_dir="${0:A:h}"
source "${_zacrs_dir}/_zacrs_util.zsh"
source "${_zacrs_dir}/_zacrs_gather.zsh"
source "${_zacrs_dir}/_zacrs_compsys.zsh"
source "${_zacrs_dir}/_zacrs_protocol.zsh"

# Internal state
zmodload zsh/zleparameter 2>/dev/null

_zacrs_restore_self_insert() {
    (( ! ${_zacrs_space_enabled:-0} && ${_zacrs_self_insert_wrapped:-0} && ! ${_zacrs_insert_shared:-0} )) || return 0
    [[ ${widgets[self-insert]:-} == user:_zacrs_insert ]] || return 0
    # Only retire an attachment with no other registered references. A plugin
    # may save the entry using either zle -A or zle -N. Keep its delegate alive;
    # never rewrite the saved alias, even when our feature is now disabled.
    local -a references
    references=( ${(k)widgets[(R)user:_zacrs_insert]} )
    if (( ${#references} != 1 )); then
        _zacrs_insert_shared=1
        return 0
    fi
    zle -A ._zacrs_insert_delegate self-insert || return
    _zacrs_insert_restored=${widgets[self-insert]}
    _zacrs_self_insert_wrapped=0
}

typeset -g _zacrs_prev_lbuffer="" _zacrs_prev_buffer=""
typeset -gi _zacrs_prev_cursor=0
typeset -g _zacrs_prev_home="${HOME:-}" _zacrs_prev_cwd="$PWD"
typeset -g _zacrs_prev_file_options=""
typeset -gi _zacrs_prev_primary_editor=0
typeset -g _zacrs_suppressed=0
typeset -g _zacrs_popup_visible=0
typeset -g _zacrs_popup_row=0
typeset -g _zacrs_popup_height=0
typeset -g _zacrs_popup_cursor_row=0
typeset -gi _zacrs_last_render_cursor_row=0
typeset -gi _zacrs_last_render_cursor_col=0
typeset -gi _zacrs_chain_retry=0
typeset -g _zacrs_daemon_available=0
typeset -gi _zacrs_daemon_filesystem=0
typeset -g _zacrs_daemon_started=0
typeset -g _zacrs_daemon_next_retry=0
typeset -g _zacrs_socket_path=""
typeset -g _zacrs_popup_snapshot_lbuffer="" _zacrs_popup_snapshot_buffer=""
typeset -gi _zacrs_popup_snapshot_cursor=0
typeset -gi _zacrs_popup_snapshot_prefix_len=0
typeset -gi _zacrs_popup_snapshot_columns=0
typeset -gi _zacrs_popup_snapshot_lines=0
typeset -gi _zacrs_popup_session_active=0
typeset -gi _zacrs_ignore_next_winch=0
typeset -gi _zacrs_redraw_deferred=0
typeset -gi _zacrs_self_insert_wrapped=${_zacrs_self_insert_wrapped:-0}
# Preserve initialization across reloads so a selected command stays protected.
# Older adapters lack this flag but declare _zacrs_enter_enabled. Its presence
# identifies a reload with a potentially protected session, even when disabled
# or unavailable. Capture it before declaring the capability state below.
typeset -gi _zacrs_enter_session_initialized=${_zacrs_enter_session_initialized:-${+_zacrs_enter_enabled}}
# Shared Enter/Space transport: 0: disabled; 1: ready; -1: invalid response;
# -2: transport unavailable. Feature opt-ins are kept separately below.
# Only a fresh editing line may recover transport; invalid responses need reload.
typeset -gi _zacrs_edit_transport_state=0
typeset -gi _zacrs_enter_expansion=0
typeset -gi _zacrs_space_enabled=0
# Preserve the counter across plugin reloads so old daemon entries cannot
# collide with a newly sourced copy in the same shell process.
typeset -gi _zacrs_filesystem_requested=0
typeset -gi _zacrs_candidate_cache_generation=${_zacrs_candidate_cache_generation:-0}

_zacrs_invalidate_candidate_cache() {
    (( ++_zacrs_candidate_cache_generation ))
}

# Render header parse results (set by _zacrs_parse_render_header)
typeset -gi _zacrs_parsed_tty_len=0
# Daemon send helper (set by _zacrs_daemon_send_render on OK)
typeset -gi _zacrs_send_render_fd=0

_zacrs_reset_popup_snapshot() {
    _zacrs_popup_snapshot_cwd=""
    _zacrs_popup_snapshot_home=""
    _zacrs_popup_snapshot_file_options=""
    _zacrs_popup_snapshot_daemon=0
    _zacrs_popup_snapshot_buffer=""
    _zacrs_popup_snapshot_cursor=0
    _zacrs_popup_snapshot_lbuffer=""
    _zacrs_popup_snapshot_prefix_len=0
    _zacrs_popup_snapshot_columns=0
    _zacrs_popup_snapshot_lines=0
}

_zacrs_reset_cache() {
    _zacrs_chain_retry=0
    _zacrs_redraw_deferred=0
    _zacrs_restore_self_insert
}

# Only enabled shells send observations; the daemon owns the editing session.
_zacrs_reset_enter_selection() {
    _zacrs_command_editor || return 0
    local _zacrs_abbreviation_header _zacrs_abbreviation_apply
    if (( _zacrs_edit_transport_state > 0 )) && _zacrs_abbreviation_request reset; then
        _zacrs_enter_session_initialized=1
    fi
    if (( _zacrs_edit_transport_state == -2 )); then
        # A new line has no selection to preserve. Reconnect here, never in the
        # middle of accepting or applying a user's existing input.
        _zacrs_edit_transport_state=0
        _zacrs_daemon_available=0
        if _zacrs_initialize_daemon; then
            if (( _zacrs_edit_transport_state > 0 )) && _zacrs_abbreviation_request reset; then
                _zacrs_enter_session_initialized=1
            fi
        else
            _zacrs_edit_transport_state=-2
        fi
    fi
    return 0
}

_zacrs_record_enter_selection() {
    _zacrs_command_editor || return 0
    local _zacrs_abbreviation_header _zacrs_abbreviation_apply
    (( _zacrs_edit_transport_state > 0 )) || return 0
    local -a reply _zacrs_abbr_words
    _zacrs_abbreviation_words
    _zacrs_abbreviation_request select "${1:-0}" "${2:-1}" "$reply[1]" "$reply[2]" "$reply[3]" || true
    return 0
}

# Both features borrow one insertion widget and one delegate. Clearing a
# deferred redraw must not remove the wrapper needed for Space expansion.
_zacrs_wrap_self_insert() {
    (( _zacrs_self_insert_wrapped )) && return 0
    # A changed public user widget is an explicit replacement, not a request
    # to repair its chain. Builtins can be captured exactly without interpreting
    # their identity; no live entry references remain after our own retirement.
    if (( ${+_zacrs_insert_restored} )) && \
        [[ ${widgets[self-insert]:-} != "$_zacrs_insert_restored" ]]; then
        return 1
    fi
    # Dot names are internal delegates, not new canonical builtin operations.
    # Preserve native aliases as well as user/completion widgets via ZLE.
    _zacrs_insert_shared=0
    zle -A self-insert ._zacrs_insert_delegate || return
    zle -N self-insert _zacrs_insert || return
    _zacrs_self_insert_wrapped=1
}
_zacrs_defer_redraw() {
    _zacrs_redraw_deferred=1
    _zacrs_wrap_self_insert
}

_zacrs_end_popup_session() {
    _zacrs_popup_session_active=0
    _zacrs_ignore_next_winch=0
}

_zacrs_record_popup_snapshot() {
    _zacrs_filesystem_editor || { _zacrs_reset_popup_snapshot; return 0; }
    local prefix_len="$1"
    _zacrs_popup_snapshot_daemon=${2:-1}
    _zacrs_popup_snapshot_cwd="$PWD"
    _zacrs_popup_snapshot_home="${HOME:-}"
    local _zacrs_file_option_key
    _zacrs_file_options
    _zacrs_popup_snapshot_file_options="$_zacrs_file_option_key"
    _zacrs_popup_snapshot_buffer="$BUFFER"
    _zacrs_popup_snapshot_cursor=$CURSOR
    _zacrs_popup_snapshot_lbuffer="$LBUFFER"
    _zacrs_popup_snapshot_prefix_len=$prefix_len
    _zacrs_popup_snapshot_columns=$COLUMNS
    _zacrs_popup_snapshot_lines=$LINES
}

_zacrs_popup_snapshot_matches_editor() {
    _zacrs_filesystem_editor &&
        [[ "$_zacrs_popup_snapshot_buffer" == "$BUFFER" && $_zacrs_popup_snapshot_cursor == $CURSOR ]] &&
        _zacrs_file_context_matches "$_zacrs_popup_snapshot_cwd" "$_zacrs_popup_snapshot_home" "$_zacrs_popup_snapshot_file_options" &&
        _zacrs_pwd_matches_cwd
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
        _zacrs_daemon_filesystem=0
        _zacrs_daemon_next_retry=$(( SECONDS + 1 ))
    }

    _zacrs_read_filesystem_capability() {
        _zacrs_daemon_filesystem=0
        [[ " $1 " == *' filesystem=2 '* ]] && _zacrs_daemon_filesystem=1
        return 0
    }

    _zacrs_ping_daemon() {
        local fd resp request
        # Older daemons close the connection for an unknown request. Confirm
        # liveness with the original ping before trying to start another daemon.
        for request in 'ping shell' ping; do
            zsocket "$_zacrs_socket_path" 2>/dev/null || return 1
            fd=$REPLY
            print -r -u $fd -- "$request"
            resp=''
            IFS= read -r -t 1 -u $fd resp
            exec {fd}<&-
            if [[ "$resp" == OK* ]]; then
                REPLY="$resp"
                return 0
            fi
        done
        return 1
    }

    _zacrs_ensure_daemon() {
        # Already confirmed available this session
        (( _zacrs_daemon_available )) && return 0
        if _zacrs_ping_daemon; then
            _zacrs_read_filesystem_capability "$REPLY"
            _zacrs_daemon_available=1
            _zacrs_daemon_started=0
            _zacrs_daemon_next_retry=0
            return 0
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
            _zacrs_ping_daemon && _zacrs_read_filesystem_capability "$REPLY"
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
    _zacrs_initialize_daemon() {
        local handshake
        # Startup needs capabilities, not installation identity. Updates stop
        # running daemons explicitly; a compatible running daemon is reused.
        if _zacrs_ping_daemon; then
            handshake="$REPLY"
            _zacrs_daemon_available=1
            _zacrs_daemon_next_retry=0
        fi
        if (( ! _zacrs_daemon_available )); then
            local ensure_status
            # Locking and startup run only for a missing or
            # unavailable daemon. Updates use daemon stop outside shell startup.
            if handshake=$("$ZACRS_BIN" daemon ensure 2>/dev/null); then
                _zacrs_daemon_available=1
                _zacrs_daemon_started=0
                _zacrs_daemon_next_retry=0
            else
                ensure_status=$?
                # Directly sourcing a new plugin with an older CLI still works.
                if (( ensure_status == 2 )) && _zacrs_ensure_daemon; then
                    handshake="$REPLY"
                else
                    _zacrs_mark_daemon_unavailable
                    return 1
                fi
            fi
        fi
        _zacrs_read_filesystem_capability "$handshake"
        if [[ "$handshake" == OK* && " $handshake " == *' edit_context=2 '* ]]; then
            _zacrs_enter_expansion=0
            _zacrs_space_enabled=0
            [[ " $handshake " == *' enter_expansion=1 '* ]] && _zacrs_enter_expansion=1
            [[ " $handshake " == *' space_expansion=1 '* ]] && _zacrs_space_enabled=1
            (( _zacrs_enter_expansion || _zacrs_space_enabled )) && _zacrs_edit_transport_state=1
        fi
        return 0
    }
    _zacrs_initialize_daemon || true
fi

# === Render header parsing (shared by daemon + subprocess paths) ===

# Parse an "OK key=val ... <tty_len>" header into globals.
# Sets: _zacrs_popup_row, _zacrs_popup_height, _zacrs_popup_cursor_row,
#        _zacrs_parsed_tty_len
_zacrs_parse_render_header() {
    local header="$1"
    _zacrs_parsed_tty_len=0
    local token
    for token in ${(s: :)header}; do
        local key="${token%%=*}" val="${token#*=}"
        case "$key" in
            OK)                     ;; # daemon header prefix — skip
            popup_row)              _zacrs_popup_row=$val ;;
            popup_height)           _zacrs_popup_height=$val ;;
            cursor_row)             _zacrs_popup_cursor_row=$val ;;
            tty_len)                _zacrs_parsed_tty_len=$val ;;
        esac
    done
}

# The daemon reads the filter as one line. Preview and Tab both send any other
# filter to the subprocess, which receives it as an argument. Tab uses this
# check; the render path below tests the same pattern inline.
_zacrs_daemon_filter() {
    [[ "$1" != *[$'\r\n']* ]]
}

# Connect to daemon, send a render request, and parse the response header.
# Args: $1=cursor_row $2=cursor_col $3=prefix $4=candidates $5=selected (optional) $6=context_key (optional) $7=command-position (0|1) $8=candidates-present (0|1) $9=command-context (optional)
# When candidates are empty, $8 distinguishes an explicit empty candidate set
# from a cache-only request. Cache-first and resize callers leave $8 unset.
# On OK        (return 0): _zacrs_send_render_fd holds open fd; caller must sysread + close.
# On EMPTY     (return 1): fd already closed.
# On CACHE_MISS (return 3): fd already closed; caller should collect candidates and retry.
# On transport failure or an unsupported filter (return 2): caller can use the subprocess.
_zacrs_daemon_send_render() {
    local _cr="$1" _cc="$2" _pfx="$3" _cands="$4" _sel="${5:-}" _ctx_key="${6:-}" _cmd_pos="${7:-0}" _cands_present="${8:-0}" _command_context="${9:-}"
    [[ "$_pfx" != *[$'\r\n']* ]] || return 2
    local fd
    if ! zsocket "$_zacrs_socket_path" 2>/dev/null; then
        (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
        return 2
    fi
    fd=$REPLY
    local render_cmd="render $_cr $_cc $COLUMNS $LINES"
    [[ -n "$_ctx_key" ]] && render_cmd+=" context_key=$_ctx_key"
    (( _cmd_pos )) && render_cmd+=" command_position=1"
    [[ -n "$_command_context" ]] && render_cmd+=" command_context_hex=$(_zacrs_encode_hex_input "$_command_context")"
    (( _cands_present )) && render_cmd+=" candidates_present=1"
    render_cmd+=" popup_key=$$"
    [[ -n "$_sel" ]] && render_cmd+=" selected=$_sel"
    if (( _cands_present && _zacrs_filesystem_requested )); then
        local flag
        local -a reply
        _zacrs_filesystem_flags
        render_cmd="render-files ${render_cmd#render }"
        for flag in "${reply[@]}"; do
            render_cmd+=" filesystem_$flag=1"
        done
        print -r -u $fd -- "$render_cmd"
        _zacrs_write_framed_fields $fd "$_zacrs_filesystem_base" "$_zacrs_filesystem_lookup" "$_zacrs_filesystem_prefix"
    else
        print -r -u $fd -- "$render_cmd"
    fi
    printf '%s\n' "$_pfx" >&$fd
    if [[ -n "$_cands" ]]; then
        printf '%s\n' "$_cands" >&$fd
    fi
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
    elif [[ "$header" == CACHE_MISS ]]; then
        exec {fd}<&-
        return 3
    else
        exec {fd}<&-
        (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
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
    local prefix="$1" prefix_len="$2" candidates_str="$3" from_gather="${4:-0}" selected="${5:-}" context_key="${6:-}"
    local cursor_row=0 cursor_col=0
    local is_cmd_pos=0
    local command_context=""
    if _zacrs_command_context "$LBUFFER"; then
        command_context="$REPLY"
        is_cmd_pos=$_zacrs_command_context_is_command
    fi
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
    if (( _zacrs_daemon_available && (!_zacrs_filesystem_requested || _zacrs_daemon_filesystem) )); then
        local _prev_vis=$_zacrs_popup_visible _prev_row=$_zacrs_popup_row _prev_height=$_zacrs_popup_height
        _zacrs_daemon_send_render "$cursor_row" "$cursor_col" "$prefix" "$candidates_str" "$selected" "$context_key" "$is_cmd_pos" 1 "$command_context"
        local _send_rc=$?
        if (( _send_rc == 0 )); then
            local fd=$_zacrs_send_render_fd
            local tty_len=$_zacrs_parsed_tty_len
            local tty_ok=1
            _zacrs_daemon_draw_atomic $fd $tty_len $_prev_vis $_prev_row $_prev_height 0 || tty_ok=0
            if (( tty_ok )); then
                _zacrs_popup_visible=1
                _zacrs_last_render_cursor_row=$cursor_row
                _zacrs_last_render_cursor_col=$cursor_col
                _zacrs_record_popup_snapshot "$prefix_len"
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
        # _send_rc == 3: CACHE_MISS — should not happen here (candidates were provided),
        #                fall through to subprocess as a safety measure
    fi

    # Fallback: subprocess (clear stale popup before spawning)
    _zacrs_clear_popup
    local -a render_args
    render_args=(render --prefix "$prefix" --cursor-row "$cursor_row" --cursor-col "$cursor_col")
    (( is_cmd_pos )) && render_args+=(--command-position)
    [[ -n "$command_context" ]] && render_args+=(--command-context "$command_context")
    [[ -n "$selected" ]] && render_args+=(--selected "$selected")
    local -a reply
    _zacrs_filesystem_args
    render_args+=("${reply[@]}")
    local output
    output=$(printf '%s' "$candidates_str" | "$ZACRS_BIN" "${render_args[@]}")
    local exit_code=$?

    if [[ $exit_code -eq 0 && -n "$output" ]]; then
        _zacrs_popup_visible=1
        _zacrs_last_render_cursor_row=$cursor_row
        _zacrs_last_render_cursor_col=$cursor_col
        _zacrs_parse_render_header "$output"
        _zacrs_record_popup_snapshot "$prefix_len" 0
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

# Decode a DONE/APPLY pair into the caller's result variables. A non-zero
# status means the reply must not be applied.
_zacrs_parse_selection_reply() {
    if ! _zacrs_parse_done_line "$1" || ! _zacrs_parse_apply_line "$2"; then
        _zacrs_clear_popup
        return 5
    fi
    (( result_code != 5 ))
}

_zacrs_finish_popup_session() {
    _zacrs_popup_visible=0
    _zacrs_popup_row=0
    _zacrs_popup_height=0
    _zacrs_popup_cursor_row=0
    _zacrs_last_render_cursor_row=0
    _zacrs_last_render_cursor_col=0
    _zacrs_reset_popup_snapshot
}

# Record the lookup context that the next redraw compares against.
_zacrs_record_prev_editor_state() {
    local _zacrs_file_option_key
    _zacrs_file_options
    _zacrs_prev_buffer="$1"
    _zacrs_prev_cursor=$2
    _zacrs_prev_home="${HOME:-}"
    _zacrs_prev_cwd="$PWD"
    _zacrs_prev_file_options="$_zacrs_file_option_key"
}

_zacrs_line_init() {
    # Terminal interrupts bypass send-break (and line-finish). Zsh has already
    # displayed the next prompt, so forget old coordinates without erasing them.
    _zacrs_finish_popup_session
    _zacrs_end_popup_session
    _zacrs_prev_lbuffer=""
    _zacrs_record_prev_editor_state "" 0
    _zacrs_suppressed=0
    _zacrs_reset_cache
}

_zacrs_literal_alias_token() {
    local token="$1" suffix="${1##*.}"
    [[ -n "$token" ]] || return 1
    (( ${+galiases[$token]} || ${+aliases[$token]} )) && return 0
    # Zsh checks suffix aliases against the spelling after the last dot.
    [[ "$token" == ?*.* && -n "$suffix" ]] && (( ${+saliases[$suffix]} ))
}

# Pin the lookup directory in the invoking widget's scope before it blocks.
_zacrs_pin_picker_cwd() {
    local lookup_cwd="$1"
    [[ "$lookup_cwd" == /* && . -ef "$lookup_cwd" ]] || return 0
    local physical_cwd="${lookup_cwd:P}"
    [[ . -ef "$physical_cwd" ]] && _zacrs_picker_cwd="$physical_cwd"
    return 0
}

_zacrs_apply() {
    local prefix_len="$1" result_code="$2" result_text="$3" chain="${4:-0}" execute="${5:-0}" restore_text="${6:-}"
    local selection_applied="${7:-0}"
    if [[ "${literal_path_known:-0}" == 1 && "$result_code" == (0|2) ]]; then
        # The logical pathname may have changed while the picker was open.
        if [[ "$literal_path" != /* ]]; then
            [[ -n "${_zacrs_picker_cwd:-}" && . -ef "$_zacrs_picker_cwd" ]] || return 0
            _zacrs_pwd_matches_cwd || return 0
        fi
        result_text="${(q)literal_path}${literal_suffix}"
        if (( ${#galiases} || ${#aliases} || ${#saliases} )); then
            # A directory suffix can be part of the alias token.
            local -a literal_words=( "${(@z)result_text}" )
            if _zacrs_literal_alias_token "${literal_words[1]:-}"; then
                result_text="${(qq)literal_path}${literal_suffix}"
                literal_words=( "${(@z)result_text}" )
                # Zsh also allows aliases explicitly named with quote characters.
                if _zacrs_literal_alias_token "${literal_words[1]:-}"; then
                    return 0
                fi
            fi
        fi
    fi
    local base
    local new_lbuffer="$LBUFFER"
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi

    # Passthrough keeps POSTDISPLAY so zsh-autosuggestions can still accept
    # the suggestion when the re-injected key fires.
    [[ $result_code -ne 3 ]] && unset POSTDISPLAY

    case "$result_code" in
        0|1|2)
            if [[ -n "$result_text" ]]; then
                new_lbuffer="${base}${result_text}"
            fi
            _zacrs_suppressed=0
            ;;
        3)
            if [[ -n "$restore_text" ]]; then
                new_lbuffer="${base}${restore_text}"
            fi
            _zacrs_suppressed=0
            _zacrs_decode_hex_to_REPLY "$result_text"
            [[ -n "$REPLY" ]] && zle -U "$REPLY"
            ;;
    esac

    BUFFER="${new_lbuffer}${RBUFFER}"
    _zacrs_record_prev_editor_state "$BUFFER" ${#new_lbuffer}
    if (( _zacrs_edit_transport_state > 0 )) && [[ "$result_code" == (0|2) && -n "$result_text" ]]; then
        _zacrs_record_enter_selection ${#base} "$selection_applied"
    fi
    if [[ -n "$cursor_offset" && "$cursor_offset" == <-> ]]; then
        CURSOR=$((${#base} + cursor_offset))
    else
        CURSOR=${#new_lbuffer}
    fi

    if (( execute )) && [[ $result_code -eq 0 ]]; then
        _zacrs_prev_lbuffer="$new_lbuffer"
        _zacrs_chain_retry=0
        zle reset-prompt
        zle accept-line
        return
    fi

    if (( chain )); then
        _zacrs_prev_lbuffer="$base"
        _zacrs_chain_retry=1
    else
        _zacrs_prev_lbuffer="$new_lbuffer"
        _zacrs_chain_retry=0
    fi
}

_zacrs_invoke_daemon() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}" reuse_visible="${6:-0}" context_key="${7:-}" accept_single="${8:-0}" candidates_present="${9:-0}"
    local shift_tab_hex=""
    local is_cmd_pos=0
    local command_context=""
    if _zacrs_command_context "$LBUFFER"; then
        command_context="$REPLY"
        is_cmd_pos=$_zacrs_command_context_is_command
    fi
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
    fi

    local stale_hex=""
    [[ -n "$_zacrs_cursor_stale" ]] && stale_hex="$(_zacrs_encode_hex_input "$_zacrs_cursor_stale")"
    [[ -n "$terminfo[kcbt]" ]] && shift_tab_hex="$(_zacrs_encode_hex_input "$terminfo[kcbt]")"
    local -a complete_args
    complete_args=(
        complete
        --daemon
        --prefix "$prefix"
        --cursor-row "$cursor_row"
        --cursor-col "$cursor_col"
        --cols "$COLUMNS"
        --rows "$LINES"
        --popup-key "$$"
    )
    (( is_cmd_pos )) && complete_args+=(--command-position)
    [[ -n "$command_context" ]] && complete_args+=(--command-context "$command_context")
    (( accept_single )) && complete_args+=(--accept-single)
    (( candidates_present )) && complete_args+=(--candidates-present)
    [[ -n "$shift_tab_hex" ]] && complete_args+=(--shift-tab-hex "$shift_tab_hex")
    [[ -n "$stale_hex" ]] && complete_args+=(--stale-hex "$stale_hex")
    (( _zacrs_popup_visible )) && complete_args+=(--prev-popup-row "$_zacrs_popup_row" --prev-popup-height "$_zacrs_popup_height")
    (( reuse_visible )) && complete_args+=(--reuse-token 1)
    [[ -n "$context_key" ]] && complete_args+=(--context-key "$context_key")

    local -a reply
    reply=()
    (( candidates_present )) && _zacrs_filesystem_args
    complete_args+=("${reply[@]}")
    local _zacrs_picker_cwd=""
    if (( reuse_visible )); then
        _zacrs_pin_picker_cwd "${_zacrs_popup_snapshot_cwd:-}"
    elif (( candidates_present && _zacrs_filesystem_requested )); then
        _zacrs_pin_picker_cwd "$_zacrs_filesystem_cwd"
    elif (( ! candidates_present )); then
        _zacrs_pin_picker_cwd "$PWD"
    fi
    local output
    _zacrs_popup_session_active=1
    _zacrs_ignore_next_winch=1
    output=$(printf '%s\nEND\n' "$candidates_str" | "$ZACRS_BIN" "${complete_args[@]}" 2>/dev/null) || {
        _zacrs_end_popup_session
        (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        return 1
    }
    _zacrs_end_popup_session
    _zacrs_cursor_stale=""

    local -a lines
    lines=("${(@f)output}")
    if [[ "${lines[1]:-}" == "CACHE_MISS" ]]; then
        return 2
    fi
    if [[ "${lines[1]:-}" != DONE* || "${lines[2]:-}" != APPLY* ]]; then
        (( ${+functions[_zacrs_mark_daemon_unavailable]} )) && _zacrs_mark_daemon_unavailable
        return 5
    fi

    local literal_path_known=0 literal_path="" literal_suffix=""
    local result_code result_text chain=0 execute=0 restore_text="" cursor_offset="" selection_applied=0 insert_trigger=0
    _zacrs_parse_selection_reply "${lines[1]:-}" "${lines[2]:-}" || return 5
    (( result_code == 4 )) && return 4
    _zacrs_finish_popup_session
    _zacrs_apply "$prefix_len" "$result_code" "$result_text" "$chain" "$execute" "$restore_text" "$selection_applied"
    (( accept_single || result_code != 0 )) && zle reset-prompt
    return 0
}

# === Core: invoke Rust binary via coproc (blocking, for Tab) ===

_zacrs_invoke() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}" accept_single="${6:-0}"
    local shift_tab_hex=""
    local is_cmd_pos=0
    local command_context=""
    if _zacrs_command_context "$LBUFFER"; then
        command_context="$REPLY"
        is_cmd_pos=$_zacrs_command_context_is_command
    fi
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
    fi
    local stale_hex=""
    [[ -n "$_zacrs_cursor_stale" ]] && stale_hex="$(_zacrs_encode_hex_input "$_zacrs_cursor_stale")"
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
    (( is_cmd_pos )) && complete_args+=(--command-position)
    [[ -n "$command_context" ]] && complete_args+=(--command-context "$command_context")
    (( accept_single )) && complete_args+=(--accept-single)
    [[ -n "$shift_tab_hex" ]] && complete_args+=(--shift-tab-hex "$shift_tab_hex")
    [[ -n "$stale_hex" ]] && complete_args+=(--stale-hex "$stale_hex")
    if (( _zacrs_popup_visible )); then
        complete_args+=(--prev-popup-row "$_zacrs_popup_row" --prev-popup-height "$_zacrs_popup_height")
    fi

    local -a reply
    _zacrs_filesystem_args
    complete_args+=("${reply[@]}")
    local _zacrs_picker_cwd=""
    (( _zacrs_filesystem_requested )) && _zacrs_pin_picker_cwd "$_zacrs_filesystem_cwd"
    local output
    _zacrs_popup_session_active=1
    _zacrs_ignore_next_winch=1
    output=$(printf '%s\nEND\n' "$candidates_str" | "$ZACRS_BIN" "${complete_args[@]}" 2>/dev/null) || {
        _zacrs_end_popup_session
        [[ -n "$_zacrs_cursor_stale" ]] && zle -U "$_zacrs_cursor_stale"
        _zacrs_cursor_stale=""
        return 1
    }
    _zacrs_end_popup_session
    _zacrs_cursor_stale=""

    local -a lines
    lines=("${(@f)output}")
    if [[ "${lines[1]:-}" != DONE* || "${lines[2]:-}" != APPLY* ]]; then
        return 5
    fi

    local literal_path_known=0 literal_path="" literal_suffix=""
    local result_code result_text chain=0 execute=0 restore_text="" cursor_offset="" selection_applied=0 insert_trigger=0
    _zacrs_parse_selection_reply "${lines[1]:-}" "${lines[2]:-}" || return 5
    (( result_code == 4 )) && return 1
    _zacrs_finish_popup_session
    _zacrs_apply "$prefix_len" "$result_code" "$result_text" "$chain" "$execute" "$restore_text" "$selection_applied"
    (( accept_single || result_code != 0 )) && zle reset-prompt
}

_zacrs_encode_hex_input() {
    local input="$1"
    [[ -z "$input" ]] && return 0
    print -rn -- "$input" | od -An -tx1 -v | tr -d ' \n'
}

# === Shared completion helpers ===

# Choose the fallback source once native completion has produced nothing.
# Writes the caller's prefix, prefix_len and candidates_str, and expects the
# caller to have reset _zacrs_filesystem_requested. Returns 0 when the shell
# command tables supplied the candidates.
_zacrs_fallback_candidates() {
    if _zacrs_filesystem_query_current; then
        _zacrs_filesystem_requested=1
        prefix="$_zacrs_filesystem_prefix"
        prefix_len=${#_zacrs_filesystem_prefix}
        return 1
    fi
    candidates_str="$(_zacrs_gather "$LBUFFER")"
    [[ -n "$candidates_str" ]] || return 1
    prefix="${LBUFFER##* }"
    prefix_len=${#prefix}
}

# Collect candidates: compsys → gather.
# Caller must declare: local prefix prefix_len candidates_str
# (zsh dynamic scoping lets us write to these from here)
_zacrs_collect_candidates() {
    candidates_str=""
    _zacrs_filesystem_requested=0
    _zacrs_filesystem_valid=0
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
    [[ -n "$candidates_str" ]] || _zacrs_fallback_candidates || :
    if [[ -z "$candidates_str" && ${#prefix} -ge 3 && "$prefix" != */* ]] \
        && _zacrs_is_cmd_pos "$LBUFFER" "$prefix"; then
        candidates_str="$(_zacrs_gather_command_rescue "$prefix")"
    fi
}

# === Popup completion widget ===

_zacrs_complete_popup() {
    local prefix="" prefix_len=0 candidates_str=""
    local cursor_row="" cursor_col=""
    local reuse_visible=0 context_key=""
    local naive_prefix="${LBUFFER##* }"

    _zacrs_filesystem_requested=0
    prefix="$naive_prefix"
    prefix_len=${#naive_prefix}

    # A reuse request carries the naive prefix as its filter.
    if (( _zacrs_daemon_available && _zacrs_popup_visible && _zacrs_popup_snapshot_daemon )) && _zacrs_popup_snapshot_matches_editor \
        && (( _zacrs_popup_snapshot_columns == COLUMNS )) \
        && (( _zacrs_popup_snapshot_lines == LINES )) && _zacrs_daemon_filter "$naive_prefix"; then
        reuse_visible=1
        prefix_len=$_zacrs_popup_snapshot_prefix_len
        cursor_row=$_zacrs_popup_cursor_row
        cursor_col=$_zacrs_last_render_cursor_col
    fi

    if (( ! reuse_visible )); then
        _zacrs_collect_candidates
    fi

    _zacrs_suppressed=0

    # Use the subprocess for file requests when the daemon serves only older
    # protocols, and for filters the daemon cannot carry.
    if (( _zacrs_daemon_available && (!_zacrs_filesystem_requested || _zacrs_daemon_filesystem) )) && _zacrs_daemon_filter "$prefix"; then
        _zacrs_current_context_key
        context_key="$REPLY"
        _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" \
            "$cursor_row" "$cursor_col" "$reuse_visible" "$context_key" 1 "$(( ! reuse_visible ))"
        local daemon_rc=$?
        if (( daemon_rc == 0 )); then
            return
        elif (( daemon_rc == 5 )); then
            _zacrs_clear_popup
            return
        elif (( daemon_rc == 4 )); then
            _zacrs_clear_popup
            zle expand-or-complete
            return
        elif (( daemon_rc == 2 )); then
            reuse_visible=0
            _zacrs_collect_candidates
            if (( !_zacrs_filesystem_requested || _zacrs_daemon_filesystem )) && _zacrs_daemon_filter "$prefix"; then
                _zacrs_current_context_key
                context_key="$REPLY"
                _zacrs_invoke_daemon "$prefix" "$prefix_len" "$candidates_str" \
                    "$cursor_row" "$cursor_col" "$reuse_visible" "$context_key" 1 1
                local retry_rc=$?
                (( retry_rc == 0 )) && return
                if (( retry_rc == 5 )); then
                    _zacrs_clear_popup
                    return
                fi
            fi
        fi
    fi
    if [[ -z "$candidates_str" ]] && (( ! _zacrs_filesystem_requested )); then
        reuse_visible=0
        _zacrs_collect_candidates
    fi
    if (( ! reuse_visible )); then
        _zacrs_clear_popup
    fi
    _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str" "$cursor_row" "$cursor_col" 1 || {
        local _invoke_rc=$?
        _zacrs_clear_popup
        (( _invoke_rc == 5 )) || zle expand-or-complete
    }
}

# === Auto-trigger via line-pre-redraw hook ===

_zacrs_line_pre_redraw() {
    # A non-insert widget may drain the type-ahead batch. Restore the original
    # self-insert before any unchanged-buffer early return.
    (( _zacrs_self_insert_wrapped && ! _zacrs_space_enabled && PENDING == 0 )) && _zacrs_restore_self_insert

    # Context depends on both sides of the cursor and the current editor.
    local -i primary_editor=0
    local _zacrs_file_option_key
    _zacrs_file_options
    _zacrs_filesystem_editor && primary_editor=1
    if [[ "$LBUFFER" != "$_zacrs_prev_lbuffer" || "$BUFFER" != "$_zacrs_prev_buffer" ||
        $CURSOR != $_zacrs_prev_cursor || "${HOME:-}" != "$_zacrs_prev_home" ||
        "$PWD" != "$_zacrs_prev_cwd" || "$_zacrs_file_option_key" != "$_zacrs_prev_file_options" ||
        $primary_editor != $_zacrs_prev_primary_editor ]]; then
        # Defer popup clear — _zacrs_render will do selective clear to avoid flicker.
        # Just invalidate the snapshot so Tab reuse check uses fresh data.
        _zacrs_reset_popup_snapshot
    else
        return
    fi
    # Type-ahead detected: skip heavy work.  Do NOT update
    # _zacrs_prev_lbuffer so the next redraw retries this buffer.
    if (( PENDING > 0 )); then
        _zacrs_clear_popup
        # ZLE may consume the rest of a type-ahead batch without invoking the
        # redraw hook again. The self-insert wrapper resumes on its final key.
        _zacrs_defer_redraw
        return
    fi

    _zacrs_redraw_deferred=0

    _zacrs_prev_lbuffer="$LBUFFER"
    _zacrs_prev_buffer="$BUFFER"
    _zacrs_prev_cursor=$CURSOR
    _zacrs_prev_primary_editor=$primary_editor
    _zacrs_prev_home="${HOME:-}"
    _zacrs_prev_cwd="$PWD"
    _zacrs_prev_file_options="$_zacrs_file_option_key"

    # 空 or 空白のみ → コマンド未入力なのでスキップ
    [[ ! "$LBUFFER" =~ [^[:space:]] ]] && { _zacrs_reset_cache; _zacrs_clear_popup; return }

    # DismissWithSpace 後の抑制: 非空 prefix 入力で解除 (naive prefix で十分)
    local naive_prefix="${LBUFFER##* }"
    if (( _zacrs_suppressed )); then
        if [[ -n "$naive_prefix" ]]; then
            _zacrs_suppressed=0
        else
            _zacrs_clear_popup; return
        fi
    fi

    # context_key は引数位置で設定する。ZLE 編集セッションの世代により
    # コマンド実行後の外部状態変化を古い候補キャッシュへ持ち越さない。
    local context_key="" context_home="${HOME:-}" context_cwd="$PWD"
    local context_file_options="$_zacrs_file_option_key"

    # 候補収集変数
    local candidates_str="" from_gather=0
    local prefix prefix_len
    prefix="$naive_prefix"
    prefix_len=${#naive_prefix}
    local command_context="" is_cmd_pos=0
    if _zacrs_command_context "$LBUFFER"; then
        command_context="$REPLY"
        is_cmd_pos=$_zacrs_command_context_is_command
    fi

    if (( !_zacrs_daemon_available )) && (( ${+functions[_zacrs_maybe_retry_daemon]} )); then
        _zacrs_maybe_retry_daemon
    fi
    if (( _zacrs_daemon_available )); then
        _zacrs_current_context_key
        context_key="$REPLY"
    fi

    # Cache-first: デーモンにキャッシュのみで render を試みる（引数位置のみ）。
    # 非空 prefix の再利用可否は daemon 側の prefix-aware cache が判定する。
    if (( _zacrs_daemon_available )) && [[ -n "$context_key" ]]; then
        local cursor_row=0 cursor_col=0
        if (( _zacrs_popup_visible
                && _zacrs_last_render_cursor_row > 0
                && COLUMNS == _zacrs_popup_snapshot_columns
                && LINES == _zacrs_popup_snapshot_lines )); then
            cursor_row=$_zacrs_last_render_cursor_row
            cursor_col=$_zacrs_last_render_cursor_col
        else
            _zacrs_get_cursor_pos
            _zacrs_cursor_stale=""
        fi
        local _prev_vis=$_zacrs_popup_visible _prev_row=$_zacrs_popup_row _prev_height=$_zacrs_popup_height
        # candidates 引数を空にして送信 → デーモンがキャッシュを探す
        _zacrs_daemon_send_render "$cursor_row" "$cursor_col" "$naive_prefix" "" "" "$context_key" "$is_cmd_pos" 0 "$command_context"
        local _cache_rc=$?
        if (( _cache_rc == 0 )); then
            # キャッシュヒット: 描画完了
            local fd=$_zacrs_send_render_fd
            local tty_len=$_zacrs_parsed_tty_len
            local tty_ok=1
            _zacrs_daemon_draw_atomic $fd $tty_len $_prev_vis $_prev_row $_prev_height 0 || tty_ok=0
            if (( tty_ok )); then
                _zacrs_popup_visible=1
                _zacrs_last_render_cursor_row=$cursor_row
                _zacrs_last_render_cursor_col=$cursor_col
                _zacrs_record_popup_snapshot "$prefix_len"
            else
                _zacrs_clear_popup
                _zacrs_mark_daemon_unavailable
            fi
            exec {fd}<&-
            return
        elif (( _cache_rc == 1 )); then
            # EMPTY: 現在のキャッシュでは候補なし。
            # 外部状態変化でキャッシュが古い可能性があるため、
            # heavy path にフォールスルーして候補を再収集する。
            :
        fi
        # _cache_rc == 1: EMPTY      → heavy path (compsys + gather) へ
        # _cache_rc == 3: CACHE_MISS → heavy path (compsys + gather) へ
        # _cache_rc == 2: daemon unavailable → heavy path へ
    fi

    _zacrs_filesystem_requested=0
    _zacrs_filesystem_valid=0
    # Heavy path: compsys + gather (no cache available). The PENDING guards
    # above and below are the debounce: unlike a timestamp-only delay, they
    # guarantee that the final keypress still gets a render pass.
    _zacrs_captured=()
    local _zacrs_fd2
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
    if [[ -z "$candidates_str" ]] && _zacrs_fallback_candidates; then
        from_gather=1
    fi
    if [[ -z "$candidates_str" && ${#prefix} -ge 3 && "$prefix" != */* ]] \
        && _zacrs_is_cmd_pos "$LBUFFER" "$prefix"; then
        candidates_str="$(_zacrs_gather_command_rescue "$prefix")"
        from_gather=1
    fi

    _zacrs_should_render_candidates "$candidates_str" "$LBUFFER" "$prefix" \
        || { _zacrs_clear_popup; return }

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )
    if [[ -n "$candidates_str" && ${#cands[@]} -eq 0 ]]; then
        _zacrs_clear_popup
        return
    fi

    # Type-ahead arrived during candidate gathering: skip render.
    # Reset prev_lbuffer so the next redraw retries this buffer.
    if (( PENDING > 0 )); then
        _zacrs_clear_popup
        _zacrs_prev_lbuffer=""
        _zacrs_defer_redraw
        return
    fi

    # A native retry can establish a valid query under changed lookup settings.
    # Keep the snapshot under the settings actually used.
    if ! _zacrs_file_context_matches "$context_cwd" "$context_home" "$context_file_options"; then
        if (( _zacrs_daemon_available )); then
            _zacrs_current_context_key
            context_key="$REPLY"
        fi
        context_home="${HOME:-}"
        context_cwd="$PWD"
        _zacrs_file_options
        context_file_options="$_zacrs_file_option_key"
        if (( _zacrs_filesystem_requested )); then
            _zacrs_prev_home="$context_home"
            _zacrs_prev_cwd="$context_cwd"
            _zacrs_prev_file_options="$context_file_options"
        fi
    fi
    # 候補あり: デーモンのキャッシュを更新しながら render
    _zacrs_render "$prefix" "$prefix_len" "$candidates_str" "$from_gather" "" "$context_key"
}

# === Widget wrappers: Enter/Ctrl-C でポップアップクリア ===

_zacrs_accept_line() {
    _zacrs_clear_popup
    local _keep_editing=0
    if (( _zacrs_edit_transport_state > 0 && _zacrs_enter_expansion )) && _zacrs_expand_command_abbreviation; then
        if [[ "$REPLY" == 0 ]]; then
            _keep_editing=1
            _zacrs_record_enter_selection
        fi
    fi
    _zacrs_prev_lbuffer="$LBUFFER"
    _zacrs_record_prev_editor_state "$BUFFER" $CURSOR
    _zacrs_chain_retry=0
    _zacrs_reset_cache
    zle reset-prompt
    if (( _keep_editing )); then
        zle -R
        return
    fi
    zle .accept-line
}
(( ${_zacrs_widgets_initialized:-0} )) || zle -N accept-line _zacrs_accept_line

_zacrs_send_break() {
    _zacrs_clear_popup
    _zacrs_prev_lbuffer=""
    _zacrs_reset_cache
    zle .send-break
}
(( ${_zacrs_widgets_initialized:-0} )) || zle -N send-break _zacrs_send_break

# === ターミナルリサイズ対応 ===

TRAPWINCH() {
    if (( _zacrs_popup_session_active )); then
        return 0
    fi
    if (( _zacrs_ignore_next_winch )); then
        _zacrs_ignore_next_winch=0
        return 0
    fi
    # Only attempt a reuse-based redraw when the terminal width is unchanged.
    # A width change reflows the prompt and input line, making the cached
    # cursor_row/cursor_col stale; in that case fall through to clear+reset
    # and let the next ZLE event render a fresh popup with correct geometry.
    if (( _zacrs_popup_visible && _zacrs_daemon_available && _zacrs_popup_snapshot_daemon )) && _zacrs_popup_snapshot_matches_editor \
        && (( _zacrs_popup_snapshot_columns == COLUMNS )); then
        local _resize_prefix="${LBUFFER##* }" _resize_cmd_pos=0 _resize_context=""
        if _zacrs_command_context "$LBUFFER"; then
            _resize_context="$REPLY"
            _resize_cmd_pos=$_zacrs_command_context_is_command
        fi
        _zacrs_daemon_send_render "$_zacrs_popup_cursor_row" "$_zacrs_last_render_cursor_col" "" "" "" "" "$_resize_cmd_pos" 0 "$_resize_context"
        if (( $? == 0 )); then
            local fd=$_zacrs_send_render_fd
            local tty_len=$_zacrs_parsed_tty_len
            local tty_ok=1
            _zacrs_daemon_draw_atomic $fd $tty_len 1 $_zacrs_popup_row $_zacrs_popup_height 0 || tty_ok=0
            exec {fd}<&-
            if (( tty_ok )); then
                _zacrs_popup_visible=1
                _zacrs_last_render_cursor_row=$_zacrs_popup_cursor_row
                _zacrs_popup_snapshot_columns=$COLUMNS
                _zacrs_popup_snapshot_lines=$LINES
                zle -R
                return 0
            fi
        fi
    fi
    _zacrs_clear_popup
    _zacrs_reset_cache
}

# === Register widgets and keybindings ===

# Keep native self-insert bindings so Zsh search readers retain native input.
# Space expansion and deferred rendering share this wrapper and its lifecycle.
_zacrs_self_insert() {
    if [[ "$KEYS" == ' ' ]] && \
        (( ${NUMERIC:-1} == 1 && ! ${SUFFIX_ACTIVE:-0} && _zacrs_space_enabled && _zacrs_edit_transport_state > 0 )) && \
        _zacrs_expand_command_abbreviation space; then
        # A leading cursor marker leaves LBUFFER equal to the empty redraw
        # sentinel. Clear old UI now, even if the next redraw skips work.
        _zacrs_clear_popup
        local insert_status=0
        (( REPLY )) && { zle ._zacrs_insert_delegate -w || insert_status=$?; }
        _zacrs_record_enter_selection
        _zacrs_prev_lbuffer=""
        _zacrs_reset_cache
        zle -R
        return $insert_status
    fi
    _zacrs_deferred_self_insert
}

# Disabled Space expansion keeps the original typeahead path free of Space checks.
_zacrs_deferred_self_insert() {
    local insert_status=0
    zle ._zacrs_insert_delegate -w || insert_status=$?
    if (( _zacrs_redraw_deferred && PENDING == 0 )); then
        _zacrs_redraw_deferred=0
        _zacrs_restore_self_insert
        _zacrs_prev_lbuffer=""
        _zacrs_line_pre_redraw
    fi
    return $insert_status
}
# Saved user widgets retain this function name across reloads and toggles.
# Disabled shells use the batching body without Space checks.
if (( _zacrs_space_enabled )); then
    functions[_zacrs_insert]=${functions[_zacrs_self_insert]}
    _zacrs_wrap_self_insert
else
    functions[_zacrs_insert]=${functions[_zacrs_deferred_self_insert]}
    _zacrs_restore_self_insert
fi

# Popup widget
zle -N _zacrs_complete_popup

# Default: Tab enters the Rust-owned popup session.
(( ${_zacrs_widgets_initialized:-0} )) || bindkey '^I' _zacrs_complete_popup
typeset -gi _zacrs_widgets_initialized=1

# Register fixed callbacks once. Re-registering a wrapped standard dispatcher
# can capture that same dispatcher into its own callback list and create a cycle.
# Reload only updates our functions, never dispatcher bodies or foreign aliases.
autoload -Uz add-zle-hook-widget
_zacrs_add_zle_hook() {
    local hook="zle-$1" fn="$2"
    [[ ${_zacrs_registered_hooks[$hook]:-} == 1 ]] && return 0
    if [[ ${widgets[$hook]:-} != user:azhw:$hook &&
          -n ${(k)widgets[(R)user:azhw:$hook]} ]]; then
        # Append our callback to the existing standard registry without asking
        # the helper to recapture the outer widget. The helper checks public
        # identity before duplicate membership and could create a cycle here.
        # Preserve every existing entry and the public widget verbatim.
        local -a entries
        local entry
        local -i index=0
        zstyle -a "$hook" widgets entries || entries=()
        if [[ -z ${(M)entries:#(<->:|)$fn} ]]; then
            for entry in "${entries[@]}"; do
                if [[ $entry == <->:* ]] && (( ${entry%%:*} > index )); then
                    index=${entry%%:*}
                fi
            done
            zle -N "$fn" || return
            zstyle "$hook" widgets "${entries[@]}" "$(( index + 1 )):$fn"
        fi
    else
        add-zle-hook-widget "$1" "$fn" || return
    fi
    _zacrs_registered_hooks[$hook]=1
}
typeset -gA _zacrs_registered_hooks
_zacrs_editor_line_init() {
    _zacrs_line_init
    (( _zacrs_edit_transport_state != 0 )) && _zacrs_reset_enter_selection
    return 0
}
zle -D _zacrs_abbreviation_context 2>/dev/null || true
_zacrs_add_zle_hook line-init _zacrs_editor_line_init
_zacrs_add_zle_hook line-pre-redraw _zacrs_line_pre_redraw
# A loader may run after line-init. Preserve any existing selected command.
if (( _zacrs_edit_transport_state > 0 && ! _zacrs_enter_session_initialized )) && zle; then
    _zacrs_reset_enter_selection
fi

# Invalidate daemon candidates before commands accepted by any ZLE widget.
autoload -Uz add-zsh-hook
add-zsh-hook preexec _zacrs_invalidate_candidate_cache
