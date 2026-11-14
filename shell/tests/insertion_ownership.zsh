#!/usr/bin/env zsh
# Real ZLE lifecycle test. Optional arguments: fsh|autosuggestions /path/to/plugin.
# Transport/edit policy is isolated here; abbreviation_shell covers the daemon.
set -eu
zmodload zsh/zpty

typeset test_dir="${0:A:h}"
typeset tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/zacrs-insertion.XXXXXX")"
trap 'zpty -d 2>/dev/null || true; rm -rf -- "$tmp_dir"' EXIT
export ZACRS_INSERT_PLUGIN="${ZACRS_INSERT_PLUGIN:-${test_dir:h}/zsh-autocomplete-rs.plugin.zsh}"
export ZACRS_INSERT_OTHER=${1:-model} ZACRS_INSERT_OTHER_PATH=${2:-}
export ZACRS_INSERT_ROOT=$tmp_dir
cat > "$tmp_dir/bin" <<'SH'
#!/bin/sh
if [ "$1 $2" = 'daemon ensure' ]; then
    if [ -f "$ZACRS_INSERT_ROOT/disabled" ]; then
        echo 'OK edit_context=2 enter_expansion=0 space_expansion=0'
    else
        echo 'OK edit_context=2 enter_expansion=0 space_expansion=1'
    fi
else
    exit 1
fi
SH
chmod +x "$tmp_dir/bin"
cat > "$tmp_dir/.zshrc" <<'RC'
HISTFILE="$ZACRS_INSERT_ROOT/history"
HISTSIZE=100
ZACRS_BIN="$ZACRS_INSERT_ROOT/bin"
FUNCNEST=100
bindkey -e
_test_ready() {
    print -s -- 'gs old-suggestion'
    print -r -- "ready:$(( ++_test_ready_count ))" > "$ZACRS_INSERT_ROOT/ready"
}
_test_probe() {
    print -r -- "$BUFFER|$CURSOR|$POSTDISPLAY" > "$ZACRS_INSERT_ROOT/state"
    print -rl -- "${region_highlight[@]}" > "$ZACRS_INSERT_ROOT/highlight"
}
_test_other() {
    case $ZACRS_INSERT_OTHER in
        model)
            # The builtin branch mirrors the failing binder in #166. Do not
            # hide exposed zacrs aliases: enumerate them as the plugin does.
            local widget fn
            for widget in self-insert ${(M)${(k)widgets}:#_zacrs_*}; do
                fn="_test_model_$widget"
                case ${widgets[$widget]:-} in
                    builtin) functions[$fn]="zle .${(q)widget} -- \"\$@\"" ;;
                    user:*) zle -N "_test_saved_$widget" "${widgets[$widget]#user:}"
                        functions[$fn]="zle _test_saved_${(q)widget} -- \"\$@\"" ;;
                    *) continue ;;
                esac
                zle -N "$widget" "$fn"
            done
            ;;
        fsh) source "$ZACRS_INSERT_OTHER_PATH" ;;
        autosuggestions)
            ZSH_AUTOSUGGEST_MANUAL_REBIND=1
            source "$ZACRS_INSERT_OTHER_PATH"
            unset ZSH_AUTOSUGGEST_USE_ASYNC
            _zsh_autosuggest_start
            ;;
    esac
}
_test_load() {
    source "$ZACRS_INSERT_PLUGIN"
    if (( ! ${_test_ready_registered:-0} )); then
        add-zle-hook-widget line-init _test_ready
        _test_ready_registered=1
    fi
    if (( ${+_test_kept_identity} )) &&
        [[ ${widgets[_test_kept_entry]} != $_test_kept_identity ||
           ${widgets[._zacrs_insert_delegate]} != $_test_kept_delegate ]]; then
        print -r -- changed > "$ZACRS_INSERT_ROOT/ownership"
    fi
    # Only transport and policy are substituted. Widget ownership, cleanup,
    # insertion, reload and dispatch all use production code.
    _zacrs_abbreviation_request() { return 0; }
    _zacrs_expand_command_abbreviation() {
        _zacrs_command_editor || return 1
        case $BUFFER in
            gs) BUFFER='git status'; CURSOR=${#BUFFER}; REPLY=1 ;;
            wrap) BUFFER='git status'; CURSOR=0; REPLY=0 ;;
            gcm) BUFFER="git commit -m ''"; CURSOR=15; REPLY=0 ;;
            connect) BUFFER='ssh user@'; CURSOR=${#BUFFER}; REPLY=0 ;;
            *) return 1 ;;
        esac
        POSTDISPLAY=''
    }
    _zacrs_line_pre_redraw() { print -r -- "$BUFFER" > "$ZACRS_INSERT_ROOT/drawn"; }
}
[[ $ZACRS_INSERT_ORDER == before ]] && _test_other
_test_load
[[ $ZACRS_INSERT_ORDER == after ]] && _test_other
zle -N _test_probe
bindkey '^Y' _test_probe
print -s -- 'gs old-suggestion'
_test_reload() {
    local original=${widgets[self-insert]} delegate=${widgets[._zacrs_insert_delegate]}
    local -A saved
    saved=( ${(kv)widgets[(R)user:_zacrs_insert]} )
    local count
    for count in {1..5}; do _test_load; done
    [[ ${widgets[self-insert]} == $original &&
       ${widgets[._zacrs_insert_delegate]} == $delegate ]] ||
        print -r -- changed > "$ZACRS_INSERT_ROOT/ownership"
    local widget
    for widget in ${(k)saved}; do
        [[ ${widgets[$widget]} == ${saved[$widget]} ]] ||
            print -r -- changed > "$ZACRS_INSERT_ROOT/ownership"
    done
}
_test_disable() {
    : > "$ZACRS_INSERT_ROOT/disabled"
    _test_load
}
_test_enable() {
    command rm -f -- "$ZACRS_INSERT_ROOT/disabled"
    _test_load
}
_test_capture() {
    zle -A self-insert _test_kept_entry
    _test_kept_identity=${widgets[_test_kept_entry]}
    _test_kept_delegate=${widgets[._zacrs_insert_delegate]}
}
_test_replace() {
    zle -N self-insert _test_replacement
    _test_load
}
_test_replacement() { zle .self-insert; }
for action in reload disable enable replace capture; do zle -N _test_$action; done
bindkey '^R' _test_reload
bindkey '^T' _test_disable
bindkey '^E' _test_enable
bindkey '^O' _test_replace
bindkey '^F' _test_capture
PROMPT='READY> '
RPROMPT=''
RC

