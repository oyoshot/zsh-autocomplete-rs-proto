# zsh-autocomplete-rs: utility functions

# Get terminal cursor position (0-indexed row, col)
# Sets: cursor_row, cursor_col
_zacrs_get_cursor_pos() {
    local _buf="" _byte="" _found=0
    typeset -g _zacrs_cursor_stale=""
    echo -ne '\e[6n' > /dev/tty
    # Read byte-by-byte until the full DSR pattern \e[row;colR is found.
    # Unlike `read -d R`, this is not confused by buffered keystrokes
    # that happen to contain 'R' or '['.  Bytes preceding the ESC are
    # saved in _zacrs_cursor_stale so callers can re-inject them.
    while IFS='' read -t 1 -rs -k 1 _byte < /dev/tty; do
        _buf+="$_byte"
        if [[ "$_buf" =~ $'\e\\[([0-9]+);([0-9]+)R$' ]]; then
            cursor_row=$(( match[1] - 1 ))
            cursor_col=$(( match[2] - 1 ))
            _found=1
            break
        fi
        # Safety: give up after 256 bytes (normal response is < 20)
        (( ${#_buf} > 256 )) && break
    done

    # Expose all bytes before the DSR response (including ESC sequences
    # such as arrow keys) so callers can re-inject them.
    if (( _found )); then
        _zacrs_cursor_stale="${_buf[1,MBEGIN-1]}"
    fi

    if (( ! _found )); then
        # Fallback: bottom of terminal, column 0
        cursor_row=$(( LINES - 1 ))
        cursor_col=0
        _zacrs_dbg "get_cursor_pos: DSR failed, fallback (raw='$_buf')"
    fi

    # Clamp to terminal bounds
    (( cursor_row >= LINES )) && cursor_row=$(( LINES - 1 ))
    (( cursor_col >= COLUMNS )) && cursor_col=$(( COLUMNS - 1 ))

    _zacrs_dbg "get_cursor_pos: row=$cursor_row col=$cursor_col stale=${#_zacrs_cursor_stale}"
}

# Build the daemon candidate-cache key for the current argument position.
# The shell-managed generation scopes reuse to one ZLE editing session:
# candidate sources may change after a command runs even when PID, PWD, and the
# command line base stay equal.
# Sets REPLY to an empty string at command position.
_zacrs_current_context_key() {
    REPLY=""
    local lbase=""
    if [[ "$LBUFFER" == *" "* ]]; then
        lbase="${LBUFFER% *} "
    fi
    [[ -z "$lbase" ]] && return 0

    local _ctx_lbase="$lbase"
    _ctx_lbase="${_ctx_lbase//\%/%25}"
    _ctx_lbase="${_ctx_lbase//:/%3A}"
    _ctx_lbase="${_ctx_lbase// /%20}"
    local _ctx_pwd="$PWD"
    _ctx_pwd="${_ctx_pwd//\%/%25}"
    _ctx_pwd="${_ctx_pwd//:/%3A}"
    _ctx_pwd="${_ctx_pwd// /%20}"
    REPLY="${$}:${_zacrs_candidate_cache_generation:-0}:${_ctx_pwd}:${_ctx_lbase}"
}

# Check if the last word in buffer is in command position
# (first word, or immediately after | || && ;)
# Args: $1=buffer $2=prefix (last word)
# Returns: 0 if command position, 1 otherwise
_zacrs_is_cmd_pos() {
    [[ "$1" == "$2" ]] && return 0
    local -a _toks=( ${(z)1} )
    (( ${#_toks} >= 2 )) && [[ "${_toks[-2]}" == ('|'|'||'|'&&'|';') ]] && return 0
    return 1
}

# Return the current simple-command context through the cursor, normalized to
# one space between parsed words. Separators start a new simple command. A
# trailing blank is preserved so patterns such as "cargo *" match a new
# argument position before any prefix has been typed.
# Args: $1=LBUFFER. Sets REPLY and _zacrs_command_context_is_command; returns 1
# when Zsh cannot tokenize the buffer.
_zacrs_command_context() {
    local _buffer="$1"
    local -a _tokens _segment
    typeset -g _zacrs_command_context_is_command=0
    _tokens=( ${(z)_buffer} ) 2>/dev/null || { REPLY=""; return 1; }

    local _token
    for _token in "${_tokens[@]}"; do
        if [[ "$_token" == ('|'|'|&'|'||'|'&&'|'&'|';'|$'\n') ]]; then
            _segment=()
        else
            _segment+=("${(Q)_token}")
        fi
    done

    REPLY="${(j: :)_segment}"
    if [[ "$_buffer" == *[[:space:]] ]]; then
        REPLY+=" "
        (( ${#_segment} == 0 )) && _zacrs_command_context_is_command=1
    else
        (( ${#_segment} <= 1 )) && _zacrs_command_context_is_command=1
    fi
    return 0
}

# Render even when compsys produced no candidates. The Rust side may still
# contribute configured abbreviations at any command-line position.
# Args: $1=candidates $2=buffer $3=prefix
_zacrs_should_render_candidates() {
    return 0
}

# Infer the shell command kind for a bare command-position token.
# Sets REPLY to one of: alias, builtin, function, command.
_zacrs_command_kind() {
    local name="$1"
    REPLY=""
    [[ -z "$name" || "$name" == */* ]] && return 1

    if (( ${+aliases[$name]} )); then
        REPLY="alias"
    elif (( ${+functions[$name]} )) && [[ "$name" != _* ]]; then
        REPLY="function"
    elif (( ${+builtins[$name]} )); then
        REPLY="builtin"
    elif (( ${+commands[$name]} )); then
        REPLY="command"
    else
        return 1
    fi

    return 0
}
