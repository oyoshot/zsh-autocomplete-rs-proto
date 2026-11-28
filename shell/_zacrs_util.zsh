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

# Write the glob and literal-word policy into the caller's _zacrs_file_option_key.
_zacrs_file_options() {
    _zacrs_file_option_key="${options[caseglob]}:${options[casepaths]:-off}:${options[extendedglob]}:${options[magicequalsubst]}:${options[multibyte]}:${options[interactivecomments]}:${options[banghist]}:${options[completealiases]}:${options[aliases]}:${options[globdots]}"
}

_zacrs_file_context_matches() {
    local _zacrs_file_option_key
    _zacrs_file_options
    [[ "$1" == "$PWD" && "$2" == "${HOME:-}" && "$3" == "$_zacrs_file_option_key" ]]
}

_zacrs_pwd_matches_cwd() {
    [[ "$PWD" == /* && . -ef "$PWD" ]]
}

# Build the daemon candidate-cache key for the current argument position.
# The shell-managed generation scopes reuse to one executed command. It is
# advanced from preexec so every accepting ZLE widget follows the same path.
# Sets REPLY to an empty string at command position.
_zacrs_current_context_key() {
    REPLY=""
    _zacrs_filesystem_editor || return 0
    # Skip context-keyed caching for large buffers before encoding fields.
    (( ${#LBUFFER} + ${#${RBUFFER:-}} + ${#PWD} + ${#${HOME:-}} <= 1024 )) || return 0
    local lbase=""
    if [[ "$LBUFFER" == *" "* ]]; then
        lbase="${LBUFFER% *} "
    fi
    [[ -z "$lbase" ]] && return 0
    _zacrs_pwd_matches_cwd || return 0

    local -a _ctx_fields=("$PWD" "$lbase" "${RBUFFER:-}" "${HOME:-}")
    _ctx_fields=("${(@)_ctx_fields//\%/%25}")
    _ctx_fields=("${(@)_ctx_fields//:/%3A}")
    # Rust's header parser uses Unicode split_whitespace. Encode its complete
    # delimiter set, independent of the interactive shell's current locale.
    _ctx_fields=("${(@)_ctx_fields//$'\t'/%09}")
    _ctx_fields=("${(@)_ctx_fields//$'\n'/%0A}")
    _ctx_fields=("${(@)_ctx_fields//$'\v'/%0B}")
    _ctx_fields=("${(@)_ctx_fields//$'\f'/%0C}")
    _ctx_fields=("${(@)_ctx_fields//$'\r'/%0D}")
    _ctx_fields=("${(@)_ctx_fields// /%20}")
    # Keep ordinary ASCII input out of the uncommon Unicode substitutions.
    local _ctx_wide=$'\xc2\x85\xc2\xa0\xe1\x9a\x80\xe2\x80\x80\xe2\x80\x81\xe2\x80\x82\xe2\x80\x83\xe2\x80\x84\xe2\x80\x85\xe2\x80\x86\xe2\x80\x87\xe2\x80\x88\xe2\x80\x89\xe2\x80\x8a\xe2\x80\xa8\xe2\x80\xa9\xe2\x80\xaf\xe2\x81\x9f\xe3\x80\x80'
    if [[ "${(j::)_ctx_fields}" == *[${_ctx_wide}]* ]]; then
        local -a _ctx_spaces=(
            $'\xc2\x85' $'\xc2\xa0' $'\xe1\x9a\x80'
            $'\xe2\x80\x80' $'\xe2\x80\x81' $'\xe2\x80\x82'
            $'\xe2\x80\x83' $'\xe2\x80\x84' $'\xe2\x80\x85'
            $'\xe2\x80\x86' $'\xe2\x80\x87' $'\xe2\x80\x88'
            $'\xe2\x80\x89' $'\xe2\x80\x8a' $'\xe2\x80\xa8'
            $'\xe2\x80\xa9' $'\xe2\x80\xaf' $'\xe2\x81\x9f' $'\xe3\x80\x80'
        )
        local -a _ctx_escapes=(
            %C2%85 %C2%A0 %E1%9A%80 %E2%80%80 %E2%80%81 %E2%80%82
            %E2%80%83 %E2%80%84 %E2%80%85 %E2%80%86 %E2%80%87 %E2%80%88
            %E2%80%89 %E2%80%8A %E2%80%A8 %E2%80%A9 %E2%80%AF %E2%81%9F %E3%80%80
        )
        local -i i
        for (( i=1; i<=${#_ctx_spaces}; i++ )); do
            _ctx_fields=("${(@)_ctx_fields//${_ctx_spaces[i]}/${_ctx_escapes[i]}}")
        done
    fi
    local _zacrs_file_option_key
    _zacrs_file_options
    REPLY="${$}:${_zacrs_candidate_cache_generation:-0}:${_ctx_fields[1]}:${_ctx_fields[2]}:${CURSOR:-${#LBUFFER}}:${_ctx_fields[3]}:${_ctx_fields[4]}:$_zacrs_file_option_key"
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

# Byte-length fields preserve multiline input without evaluation or encoders.
# Transport failure disables edits until the next line; malformed replies need reload.
_zacrs_abbreviation_transport_failed() {
    _zacrs_edit_transport_state=-2
    _zacrs_mark_daemon_unavailable
    return 1
}

# Write each argument as a byte count line followed by the value.
_zacrs_write_framed_fields() {
    local fd=$1 LC_ALL=C item
    shift
    for item in "$@"; do
        print -r -u $fd -- "${#item}"
        print -r -u $fd -- "$item"
    done
}

_zacrs_abbreviation_request() {
    local operation="$1" fd
    shift
    (( _zacrs_edit_transport_state > 0 )) || return 1
    (( _zacrs_daemon_available )) || { _zacrs_abbreviation_transport_failed; return 1; }
    if ! zsocket "$_zacrs_socket_path" 2>/dev/null; then
        _zacrs_abbreviation_transport_failed
        return 1
    fi
    fd=$REPLY
    local LC_ALL=C
    {
        print -r -u $fd -- "abbreviation $$ $operation $#"
        _zacrs_write_framed_fields $fd "$@"
        IFS= read -r -t 1 -u $fd _zacrs_abbreviation_header || { _zacrs_abbreviation_transport_failed; return 1; }
        [[ "$_zacrs_abbreviation_header" == DONE* ]] || { _zacrs_edit_transport_state=-1; return 1; }
        IFS= read -r -t 1 -u $fd _zacrs_abbreviation_apply || { _zacrs_abbreviation_transport_failed; return 1; }
        [[ "$_zacrs_abbreviation_header" == DONE* && "$_zacrs_abbreviation_apply" == APPLY* ]] || { _zacrs_edit_transport_state=-1; return 1; }
    } always {
        exec {fd}<&-
    }
}

# Native Zsh lexer facts. Unknown lexical contexts are represented explicitly.
# Caller declares local arrays reply and _zacrs_abbr_words.
_zacrs_abbreviation_words() {
    reply=(0 '' 0 0)
    _zacrs_abbr_words=( ${(z)BUFFER} ) 2>/dev/null || return 0
    local leading="${BUFFER%%[^[:space:]]*}" word="${_zacrs_abbr_words[1]:-}"
    local -i start=${#leading} end=$(( ${#leading} + ${#word} ))
    [[ -z "$word" || "${BUFFER[start+1,end]}" == "$word" ]] || return 0
    reply=(1 "$word" "$start" "$end")
}

# No candidate providers are called. Zsh's completion parser supplies context;
# its command words must cover the entire native lexical input before we opt in.
_zacrs_abbreviation_context_func() {
    compstate[insert]=''
    compstate[list]=''
    [[ "${compstate[context]}" == command ]] || return 0
    local -a native_words=( "${words[@]}" )
    # Zsh adds an empty current word at an insertion point between words.
    # Remove that native cursor marker, not a quoted empty argument.
    if (( ${#native_words} == ${#_zacrs_abbr_words} + 1 )) && [[ -z "${native_words[CURRENT]:-}" ]]; then
        native_words[CURRENT]=()
    fi
    (( ${#native_words} == ${#_zacrs_abbr_words} )) || return 0
    local head="$reply[2]"
    # Unquoting alone leaves substitutions intact. Require Zsh's literal
    # quoting to leave the head unchanged, independently of cursor context.
    [[ -n "$head" && "${(q)head}" == "$head" ]] || return 0
    (( ${reswords[(Ie)$head]} == 0 )) || return 0
    local -i i
    for ((i=1; i<=${#native_words}; i++)); do
        [[ "$native_words[i]" == "${_zacrs_abbr_words[i]}" ]] || return 0
    done
    _zacrs_abbr_valid=1
}

# File queries and their reusable snapshots require a top-level command editor.
_zacrs_filesystem_editor() {
    (( ${ZLE_RECURSIVE:-0} == 0 )) && [[ "${CONTEXT:-start}" == start && -z "${PREBUFFER:-}" ]]
}

# Only primary command editing may update the daemon's abbreviation session.
_zacrs_command_editor() {
    (( ZLE_RECURSIVE == 0 )) && [[ "${CONTEXT:-start}" == start ]]
}

_zacrs_expand_command_abbreviation() {
    # Recursive editors can be search/replacement text, not shell command input.
    _zacrs_command_editor || return 1
    local operation="${1:-accept}"
    local -a reply _zacrs_abbr_words
    _zacrs_abbreviation_words
    # Space only terminates the literal command word at the editing cursor.
    # Argument spaces need neither a completion query nor a daemon request.
    if [[ "$operation" == space ]]; then
        [[ "$reply[1]" == 1 && -n "$reply[2]" && $CURSOR == $reply[4] ]] || return 1
    fi
    local -i _zacrs_abbr_valid=0 _zacrs_abbr_context_only=1
    local original_buffer="$BUFFER" original_cursor=$CURSOR
    if [[ -z "${PREBUFFER:-}" && "$reply[1]" == 1 && -n "$reply[2]" ]]; then
        {
            # Query the command word, independent of the editing cursor. Inside
            # function parentheses Zsh can expose the whole line as plain words.
            CURSOR=$reply[4]
            # Complete the literal alias name; restore caller options immediately.
            () {
                setopt localoptions completealiases
                zle _zacrs_compsys 2>/dev/null || true
            }
        } always {
            [[ "$BUFFER" == "$original_buffer" ]] || BUFFER="$original_buffer"
            (( CURSOR == original_cursor )) || CURSOR=$original_cursor
        }
    fi
    local context="${(j: :)${(@Q)_zacrs_abbr_words}}"
    [[ "$BUFFER" == *[[:space:]] ]] && context+=" "
    local _zacrs_abbreviation_header _zacrs_abbreviation_apply
    _zacrs_abbreviation_request "$operation" "$BUFFER" "$reply[1]" "$reply[2]" "$reply[3]" "$reply[4]" "$_zacrs_abbr_valid" "$context" || return 1
    local literal_path_known=0 literal_path="" literal_suffix=""
    local result_code result_text chain execute restore_text cursor_offset selection_applied insert_trigger
    if ! _zacrs_parse_done_line "$_zacrs_abbreviation_header" ||
        ! _zacrs_parse_apply_line "$_zacrs_abbreviation_apply" || (( literal_path_known )); then
        _zacrs_edit_transport_state=-1
        return 1
    fi
    (( result_code == 0 )) || return 1
    # Enter placeholder edits and Space edits both invalidate the old suggestion.
    unset POSTDISPLAY
    BUFFER="$result_text"
    CURSOR=${cursor_offset:-${#BUFFER}}
    REPLY="$execute"
    [[ "$operation" == space ]] && REPLY="$insert_trigger"
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