typeset pty_output=''
drain_pty() {
    local chunk
    while zpty -r -t zacrs_insert chunk 2>/dev/null; do pty_output+="$chunk"; done
    return 0
}
wait_file() {
    local file=$1 expected=$2 i
    for (( i=0; i<300; i++ )); do
        drain_pty
        [[ -f $tmp_dir/$file && $(<"$tmp_dir/$file") == "$expected" ]] && return 0
        sleep 0.01
    done
    print -u2 -r -- "not ok: $ZACRS_INSERT_OTHER/$ZACRS_INSERT_ORDER $file expected ${(qqq)expected}"
    [[ ! -f $tmp_dir/$file ]] || cat "$tmp_dir/$file" >&2
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    return 1
}
start_shell() {
    HOME="$tmp_dir" ZDOTDIR="$tmp_dir" XDG_RUNTIME_DIR="$tmp_dir" \
        XDG_CONFIG_HOME="$tmp_dir/config" XDG_CACHE_HOME="$tmp_dir/cache" \
        zpty -b zacrs_insert zsh -d
}
observe() {
    : > "$tmp_dir/drawn"
    : > "$tmp_dir/state"
    zpty -w -n zacrs_insert "$1"
    wait_file drawn "$2"
    zpty -w -n zacrs_insert $'\x19'
}
typeset -a orders
if (( $# >= 3 )); then orders=("$3"); else orders=(before after); fi
for order in "${orders[@]}"; do
    export ZACRS_INSERT_ORDER=$order
    pty_output=''
    command rm -f -- "$tmp_dir/disabled" "$tmp_dir/ready" "$tmp_dir/state"
    start_shell
    wait_file ready ready:1
    # Separate keys also check that the other plugin's suggestion/highlight
    # state survives ordinary input, before the expansion changes that input.
    observe gs gs
    if [[ $ZACRS_INSERT_OTHER == autosuggestions ]]; then
        wait_file state 'gs|2| old-suggestion'
    else
        wait_file state 'gs|2|'
    fi
    observe ' ' 'git status '
    wait_file state 'git status |11|'
    if [[ $ZACRS_INSERT_OTHER == fsh ]]; then
        [[ -s "$tmp_dir/highlight" && $(<"$tmp_dir/highlight") == *[0-9]* ]] || {
            print -u2 -r -- "not ok: FSH produced no highlight after expansion"
            print -u2 -r -- "pty: ${(qqq)pty_output}"
            return 1
        }
    fi
    : > "$tmp_dir/state"
    observe $'\x15\x12gs ' 'git status '
    wait_file state 'git status |11|'
    : > "$tmp_dir/state"
    observe $'\x15\x14gs ' 'gs '
    if [[ $ZACRS_INSERT_OTHER == autosuggestions ]]; then
        wait_file state 'gs |3|old-suggestion'
    else
        wait_file state 'gs |3|'
    fi
    : > "$tmp_dir/state"
    observe $'\x15\x05gs ' 'git status '
    wait_file state 'git status |11|'
    : > "$tmp_dir/state"
    # Retaining an alias while the public entry is still ours must not reuse
    # its delegate on disable/re-enable. Marker edits consume the Space.
    observe $'\x15\x06\x14gs ' 'gs '
    if [[ $ZACRS_INSERT_OTHER == autosuggestions ]]; then
        wait_file state 'gs |3|old-suggestion'
    else
        wait_file state 'gs |3|'
    fi
    observe $'\x15\x05wrap ' 'git status'
    wait_file state 'git status|0|'
    observe $'\x01\x0bgcm ' "git commit -m ''"
    wait_file state "git commit -m ''|15|"
    observe $'\x01\x0bconnect ' 'ssh user@'
    wait_file state 'ssh user@|9|'
    observe $'\x15\x0fgs ' 'gs '
    wait_file state 'gs |3|'
    [[ ! -f "$tmp_dir/ownership" ]] || return 1
    drain_pty
    [[ $pty_output != *'No such widget'* && $pty_output != *'maximum nested'* ]] || return 1
    zpty -d zacrs_insert
    print -r -- "ok: $ZACRS_INSERT_OTHER $order preserves insertion, plugin state and reload ownership"
done
