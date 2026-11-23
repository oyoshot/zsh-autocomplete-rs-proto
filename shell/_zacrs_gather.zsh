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

# Capture only native, literal word-end context before providers can call compset.
# No shell expression is evaluated. Unknown contexts do not request files.
_zacrs_filesystem_context() {
    typeset -gi _zacrs_filesystem_valid=0
    typeset -g _zacrs_filesystem_prefix="" _zacrs_filesystem_lookup=""
    typeset -g _zacrs_filesystem_home="${HOME:-}" _zacrs_filesystem_cwd="$PWD"
    _zacrs_filesystem_editor || return
    [[ ${compstate[context]} == command && -z "$SUFFIX$QIPREFIX$QISUFFIX" ]] || return
    local -a lexical=( "${(@z)BUFFER}" )
    # Require the native command words to cover the complete lexical input.
    local -a native=( "${words[@]}" )
    if (( ${#native} == ${#lexical} + 1 )) && [[ -z "${native[CURRENT]}" ]]; then
        native[CURRENT]=()
    fi
    (( ${#native} == ${#lexical} )) || return
    local -i i
    for ((i=1; i<=${#native}; i++)); do
        [[ "$native[i]" == "$lexical[i]" ]] || return
    done
    # Native words can also cover a reserved construct such as `for name`.
    # Only accept an unambiguous, literal command head.
    local head="${lexical[1]:-}" literal_head="${lexical[1]:-}"
    [[ "$literal_head" == '~/'* ]] && literal_head="${literal_head[3,-1]}"
    [[ -n "$head" && "$head" != *=* && "${(q)literal_head}" == "$literal_head" ]] || return
    (( ${reswords[(Ie)$head]} == 0 )) || return
    local literal="${IPREFIX}${PREFIX}"
    [[ -z "$literal" || "${LBUFFER[-${#literal},-1]}" == "$literal" ]] || return
    local check="$literal"
    [[ "$check" == '~/'* ]] && check="${check[3,-1]}"
    [[ -z "$check" || "${(q)check}" == "$check" ]] || return
    # Bare command words retain the shell command-table path. Explicit paths
    # and argument positions may also offer filesystem candidates.
    (( CURRENT > 1 )) || [[ "$literal" == */* ]] || return
    _zacrs_filesystem_prefix="$literal"
    _zacrs_filesystem_lookup="$literal"
    if [[ "$literal" == '~/'* ]]; then
        _zacrs_filesystem_lookup="${_zacrs_filesystem_home}/${literal[3,-1]}"
    fi
    _zacrs_filesystem_valid=1
}

# Add the same structured query to daemon and subprocess requests.
_zacrs_filesystem_args() {
    reply=()
    (( ${_zacrs_filesystem_requested:-0} )) || return 0
    reply=(--filesystem-cwd "$_zacrs_filesystem_cwd" --filesystem-prefix "$_zacrs_filesystem_lookup"
        --filesystem-insertion-prefix "$_zacrs_filesystem_prefix")
}

# Byte counts are local to this transport function; no subprocess is needed.
_zacrs_write_filesystem_query() {
    local fd=$1 LC_ALL=C item
    for item in "$_zacrs_filesystem_cwd" "$_zacrs_filesystem_lookup" "$_zacrs_filesystem_prefix"; do
        print -r -u $fd -- "${#item}"
        print -r -u $fd -- "$item"
    done
}
