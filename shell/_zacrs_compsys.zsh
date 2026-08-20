# zsh-autocomplete-rs: compadd capture via completion widget
#
# Uses zle -C to register a completion widget that invokes the full zsh
# completion system while capturing candidates via a compadd function
# override with builtin compadd -O.
# Format: text\tdescription\tkind

# Debug log file (only used when ZACRS_DEBUG=1)
typeset -g ZACRS_LOG="${TMPDIR:-/tmp}/zacrs-debug.log"

_zacrs_dbg() {
    [[ -n "$ZACRS_DEBUG" ]] && print -r -- "$@" >> "$ZACRS_LOG"
}

_zacrs_ensure_compinit() {
    # Already available — nothing to do
    (( $+functions[_main_complete] )) && return 0

    # compinit was called but _main_complete not autoloaded yet — try loading
    if [[ -v _comp_setup ]]; then
        autoload -Uz _main_complete 2>/dev/null
        (( $+functions[_main_complete] )) && return 0
    fi

    # compinit hasn't run at all — initialize it
    _zacrs_dbg "compinit: initializing (was not called yet)"
    autoload -Uz compinit 2>/dev/null && compinit -C -i -d "${ZSH_COMPDUMP:-${XDG_CACHE_HOME:-$HOME/.cache}/zsh/compdump}" 2>/dev/null

    (( $+functions[_main_complete] ))
}

_zacrs_compadd_has_file_flag() {
    local _arg _chars _char
    local -i _idx _expect_value=0

    for _arg in "$@"; do
        if (( _expect_value )); then
            _expect_value=0
            continue
        fi
        [[ "$_arg" == "-" || "$_arg" == "--" ]] && break
        [[ "$_arg" == -?* ]] || continue

        _chars="${_arg[2,-1]}"
        for (( _idx = 1; _idx <= ${#_chars}; _idx++ )); do
            _char="${_chars[_idx]}"
            case "$_char" in
                f) return 0 ;;
                [akqQenUl12C]) ;;
                [diIOADVJXxPSpsWFMrRE])
                    (( _idx == ${#_chars} )) && _expect_value=1
                    break
                    ;;
                o) break ;;
                *) break ;;
            esac
        done
    done

    return 1
}

_zacrs_compadd_has_array_output_flag() {
    local _arg _chars _char
    local -i _idx _expect_value=0

    for _arg in "$@"; do
        if (( _expect_value )); then
            _expect_value=0
            continue
        fi
        [[ "$_arg" == "-" || "$_arg" == "--" ]] && break
        [[ "$_arg" == -?* ]] || continue

        _chars="${_arg[2,-1]}"
        for (( _idx = 1; _idx <= ${#_chars}; _idx++ )); do
            _char="${_chars[_idx]}"
            case "$_char" in
                [OD]) return 0 ;;
                [akqQenUfl12C]) ;;
                [diIAVJXxPSpsWFMrRE])
                    (( _idx == ${#_chars} )) && _expect_value=1
                    break
                    ;;
                o) break ;;
                *) break ;;
            esac
        done
    done

    return 1
}

_zacrs_path_candidate_kind() {
    local _path="$1"
    case "$_path" in
        '~') _path="$HOME" ;;
        '~/'*) _path="${HOME}${_path[2,-1]}" ;;
    esac

    if [[ -d "$_path" ]]; then
        REPLY="directory"
    else
        REPLY="file"
    fi
}

