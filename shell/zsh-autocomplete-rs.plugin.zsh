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

# Settings via zstyle (defaults)
zstyle -s ':zacrs:' min-input '_zacrs_min_input' || _zacrs_min_input=1

# Internal state
typeset -g _zacrs_prev_lbuffer=""
typeset -g _zacrs_suppressed=0
typeset -g _zacrs_in_popup=0

# === Core: invoke Rust binary ===

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
    local prefix="$(_zacrs_get_prefix)"

    # Fallback to default completion if no prefix
    if [[ -z "$prefix" ]]; then
        zle expand-or-complete
        return
    fi

    # Try compsys first for full context-aware completion
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

    # Fallback to simple gather if compsys returned nothing
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
    fi

    # Fallback if still no candidates
    if [[ -z "$candidates_str" ]]; then
        zle expand-or-complete
        return
    fi

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )

    # Single candidate: complete immediately
    if [[ ${#cands[@]} -eq 1 ]]; then
        local text="${cands[1]%%	*}"
        LBUFFER="${LBUFFER%$prefix}${text}"
        zle reset-prompt
        return
    fi

    _zacrs_suppressed=0
    _zacrs_invoke "$prefix" "$candidates_str"
}

# === Auto-trigger via line-pre-redraw hook ===

_zacrs_line_pre_redraw() {
    (( _zacrs_in_popup )) && return

    [[ "$LBUFFER" == "$_zacrs_prev_lbuffer" ]] && return
    _zacrs_prev_lbuffer="$LBUFFER"

    # Buffer ends with space → no word to complete → skip
    [[ "$LBUFFER" == *" " ]] && return

    # Suppression: reaching here means a non-space char was typed → new word
    if (( _zacrs_suppressed )); then
        _zacrs_suppressed=0
    fi

    local prefix="$(_zacrs_get_prefix)"
    [[ ${#prefix} -lt $_zacrs_min_input ]] && return
    # "." and ".." are path literals, not dotfile prefixes — skip auto-trigger
    [[ "$prefix" == "." || "$prefix" == ".." ]] && return

    # Gather candidates: use compsys for 2nd+ words, gather as fallback
    local candidates_str=""
    if [[ "$LBUFFER" != "$prefix" ]]; then
        _zacrs_captured=()
        zle _zacrs_compsys 2>/dev/null
        if (( ${#_zacrs_captured} > 0 )); then
            candidates_str="${(pj:\n:)_zacrs_captured}"
        fi
    fi
    if [[ -z "$candidates_str" ]]; then
        candidates_str="$(_zacrs_gather "$LBUFFER")"
    fi
    [[ -z "$candidates_str" ]] && return

    local -a cands
    cands=( ${(f)candidates_str} )
    cands=( ${cands:#} )
    [[ ${#cands[@]} -lt 2 ]] && return

    _zacrs_in_popup=1
    _zacrs_invoke "$prefix" "$candidates_str"
    _zacrs_in_popup=0
}

# === Register widgets and keybindings ===

zle -N _zacrs_tab_complete
bindkey '^I' _zacrs_tab_complete

# Register line-pre-redraw hook (auto-trigger without key rebinding)
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _zacrs_line_pre_redraw
