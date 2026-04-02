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

_zacrs_compsys_func() {
    typeset -ga _zacrs_captured=()
    typeset -gi _zacrs_compadd_calls=0
    typeset -gi _zacrs_ctx_valid=0
    typeset -g  _zacrs_ctx_prefix=""
    typeset -gi _zacrs_ctx_prefix_len=0

    [[ -n "$ZACRS_DEBUG" ]] && print -r -- "=== compsys $(date '+%H:%M:%S') BUFFER='$BUFFER' LBUFFER='$LBUFFER' ===" >> "$ZACRS_LOG"

    # Ensure completion system is ready
    _zacrs_ensure_compinit
    _zacrs_dbg "funcs: _main_complete=$+functions[_main_complete] _normal=$+functions[_normal] _comp_setup=${${+_comp_setup}:-unset}"

    # Override compadd to capture candidates
    compadd() {
        (( _zacrs_compadd_calls++ ))

        # Skip probe calls that use -O or -D (internal completion system tests)
        local _a _skip=0 _xdesc="" _vis_prefix="" _vis_suffix="" _hidden_prefix="" _is_file=0 _disp_array_name=""
        local _prev=""
        for _a in "$@"; do
            if [[ "$_prev" == -[pPSXd] ]]; then
                : # この "--" はフラグの値 → breakしない
            else
                [[ "$_a" == "--" ]] && break
            fi
            [[ "$_a" == "-O" || "$_a" == "-D" ]] && { _skip=1; break; }
            [[ "$_prev" == "-P" ]] && _vis_prefix="$_a"
            [[ "$_prev" == "-p" ]] && _hidden_prefix="$_a"
            [[ "$_prev" == "-S" ]] && _vis_suffix="$_a"
            [[ "$_prev" == "-X" ]] && _xdesc="$_a"
            [[ "$_prev" == "-d" ]] && _disp_array_name="$_a"
            [[ "$_a" == "-f" ]] && _is_file=1
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
                    [[ -d "${_full_prefix}${_m}" ]] && _kind="directory" || _kind="file"
                elif [[ "$_text" == */ ]]; then
                    _kind="directory"
                fi
                _zacrs_captured+=( "${_text}"$'\t'"${_desc_map[$_m]:-$_xdesc}"$'\t'"${_kind}" )
            done
        else
            _zacrs_dbg "  compadd[$_zacrs_compadd_calls]: SKIPPED (-O/-D) args: ${(j: :)${(@q)@}}"
        fi

        builtin compadd "$@"
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
        # IPREFIX+PREFIX always equals the raw LBUFFER suffix that compsys is
        # completing, so its length is the precise number of characters to strip.
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
