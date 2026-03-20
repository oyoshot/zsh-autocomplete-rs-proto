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

# === Non-blocking render (auto-trigger) ===

_zacrs_render() {
    local prefix="$1" candidates_str="$2"
    local cursor_row=0 cursor_col=0
    _zacrs_get_cursor_pos

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
    local candidates_str="$2"

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
    zle reset-prompt

    if [[ $exit_code -eq 0 && -n "$output" ]]; then
        # Confirm: replace prefix with selected candidate
        LBUFFER="${LBUFFER%$prefix}${output}"
        _zacrs_suppressed=0
    elif [[ $exit_code -eq 2 && -n "$output" ]]; then
        # DismissWithSpace: text+space, suppress until next word
        LBUFFER="${LBUFFER%$prefix}${output}"
        _zacrs_suppressed=1
    elif [[ $exit_code -eq 1 && -n "$output" ]]; then
        # Cancel with text change
        LBUFFER="${LBUFFER%$prefix}${output}"
        _zacrs_suppressed=0
    elif [[ $exit_code -eq 1 ]]; then
        # Cancel with no change
        _zacrs_suppressed=0
    fi

    # Prevent immediate re-trigger from line-pre-redraw
    _zacrs_prev_lbuffer="$LBUFFER"
}

# === Tab completion widget ===

_zacrs_tab_complete() {
    _zacrs_clear_popup

    local prefix="${LBUFFER##* }"

    # 候補収集: compsys → gather fallback
    _zacrs_captured=()
    if [[ -n "$ZACRS_DEBUG" ]]; then
        zle _zacrs_compsys
    else
        zle _zacrs_compsys 2>/dev/null
    fi

    local candidates_str=""
    if (( ${#_zacrs_captured} > 0 )); then
        candidates_str="${(pj:\n:)_zacrs_captured}"
    fi
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
    fi

    # タイポ補正: prefix matchで候補なし & コマンド位置 → 全コマンドをfuzzyに渡す
    if [[ -z "$candidates_str" && "$LBUFFER" == "$prefix" && ${#prefix} -ge 2 ]]; then
        candidates_str="$(_zacrs_gather --all-commands)"
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
        LBUFFER="${LBUFFER%$prefix}${text}"
        [[ "$kind" == "directory" && "$text" != */ ]] && LBUFFER+="/"
        _zacrs_prev_lbuffer="$LBUFFER"
        unset POSTDISPLAY
        zle reset-prompt
        return
    fi

    _zacrs_suppressed=0
    _zacrs_invoke "$prefix" "$candidates_str"
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

    # DismissWithSpace 後の抑制: 非空 prefix 入力で解除
    local prefix="${LBUFFER##* }"
    if (( _zacrs_suppressed )); then
        if [[ -n "$prefix" ]]; then
            _zacrs_suppressed=0
        else
            return
        fi
    fi

    # 候補収集: compsys → gather fallback
    local candidates_str=""
    _zacrs_captured=()
    if [[ -n "$ZACRS_DEBUG" ]]; then
        zle _zacrs_compsys
    else
        zle _zacrs_compsys 2>/dev/null
    fi
    if (( ${#_zacrs_captured} > 0 )); then
        candidates_str="${(pj:\n:)_zacrs_captured}"
    fi
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
    fi

    # タイポ補正: prefix matchで候補なし & コマンド位置 → 全コマンドをfuzzyに渡す
    if [[ -z "$candidates_str" && "$LBUFFER" == "$prefix" && ${#prefix} -ge 2 ]]; then
        candidates_str="$(_zacrs_gather --all-commands)"
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
    zle .accept-line
}
zle -N accept-line _zacrs_accept_line

_zacrs_send_break() {
    _zacrs_clear_popup
    _zacrs_prev_lbuffer=""
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
