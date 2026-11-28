# Helpers for decoding and parsing the shell-facing completion result protocol.

_zacrs_parse_done_line() {
    local header="$1"
    [[ "$header" == 'DONE '* ]] || return 1
    result_code="${${(s: :)header}[2]}"
    [[ "$result_code" == <-> ]] || return 1
    # <-> matches decimal digits without requiring the user's EXTENDED_GLOB.
    result_text="${header#DONE <-> }"
    [[ "$result_text" == "$header" ]] && result_text=""
    return 0
}

# Hex-encode the bytes of $1 into REPLY. Every daemon render sends one, so
# input of up to 2048 bytes is encoded in the shell: starting od and tr costs
# more than the rest of the request. Beyond about 3000 bytes they are faster.
_zacrs_encode_hex_to_REPLY() {
    # Bytes rather than characters, no splitting or globbing of the unquoted
    # expansion, and decimal arithmetic output, whatever the caller has set.
    emulate -L zsh
    unsetopt multibyte
    if (( ${#1} > 2048 )); then
        REPLY=$(print -rn -- "$1" | od -An -tx1 -v | tr -d ' \n')
        return
    fi
    local char
    REPLY=""
    for char in ${(s::)1}; do
        REPLY+=${(l:2::0:)$(( [##16] #char ))}
    done
    REPLY=${(L)REPLY}
}

_zacrs_decode_hex_to_REPLY() {
    local hex="$1"
    REPLY=""
    [[ -z "$hex" ]] && return 0
    (( ${#hex} % 2 == 0 )) || return 1

    local escaped="" i pair
    for (( i = 1; i <= ${#hex}; i += 2 )); do
        pair="${hex[i,i+1]}"
        [[ "$pair" == [[:xdigit:]][[:xdigit:]] ]] || return 1
        escaped+="\\x${pair}"
    done
    printf -v REPLY '%b' "$escaped"
}

_zacrs_parse_apply_line() {
    local apply_line="$1"
    [[ "$apply_line" == 'APPLY '* ]] || return 1
    local metadata="$apply_line"
    local -i path_seen=0 suffix_seen=0
    literal_path_known=0
    literal_path=""
    literal_suffix=""
    chain=0
    execute=0
    selection_applied=0
    restore_text=""
    cursor_offset=""
    insert_trigger=0

    if [[ "$metadata" == *" restore_hex="* ]]; then
        local restore_hex="${metadata#* restore_hex=}"
        metadata="${metadata%% restore_hex=*}"
        _zacrs_decode_hex_to_REPLY "$restore_hex" || return 1
        restore_text="$REPLY"
    elif [[ "$metadata" == *" restore="* ]]; then
        restore_text="${metadata#* restore=}"
        metadata="${metadata%% restore=*}"
    else
        # Command substitution removes the newline. The final field marks
        # the end of a complete record, even when its value is empty.
        return 1
    fi

    local token
    for token in ${(s: :)metadata}; do
        if [[ "$token" == path_hex=* ]]; then
            (( path_seen++ == 0 )) || return 1
            _zacrs_decode_hex_to_REPLY "${token#path_hex=}" || return 1
            literal_path="$REPLY"
            literal_path_known=1
        elif [[ "$token" == suffix_hex=* ]]; then
            (( suffix_seen++ == 0 )) || return 1
            _zacrs_decode_hex_to_REPLY "${token#suffix_hex=}" || return 1
            literal_suffix="$REPLY"
        fi
        [[ "$token" == "chain=1" ]] && chain=1
        [[ "$token" == "execute=1" ]] && execute=1
        [[ "$token" == "insert_trigger=1" ]] && insert_trigger=1
        [[ "$token" == "selection_applied=1" ]] && selection_applied=1
        [[ "$token" == cursor_offset=<-> ]] && cursor_offset="${token#cursor_offset=}"
        if [[ "$token" == text_hex=* ]]; then
            _zacrs_decode_hex_to_REPLY "${token#text_hex=}" || return 1
            result_text="$REPLY"
        fi
    done
    (( path_seen == suffix_seen )) || return 1
    if (( path_seen )); then
        [[ -n "$literal_path" && "$literal_path" != *$'\0'* ]] || return 1
    fi
    return 0
}
