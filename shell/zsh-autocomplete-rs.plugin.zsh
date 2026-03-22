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
typeset -gi _zacrs_popup_snapshot_columns=0
typeset -gi _zacrs_popup_snapshot_lines=0

_zacrs_reset_popup_snapshot() {
    _zacrs_popup_snapshot_lbuffer=""
    _zacrs_popup_snapshot_prefix=""
    _zacrs_popup_snapshot_prefix_len=0
    _zacrs_popup_snapshot_candidates=""
    _zacrs_popup_snapshot_cursor_row=0
    _zacrs_popup_snapshot_cursor_col=0
    _zacrs_popup_snapshot_reuse_token=""
    _zacrs_popup_snapshot_columns=0
    _zacrs_popup_snapshot_lines=0
}

_zacrs_record_popup_snapshot() {
    local prefix="$1" prefix_len="$2" candidates_str="$3" cursor_col="$4" reuse_token="$5"
    _zacrs_popup_snapshot_lbuffer="$LBUFFER"
    _zacrs_popup_snapshot_prefix="$prefix"
    _zacrs_popup_snapshot_prefix_len=$prefix_len
    _zacrs_popup_snapshot_candidates="$candidates_str"
    _zacrs_popup_snapshot_cursor_row=$_zacrs_popup_cursor_row
    _zacrs_popup_snapshot_cursor_col=$cursor_col
    _zacrs_popup_snapshot_reuse_token="$reuse_token"
    _zacrs_popup_snapshot_columns=$COLUMNS
    _zacrs_popup_snapshot_lines=$LINES
}

# === Daemon lifecycle ===

# Try to load zsh/system for sysread (used by daemon complete)
zmodload zsh/system 2>/dev/null

# Try to load zsh/net/unix for zsocket support
if zmodload zsh/net/unix 2>/dev/null; then
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

# === Non-blocking render (auto-trigger) ===

_zacrs_render() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row=0 cursor_col=0
    _zacrs_get_cursor_pos

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
            print -u $fd -- "render $cursor_row $cursor_col $cols $rows"
            printf '%s\n' "$prefix" >&$fd
            printf '%s\n' "$candidates_str" >&$fd
            print -u $fd "END"
            # Read response header
            local header
            IFS= read -r -u $fd header
            if [[ "$header" == OK* ]]; then
                # Parse: OK popup_row=N popup_height=N cursor_row=N reuse_token=N <tty_len>
                local token tty_len=0 reuse_token=""
                for token in ${(s: :)header}; do
                    local key="${token%%=*}" val="${token#*=}"
                    case "$key" in
                        popup_row)    _zacrs_popup_row=$val ;;
                        popup_height) _zacrs_popup_height=$val ;;
                        cursor_row)   _zacrs_popup_cursor_row=$val ;;
                        reuse_token)  reuse_token=$val ;;
                    esac
                    # Last token is tty_len (no = sign)
                    [[ "$token" != *=* ]] && tty_len=$token
                done
                # Read tty_bytes and pipe directly to /dev/tty
                local tty_ok=1
                if (( tty_len > 0 )); then
                    if ! head -c "$tty_len" <&$fd > /dev/tty; then
                        tty_ok=0
                    fi
                fi
                if (( tty_ok )); then
                    _zacrs_popup_visible=1
                    _zacrs_record_popup_snapshot "$prefix" "$prefix_len" "$candidates_str" "$cursor_col" "$reuse_token"
                else
                    _zacrs_reset_popup_snapshot
                    _zacrs_mark_daemon_unavailable
                fi
                exec {fd}<&-
                return
            elif [[ "$header" == EMPTY ]]; then
                _zacrs_reset_popup_snapshot
                exec {fd}<&-
                return
            elif [[ "$header" == ERROR* ]]; then
                _zacrs_reset_popup_snapshot
                _zacrs_mark_daemon_unavailable
            fi
            exec {fd}<&-
        fi
        # Socket connect failed, daemon may have died
        _zacrs_reset_popup_snapshot
        _zacrs_mark_daemon_unavailable
    fi

    # Fallback: subprocess
    local output
    output=$(printf '%s' "$candidates_str" | \
        "$ZACRS_BIN" render \
        --prefix "$prefix" \
        --cursor-row "$cursor_row" \
        --cursor-col "$cursor_col")
    local exit_code=$?

    if [[ $exit_code -eq 0 && -n "$output" ]]; then
        _zacrs_popup_visible=1
        local token
        for token in ${(s: :)output}; do
            local key="${token%%=*}" val="${token#*=}"
            case "$key" in
                popup_row)    _zacrs_popup_row=$val ;;
                popup_height) _zacrs_popup_height=$val ;;
                cursor_row)   _zacrs_popup_cursor_row=$val ;;
            esac
        done
        _zacrs_record_popup_snapshot "$prefix" "$prefix_len" "$candidates_str" "$cursor_col" ""
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
    printf '\e7' > /dev/tty
    local i
    for (( i = 0; i < _zacrs_popup_height; i++ )); do
        printf '\e[%d;1H\e[2K' $(( _zacrs_popup_row + i + 1 )) > /dev/tty
    done
    printf '\e8' > /dev/tty
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

