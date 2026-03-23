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

    # Expose any pre-DSR keystrokes so callers can recover them
    _zacrs_cursor_stale="${_buf%%$'\e'*}"

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
