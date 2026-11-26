# zsh-autocomplete-rs: candidate gathering (fallback)
#
# Fallback candidate source when compsys (_zacrs_compsys) returns no results.
# Uses command/alias/builtin/function tables; Rust owns filesystem fallback.
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

# Verify the native word-end facts captured before providers could call compset.
# No shell expression is evaluated. Unknown contexts do not request files.
_zacrs_filesystem_context() {
    local -i current=$1
    local literal="$2"
    shift 2
    local -a lexical=( "${(@z)BUFFER}" )
    # Require the native command words to cover the complete lexical input.
    local -a native=( "$@" )
    (( current <= ${#native} )) || return
    [[ "${native[current]}" == "$literal" ]] || return
    if (( ${#native} == ${#lexical} + 1 )) && [[ -z "$literal" ]]; then
        native[current]=()
    fi
    (( ${#native} == ${#lexical} )) || return
    # Quote each element before joining so spaces within words cannot hide a
    # different token boundary.
    [[ "${(j: :)${(@qq)native}}" == "${(j: :)${(@qq)lexical}}" ]] || return
    # Native words can also cover a reserved construct such as `for name`.
    # Only accept an unambiguous, literal command head.
    local head="${lexical[1]:-}" literal_head="${lexical[1]:-}"
    [[ "$literal_head" == '~/'* ]] && literal_head="${literal_head[3,-1]}"
    [[ -n "$head" && "$head" != *=* && ( -z "$literal_head" || "${(q)literal_head}" == "$literal_head" ) ]] || return
    (( ${reswords[(Ie)$head]} == 0 )) || return
    # The native word array can include a precommand modifier before the
    # command being completed. Leave those lines to native completion.
    [[ "$head" != (-|builtin|command|exec|nocorrect|noglob) ]] || return
    # COMPLETE_ALIASES can leave an aliased command head in both word arrays.
    if (( current > 1 )) && [[ -o aliases ]] && _zacrs_literal_alias_token "$head"; then
        return 1
    fi
    [[ -z "$literal" || "${LBUFFER[-${#literal},-1]}" == "$literal" ]] || return
    local check="$literal"
    [[ "$check" == '~/'* ]] && check="${check[3,-1]}"
    [[ -z "$check" || "${(q)check}" == "$check" ]] || return
    # Bare command words retain the shell command-table path. Explicit paths
    # and argument positions may also offer filesystem candidates.
    (( current > 1 )) || [[ "$literal" == */* ]] || return
    typeset -g _zacrs_filesystem_prefix="$literal" _zacrs_filesystem_lookup="$literal" _zacrs_filesystem_base="$_zacrs_filesystem_cwd"
    if [[ "$literal" == '~/'* ]]; then
        [[ "$_zacrs_filesystem_home" == /* ]] || return
        _zacrs_filesystem_base="$_zacrs_filesystem_home"
        _zacrs_filesystem_lookup="${literal[3,-1]}"
        # Repeated separators after ~/ are still relative to the home anchor.
        _zacrs_filesystem_lookup="${_zacrs_filesystem_lookup#"${_zacrs_filesystem_lookup%%[^/]*}"}"
    fi
    _zacrs_filesystem_valid=1
}

_zacrs_filesystem_query_current() {
    (( ${_zacrs_filesystem_valid:-0} )) || return 1
    _zacrs_file_context_matches "$_zacrs_filesystem_cwd" "$_zacrs_filesystem_home" "$_zacrs_filesystem_file_options" &&
        _zacrs_pwd_matches_cwd
}

# Add the same structured query to daemon and subprocess requests.
_zacrs_filesystem_args() {
    reply=()
    (( ${_zacrs_filesystem_requested:-0} )) || return 0
    reply=(--filesystem-cwd "$_zacrs_filesystem_base" --filesystem-prefix "$_zacrs_filesystem_lookup"
        --filesystem-insertion-prefix "$_zacrs_filesystem_prefix")
    [[ "$_zacrs_filesystem_file_options" == off:* ]] && reply+=(--filesystem-ignore-case)
    [[ "$_zacrs_filesystem_file_options" == off:off:* ]] && reply+=(--filesystem-ignore-parent-case)
    [[ "$_zacrs_filesystem_file_options" == *:on ]] && reply+=(--filesystem-include-hidden)
    [[ "$_zacrs_filesystem_prefix" == '~/'* ]] && reply+=(--filesystem-absolute-insertion)
    return 0
}

# Byte counts are local to this transport function; no subprocess is needed.
_zacrs_write_filesystem_query() {
    local fd=$1 LC_ALL=C item
    for item in "$_zacrs_filesystem_base" "$_zacrs_filesystem_lookup" "$_zacrs_filesystem_prefix"; do
        print -r -u $fd -- "${#item}"
        print -r -u $fd -- "$item"
    done
}