_zacrs_invoke_daemon() {
    local prefix="$1" prefix_len="$2" candidates_str="$3"
    local cursor_row="${4:-}" cursor_col="${5:-}" reuse_visible="${6:-0}" reuse_token="${7:-}"
    if [[ -z "$cursor_row" || -z "$cursor_col" ]]; then
        cursor_row=0 cursor_col=0
        _zacrs_get_cursor_pos
    fi

    local fd
    zsocket "$_zacrs_socket_path" 2>/dev/null || return 1
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
            return 1
            ;;
    esac

    if (( initial_done )); then
        exec {fd}<&-
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
            case "$header" in
                FRAME*)
                    _zacrs_complete_parse_frame "$header"
                    if (( _f_tty_len > 0 )); then
                        sysread -i $fd -o $tty_wfd -c $_f_tty_len
                    fi
                    _zacrs_popup_row=$_f_popup_row
                    _zacrs_popup_height=$_f_popup_height
                    ;;
                DONE*)
                    local -a parts
                    parts=( ${(s: :)header} )
                    result_code="${parts[2]}"
                    # Extract text after "DONE <code> "
                    result_text="${header#DONE [0-9]## }"
                    [[ "$result_text" == "$header" ]] && result_text=""
                    break
                    ;;
                NONE) ;;
                *) break ;;
            esac
        done
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

# === Tab completion widget ===

_zacrs_tab_complete() {
    local prefix="" prefix_len=0 candidates_str=""
    local cursor_row="" cursor_col=""
    local reuse_visible=0
    local reuse_token=""

    if (( _zacrs_popup_visible )) \
        && [[ "$_zacrs_popup_snapshot_lbuffer" == "$LBUFFER" ]] \
        && [[ -n "$_zacrs_popup_snapshot_candidates" ]] \
        && (( _zacrs_popup_snapshot_columns == COLUMNS )) \
        && (( _zacrs_popup_snapshot_lines == LINES )); then
        reuse_visible=1
        prefix="$_zacrs_popup_snapshot_prefix"
        prefix_len=$_zacrs_popup_snapshot_prefix_len
        candidates_str="$_zacrs_popup_snapshot_candidates"
        cursor_row=$_zacrs_popup_snapshot_cursor_row
        cursor_col=$_zacrs_popup_snapshot_cursor_col
        reuse_token="$_zacrs_popup_snapshot_reuse_token"
    fi

    if (( ! reuse_visible )); then
        # 候補収集: compsys → gather fallback
        _zacrs_captured=()
        local _zacrs_fd2
        exec {_zacrs_fd2}>&2
        zle _zacrs_compsys 2>/dev/null
        exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-

        # compsys コンテキストから prefix 取得
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

        # Fuzzy fallback: auto-trigger キャッシュを再利用
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
        _zacrs_clear_popup
        local text="${cands[1]%%	*}"
        local kind="${${cands[1]##*	}}"
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
        # 末尾がスペース/スラッシュなら prev_lbuffer を更新せず
        # line-pre-redraw にチェーンさせる
        if [[ "$LBUFFER" == *[\ /] ]]; then
            _zacrs_prev_lbuffer="$base"
            _zacrs_chain_retry=1
        else
            _zacrs_prev_lbuffer="$LBUFFER"
        fi
        zle reset-prompt
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
        _zacrs_clear_popup
    else
        return
    fi
    _zacrs_prev_lbuffer="$LBUFFER"

    # 空 or 空白のみ → コマンド未入力なのでスキップ
    [[ ! "$LBUFFER" =~ [^[:space:]] ]] && return

    # DismissWithSpace 後の抑制: 非空 prefix 入力で解除 (naive prefix で十分)
    local naive_prefix="${LBUFFER##* }"
    if (( _zacrs_suppressed )); then
        if [[ -n "$naive_prefix" ]]; then
            _zacrs_suppressed=0
        else
            return
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
        _zacrs_cached_candidates=""
        _zacrs_cached_lbase="$lbase"
    fi

    # 候補収集: compsys → gather fallback
    local candidates_str=""
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
        fi
    fi

    # キャッシュ更新: 同じ lbase で初めて候補が見つかった場合
    if [[ -n "$candidates_str" && -z "$_zacrs_cached_candidates" ]]; then
        _zacrs_cached_candidates="$candidates_str"
    fi

    # Fuzzy fallback: 候補なし → キャッシュから再利用
    if [[ -z "$candidates_str" && -n "$_zacrs_cached_candidates" ]]; then
        candidates_str="$_zacrs_cached_candidates"
        prefix="$naive_prefix"
        prefix_len=${#naive_prefix}
    fi

    [[ -z "$candidates_str" ]] && return

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )
    [[ ${#cands[@]} -eq 0 ]] && return

    _zacrs_render "$prefix" "$prefix_len" "$candidates_str"
}

# === Widget wrappers: Enter/Ctrl-C でポップアップクリア ===

_zacrs_accept_line() {
    _zacrs_clear_popup
    _zacrs_prev_lbuffer="$LBUFFER"
    _zacrs_cached_candidates=""
    _zacrs_cached_lbase=""
    zle .accept-line
}
zle -N accept-line _zacrs_accept_line

_zacrs_send_break() {
    _zacrs_clear_popup
    _zacrs_prev_lbuffer=""
    _zacrs_cached_candidates=""
    _zacrs_cached_lbase=""
    zle .send-break
}
zle -N send-break _zacrs_send_break

# === ターミナルリサイズ対応 ===

TRAPWINCH() { _zacrs_clear_popup }

# === Register widgets and keybindings ===

zle -N _zacrs_tab_complete
bindkey '^I' _zacrs_tab_complete

# Register line-pre-redraw hook (auto-trigger without key rebinding)
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _zacrs_line_pre_redraw
