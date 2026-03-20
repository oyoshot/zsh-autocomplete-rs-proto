# zsh-autocomplete-rs: utility functions

# Get terminal cursor position (0-indexed row, col)
# Sets: cursor_row, cursor_col
_zacrs_get_cursor_pos() {
    local pos
    echo -ne '\e[6n' > /dev/tty
    IFS='' read -t 1 -rs -d R pos < /dev/tty
    pos="${pos#*\[}"
    local row_str="${pos%;*}"
    local col_str="${pos#*;}"
    cursor_row=$(( row_str - 1 ))
    cursor_col=$(( col_str - 1 ))
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
