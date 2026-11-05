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
# The shell-managed generation scopes reuse to one executed command. It is
# advanced from preexec so every accepting ZLE widget follows the same path.
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

# Parse a standalone command without consulting configuration or changing ZLE.
# Args: buffer, prebuffer. Returns reply=(trigger leading suffix context).
_zacrs_abbreviation_target() {
    local _buffer="$1" _prebuffer="${2:-}"
    reply=()
    # At a continuation prompt BUFFER is only the new line; preceding input
    # (including heredoc delimiters and open quotes) lives in PREBUFFER.
    [[ -n "$_prebuffer" ]] && return 1
    local -a _tokens
    _tokens=( ${(z)_buffer} ) 2>/dev/null || return 1
    (( ${#_tokens} )) || return 1
    local _word="${_tokens[1]}" _token
    # Quoted/escaped words, paths, assignments and substitutions are explicit
    # shell syntax and must not be reinterpreted as an abbreviation trigger.
    [[ "$_word" == *[^[:alnum:]_.+-]* || "$_word" == ('.'|'..') ]] && return 1
    [[ "$_word" == ('if'|'then'|'elif'|'else'|'fi'|'for'|'foreach'|'while'|'until'|'repeat'|'do'|'done'|'case'|'esac'|'select'|'function'|'coproc'|'time'|'nocorrect'|'noglob'|'command'|'exec'|'builtin'|'end') ]] && return 1
    # Compound commands need a separate replacement-range model. Leave the
    # entire line alone rather than partially expanding a pipeline or list.
    # Zsh emits a single () token for function declarations, even without braces.
    for _token in "${_tokens[@]}"; do
        [[ "$_token" == ('|'|'|&'|'||'|'&&'|'&'|'&!'|'&|'|';'|$'\n'|'('|')'|'()'|'{'|'}') ]] && return 1
    done

    local _leading="${_buffer%%[^[:space:]]*}"
    local -i _end=$(( ${#_leading} + ${#_word} ))
    [[ "${_buffer[${#_leading}+1,_end]}" == "$_word" ]] || return 1
    local _context="${(j: :)${(@Q)_tokens}}"
    [[ "$_buffer" == *[[:space:]] ]] && _context+=" "
    reply=("$_word" "$_leading" "${_buffer[_end+1,-1]}" "$_context")
}

# Resolve the parsed trigger through Rust and apply the literal edit to ZLE.
# On success REPLY is 1 to execute, or 0 to keep editing a cursor placeholder.
_zacrs_expand_command_abbreviation() {
    _zacrs_update_enter_selection
    [[ -n "$_zacrs_enter_selected_command" ]] && return 1
    local -a reply
    _zacrs_abbreviation_target "$BUFFER" "${PREBUFFER:-}" || return 1
    local _word="$reply[1]" _leading="$reply[2]" _suffix="$reply[3]" _context="$reply[4]"
    local _response
    _response=$("$ZACRS_BIN" expand-abbreviation --trigger "$_word" \
        --command-context "$_context" 2>/dev/null) || return 1
    local -a _lines=( "${(@f)_response}" )
    (( ${#_lines} == 2 )) || return 1
    [[ "${_lines[1]}" == 'DONE 0 '* || "${_lines[1]}" == 'DONE 0' ]] || return 1
    [[ "${_lines[2]}" == 'APPLY '* ]] || return 1
    local result_code result_text chain execute restore_text cursor_offset selection_applied
    _zacrs_parse_done_line "${_lines[1]}"
    _zacrs_parse_apply_line "${_lines[2]}"
    BUFFER="${_leading}${result_text}${_suffix}"
    if [[ "$cursor_offset" == <-> ]]; then
        CURSOR=$(( ${#_leading} + cursor_offset ))
    else
        CURSOR=${#BUFFER}
    fi
    REPLY="$execute"
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
