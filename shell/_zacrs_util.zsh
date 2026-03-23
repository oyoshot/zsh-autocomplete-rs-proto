# zsh-autocomplete-rs: utility functions

# Get terminal cursor position (0-indexed row, col)
# Sets: cursor_row, cursor_col
_zacrs_get_cursor_pos() {
    local pos
    # Drain buffered input so it doesn't corrupt the DSR response.
    # Auto-trigger callers are guarded by PENDING checks and never
    # reach here with pending keystrokes; Tab callers accept the
    # drain as pre-existing behaviour.
    read -t 0 -rs -k 256 _ < /dev/tty 2>/dev/null
    echo -ne '\e[6n' > /dev/tty
    IFS='' read -t 1 -rs -d R pos < /dev/tty

    pos="${pos#*\[}"
    local row_str="${pos%;*}"
    local col_str="${pos#*;}"

    # Validate: must be positive integers
    if [[ "$row_str" =~ ^[0-9]+$ && "$col_str" =~ ^[0-9]+$ \
          && "$row_str" -ge 1 && "$col_str" -ge 1 ]]; then
        cursor_row=$(( row_str - 1 ))
        cursor_col=$(( col_str - 1 ))
    else
        # Fallback: bottom of terminal, column 0
        cursor_row=$(( LINES - 1 ))
        cursor_col=0
        _zacrs_dbg "get_cursor_pos: DSR failed, fallback (raw='$pos')"
    fi

    # Clamp to terminal bounds
    (( cursor_row >= LINES )) && cursor_row=$(( LINES - 1 ))
    (( cursor_col >= COLUMNS )) && cursor_col=$(( COLUMNS - 1 ))

    _zacrs_dbg "get_cursor_pos: row=$cursor_row col=$cursor_col"
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