_zacrs_compsys_func() {
    typeset -ga _zacrs_captured=()
    typeset -gi _zacrs_compadd_calls=0
    typeset -gi _zacrs_ctx_valid=0
    typeset -g  _zacrs_ctx_prefix=""
    typeset -gi _zacrs_ctx_prefix_len=0
    local _zacrs_cmd_pos=0
    _zacrs_is_cmd_pos "$LBUFFER" "${LBUFFER##* }" && _zacrs_cmd_pos=1

    [[ -n "$ZACRS_DEBUG" ]] && print -r -- "=== compsys $(date '+%H:%M:%S') BUFFER='$BUFFER' LBUFFER='$LBUFFER' ===" >> "$ZACRS_LOG"

    # Ensure completion system is ready
    _zacrs_ensure_compinit
    _zacrs_dbg "funcs: _main_complete=$+functions[_main_complete] _normal=$+functions[_normal] _comp_setup=${${+_comp_setup}:-unset}"

    # Override compadd to capture candidates
    compadd() {
        (( _zacrs_compadd_calls++ ))

        # Skip probe calls that use -O or -D (internal completion system tests)
        local _a _skip=0 _xdesc="" _vis_prefix="" _vis_suffix="" _hidden_prefix="" _is_file=0 _disp_array_name=""
        local _prev="" _is_option_value=0 _prev_flags=""
        _zacrs_compadd_has_file_flag "$@" && _is_file=1
        _zacrs_compadd_has_array_output_flag "$@" && _skip=1
        for _a in "$@"; do
            _is_option_value=0
            if (( ${#_prev} >= 2 )); then
                _prev_flags="${_prev[2,-2]}"
                if [[ "${_prev[-1]}" == [diIOADVJXxPSpsWFMrRE] && "${_prev_flags//[akqQfenUl12C]/}" == "" ]]; then
                    _is_option_value=1
                fi
            fi
            if (( _is_option_value )); then
                : # この "-" / "--" はフラグの値 → breakしない
            else
                [[ "$_a" == "-" || "$_a" == "--" ]] && break
            fi
            [[ "$_prev" == "-P" ]] && _vis_prefix="$_a"
            [[ "$_prev" == "-p" ]] && _hidden_prefix="$_a"
            [[ "$_prev" == "-S" ]] && _vis_suffix="$_a"
            [[ "$_prev" == "-X" ]] && _xdesc="$_a"
            [[ "$_prev" == "-d" ]] && _disp_array_name="$_a"
            _prev="$_a"
        done

        if (( ! _skip )); then
            local -A _desc_map=()
            if [[ -n "$_disp_array_name" ]]; then
                local _sep="${LIST_SEPARATOR:- -- }"
                local _disp_elem _key
                for _disp_elem in "${(@P)_disp_array_name}"; do
                    if [[ "$_disp_elem" == *"$_sep"* ]]; then
                        _key="${_disp_elem%%"$_sep"*}"
                        _key="${_key%"${_key##*[! ]}"}"  # trim trailing spaces (_describe pads for alignment)
                        _desc_map[$_key]="${_disp_elem#*"$_sep"}"
                    fi
                done
                _zacrs_dbg "  compadd[$_zacrs_compadd_calls]: disp=$_disp_array_name desc_map_size=${#_desc_map}"
            fi

            local -a _zacrs_cap=()
            builtin compadd -O _zacrs_cap "$@" 2>/dev/null

            _zacrs_dbg "  compadd[$_zacrs_compadd_calls]: captured=${#_zacrs_cap} skip=0 args: ${(j: :)${(@q)@}}"

            local _full_prefix="${IPREFIX}${_hidden_prefix}${_vis_prefix}"
            local _m
            for _m in "${_zacrs_cap[@]}"; do
                local _text="${_full_prefix}${_m}${_vis_suffix}"
                local _kind=""
                if (( _is_file )); then
                    _zacrs_path_candidate_kind "${_full_prefix}${_m}"
                    _kind="$REPLY"
                elif [[ "$_text" == */ ]]; then
                    _kind="directory"
                elif (( _zacrs_cmd_pos )) && [[ "$_text" != */* ]] && _zacrs_command_kind "$_m"; then
                    _kind="$REPLY"
                fi
                _zacrs_captured+=( "${_text}"$'\t'"${_desc_map[$_m]:-$_xdesc}"$'\t'"${_kind}" )
            done

            # Completion helpers use compstate[nmatches] to decide whether a
            # source succeeded and whether to try alternatives. Preserve zero,
            # single, and many-match behavior without re-adding a pathological
            # candidate set in full. Native insertion/list output is suppressed
            # after _main_complete returns.
            if (( ${#_zacrs_cap} > 0 )); then
                local -a _zacrs_state_matches=( "${_zacrs_cap[1,101]}" )
                builtin compadd -QU - "${_zacrs_state_matches[@]}"
            fi
        else
            _zacrs_dbg "  compadd[$_zacrs_compadd_calls]: SKIPPED (-O/-D) args: ${(j: :)${(@q)@}}"
            # Probe calls explicitly request their own output arrays and must
            # retain the original compadd behavior. Regular calls are captured
            # above only: zacrs renders its own popup and suppresses compsys'
            # insertion and list output after completion returns.
            builtin compadd "$@"
        fi
    }

    # Call the completion system entry point
    local _zacrs_entry_found=0
    if (( $+functions[_main_complete] )); then
        _zacrs_dbg "entry: _main_complete"
        _main_complete
        _zacrs_entry_found=1
    elif (( $+functions[_normal] )); then
        _zacrs_dbg "entry: _normal (fallback)"
        _normal
        _zacrs_entry_found=1
    else
        _zacrs_dbg "entry: NONE FOUND (even after compinit attempt)"
    fi

    # ---- Capture completion context ----
    if (( _zacrs_entry_found )); then
        _zacrs_ctx_valid=1
        _zacrs_ctx_prefix="${IPREFIX}${PREFIX}"

        # Prefix length on LBUFFER: use the length of IPREFIX+PREFIX directly.
        # Using ${(z)LBUFFER} to get the last token length is fragile when
        # LBUFFER contains an open (unclosed) quote character such as `"s` —
        # zsh's (z) flag fails to parse the unmatched quote and may return an
        # empty array, leaving prefix_len=0.  With prefix_len=0 the apply step
        # treats the whole LBUFFER as the base and prepends it to the candidate
        # text, producing doubled output like `"ssrc`.
        # _zacrs_ctx_prefix (IPREFIX+PREFIX) is exactly what this implementation
        # uses for candidate reconstruction and LBUFFER trimming, so its length
        # is the correct character count to strip.
        # Caveat: when compset -q further splits the prefix, zsh stores the
        # additional dequoted text in QIPREFIX; this implementation does not
        # model QIPREFIX, so deeply-nested quoting paths are untested.
        _zacrs_ctx_prefix_len=${#_zacrs_ctx_prefix}

        _zacrs_dbg "context: PREFIX='$PREFIX' IPREFIX='$IPREFIX' ctx_prefix='$_zacrs_ctx_prefix' raw_len=$_zacrs_ctx_prefix_len"
    fi
    _zacrs_dbg "result: compadd_calls=$_zacrs_compadd_calls captured=${#_zacrs_captured}"

    unfunction compadd 2>/dev/null

    # Suppress actual insertion and menu display
    compstate[insert]=
    compstate[list]=
}

# Register as a completion widget
zle -C _zacrs_compsys complete-word _zacrs_compsys_func
