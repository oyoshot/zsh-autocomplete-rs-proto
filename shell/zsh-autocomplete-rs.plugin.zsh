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
typeset -g _zacrs_daemon_available=0
typeset -g _zacrs_daemon_started=0
typeset -g _zacrs_socket_path=""

# === Daemon lifecycle ===

# Try to load zsh/net/unix for zsocket support
if zmodload zsh/net/unix 2>/dev/null; then
    _zacrs_socket_path="${XDG_RUNTIME_DIR:-/tmp}/zacrs.sock"
    [[ -z "$XDG_RUNTIME_DIR" ]] && _zacrs_socket_path="/tmp/zacrs-${USER:-unknown}.sock"

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
        fi
    }
    _zacrs_ensure_daemon

    _zacrs_zshexit() {
        (( _zacrs_daemon_started )) && "$ZACRS_BIN" daemon stop 2>/dev/null
    }
    autoload -Uz add-zsh-hook
    add-zsh-hook zshexit _zacrs_zshexit
fi

# === Non-blocking render (auto-trigger) ===

_zacrs_render() {
    local prefix="$1" candidates_str="$2"
    local cursor_row=0 cursor_col=0
    _zacrs_get_cursor_pos

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
                # Parse: OK popup_row=N popup_height=N cursor_row=N <tty_len>
                local token tty_len=0
                for token in ${(s: :)header}; do
                    local key="${token%%=*}" val="${token#*=}"
                    case "$key" in
                        popup_row)    _zacrs_popup_row=$val ;;
                        popup_height) _zacrs_popup_height=$val ;;
                        cursor_row)   _zacrs_popup_cursor_row=$val ;;
                    esac
                    # Last token is tty_len (no = sign)
                    [[ "$token" != *=* ]] && tty_len=$token
                done
                # Read tty_bytes and pipe directly to /dev/tty
                if (( tty_len > 0 )); then
                    dd bs=$tty_len count=1 <&$fd 2>/dev/null > /dev/tty
                fi
                _zacrs_popup_visible=1
                exec {fd}<&-
                return
            elif [[ "$header" == EMPTY || "$header" == ERROR* ]]; then
                exec {fd}<&-
                return
            fi
            exec {fd}<&-
        fi
        # Socket connect failed, daemon may have died
        _zacrs_daemon_available=0
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
    fi
}

# === Clear popup (zsh-native, no process spawn) ===

_zacrs_clear_popup() {
    (( _zacrs_popup_visible )) || return 0
    printf '\e7' > /dev/tty
    local i
    for (( i = 0; i < _zacrs_popup_height; i++ )); do
        printf '\e[%d;1H\e[2K' $(( _zacrs_popup_row + i + 1 )) > /dev/tty
    done
    printf '\e8' > /dev/tty
    _zacrs_popup_visible=0
}

# === Core: invoke Rust binary (blocking, for Tab) ===

_zacrs_invoke() {
    local prefix="$1"
    local prefix_len="$2"
    local candidates_str="$3"

    local cursor_row=0 cursor_col=0
    _zacrs_get_cursor_pos

    local output
    output=$(printf '%s' "$candidates_str" | \
        "$ZACRS_BIN" complete \
        --prefix "$prefix" \
        --cursor-row "$cursor_row" \
        --cursor-col "$cursor_col")
    local exit_code=$?

    unset POSTDISPLAY

    local base
    if (( prefix_len > 0 )); then
        base="${LBUFFER[1,-(prefix_len+1)]}"
    else
        base="$LBUFFER"
    fi

    if [[ $exit_code -eq 0 && -n "$output" ]]; then
        # Confirm: replace prefix with selected candidate
        LBUFFER="${base}${output}"
        _zacrs_suppressed=0
    elif [[ $exit_code -eq 2 && -n "$output" ]]; then
        # DismissWithSpace: text+space, suppress until next word
        LBUFFER="${base}${output}"
        _zacrs_suppressed=1
    elif [[ $exit_code -eq 1 && -n "$output" ]]; then
        # Cancel with text change
        LBUFFER="${base}${output}"
        _zacrs_suppressed=0
    elif [[ $exit_code -eq 1 ]]; then
        # Cancel with no change
        _zacrs_suppressed=0
    fi

    # Prevent immediate re-trigger from line-pre-redraw
    _zacrs_prev_lbuffer="$LBUFFER"
    zle reset-prompt
}

