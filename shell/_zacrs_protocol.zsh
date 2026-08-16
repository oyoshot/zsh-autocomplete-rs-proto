# Helpers for decoding and parsing the shell-facing completion result protocol.

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
    local metadata="$apply_line"
    chain=0
    execute=0
    restore_text=""
    cursor_offset=""

    if [[ "$metadata" == *" restore_hex="* ]]; then
        local restore_hex="${metadata#* restore_hex=}"
        metadata="${metadata%% restore_hex=*}"
        _zacrs_decode_hex_to_REPLY "$restore_hex"
        restore_text="$REPLY"
    elif [[ "$metadata" == *" restore="* ]]; then
        restore_text="${metadata#* restore=}"
        metadata="${metadata%% restore=*}"
    fi

    local token
    for token in ${(s: :)metadata}; do
        [[ "$token" == "chain=1" ]] && chain=1
        [[ "$token" == "execute=1" ]] && execute=1
        [[ "$token" == cursor_offset=<-> ]] && cursor_offset="${token#cursor_offset=}"
        if [[ "$token" == text_hex=* ]]; then
            _zacrs_decode_hex_to_REPLY "${token#text_hex=}"
            result_text="$REPLY"
        fi
    done
}
