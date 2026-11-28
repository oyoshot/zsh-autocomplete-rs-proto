# zsh-autocomplete-rs: candidate gathering (fallback)
#
# Fallback candidate source when compsys (_zacrs_compsys) returns no results.
# Uses file globs and command/alias/builtin/function tables directly.
# Format: text\tdescription\tkind

_zacrs_gather() {
    local lbuffer="$1"
    local prefix="${lbuffer##* }"
    # 空バッファ（コマンド未入力）のみスキップ
    if [[ -z "$prefix" && ! "$lbuffer" =~ [^[:space:]] ]]; then
        return
    fi

    # Determine if prefix is in command position (first word, or after | ; && ||)
    local is_first_word=0
    _zacrs_is_cmd_pos "$lbuffer" "$prefix" && is_first_word=1

    # File/directory candidates
    local f
    for f in ${prefix}*(N); do
        if [[ -d "$f" ]]; then
            printf '%s/\tdirectory\tdirectory\n' "$f"
        else
            printf '%s\tfile\tfile\n' "$f"
        fi
    done

    # Command-position candidates
    if (( is_first_word )); then
        local cmd
        for cmd in ${(k)commands[(I)${prefix}*]}; do
            printf '%s\tcommand\tcommand\n' "$cmd"
        done
        for cmd in ${(k)aliases[(I)${prefix}*]}; do
            printf '%s\talias\talias\n' "$cmd"
        done
        for cmd in ${(k)builtins[(I)${prefix}*]}; do
            printf '%s\tbuiltin\tbuiltin\n' "$cmd"
        done
        for cmd in ${(k)functions[(I)${prefix}*]}; do
            # Skip internal/private functions
            [[ "$cmd" == _* ]] && continue
            printf '%s\tfunction\tfunction\n' "$cmd"
        done
    fi
}

_zacrs_gather_command_rescue() {
    local prefix="$1"
    [[ ${#prefix} -lt 3 || "$prefix" == */* ]] && return

    local initial="${prefix[1]}"
    local name
    for name in ${(k)commands}; do
        [[ "${name[1]}" == "$initial" ]] || continue
        printf '%s\tcommand\tcommand_rescue\n' "$name"
    done
    for name in ${(k)aliases}; do
        [[ "${name[1]}" == "$initial" ]] || continue
        printf '%s\talias\talias_rescue\n' "$name"
    done
    for name in ${(k)builtins}; do
        [[ "${name[1]}" == "$initial" ]] || continue
        printf '%s\tbuiltin\tbuiltin_rescue\n' "$name"
    done
    for name in ${(k)functions}; do
        [[ "$name" == _* ]] && continue
        [[ "${name[1]}" == "$initial" ]] || continue
        printf '%s\tfunction\tfunction_rescue\n' "$name"
    done
}