# === Tab completion widget ===

_zacrs_tab_complete() {
    _zacrs_clear_popup

    # 候補収集: compsys → gather fallback
    _zacrs_captured=()
    local _zacrs_fd2
    exec {_zacrs_fd2}>&2
    zle _zacrs_compsys 2>/dev/null
    exec 2>&$_zacrs_fd2 {_zacrs_fd2}>&-

    # compsys コンテキストから prefix 取得
    local prefix prefix_len
    if (( _zacrs_ctx_valid )); then
        prefix="$_zacrs_ctx_prefix"
        prefix_len=$_zacrs_ctx_prefix_len
    else
        prefix="${LBUFFER##* }"
        prefix_len=${#prefix}
    fi

    local candidates_str=""
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

    # タイポ補正: コマンド位置で候補なし → 全コマンドをfuzzyに渡す
    if [[ -z "$candidates_str" && ${#prefix} -ge 2 ]] && _zacrs_is_cmd_pos "$LBUFFER" "$prefix"; then
        candidates_str="$(_zacrs_gather --all-commands)"
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

    # 候補なし → default zsh 補完にフォールバック
    if [[ -z "$candidates_str" ]]; then
        zle expand-or-complete
        return
    fi

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )

    # 単一候補 → 即補完
    if [[ ${#cands[@]} -eq 1 ]]; then
        local text="${cands[1]%%	*}"
        local kind="${${cands[1]##*	}}"
        local base
        if (( prefix_len > 0 )); then
            base="${LBUFFER[1,-(prefix_len+1)]}"
        else
            base="$LBUFFER"
        fi
        LBUFFER="${base}${text}"
        [[ "$kind" == "directory" && "$text" != */ ]] && LBUFFER+="/"
        _zacrs_prev_lbuffer="$LBUFFER"
        unset POSTDISPLAY
        zle reset-prompt
        return
    fi

    _zacrs_suppressed=0
    _zacrs_invoke "$prefix" "$prefix_len" "$candidates_str"
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
    local prefix
    if (( _zacrs_ctx_valid )); then
        prefix="$_zacrs_ctx_prefix"
    else
        prefix="$naive_prefix"
    fi

    if (( ${#_zacrs_captured} > 0 )); then
        candidates_str="${(pj:\n:)_zacrs_captured}"
    fi
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
        if [[ -n "$candidates_str" ]]; then
            prefix="$naive_prefix"
        fi
    fi

    # タイポ補正: コマンド位置で候補なし → 全コマンドをfuzzyに渡す
    if [[ -z "$candidates_str" && ${#naive_prefix} -ge 2 ]] && _zacrs_is_cmd_pos "$LBUFFER" "$naive_prefix"; then
        candidates_str="$(_zacrs_gather --all-commands)"
    fi

    # キャッシュ更新: 同じ lbase で初めて候補が見つかった場合
    if [[ -n "$candidates_str" && -z "$_zacrs_cached_candidates" ]]; then
        _zacrs_cached_candidates="$candidates_str"
    fi

    # Fuzzy fallback: 候補なし → キャッシュから再利用
    if [[ -z "$candidates_str" && -n "$_zacrs_cached_candidates" ]]; then
        candidates_str="$_zacrs_cached_candidates"
        prefix="$naive_prefix"
    fi

    [[ -z "$candidates_str" ]] && return

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )
    [[ ${#cands[@]} -eq 0 ]] && return

    _zacrs_render "$prefix" "$candidates_str"
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
