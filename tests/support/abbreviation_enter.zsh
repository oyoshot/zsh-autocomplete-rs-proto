set -eu

# Exercise the actual CLI and shell adapter, observing acceptance without
# executing commands from the test. Real ZLE keyboard input is checked separately.
zle() {
    if [[ "$1" == .accept-line ]]; then
        accepted="$BUFFER"
    elif [[ "$1" == accept-line ]]; then
        _zacrs_accept_line
    fi
    return 0
}
bindkey() { :; }
autoload() { :; }
add-zle-hook-widget() { :; }
add-zsh-hook() { :; }
# The CLI lookup needs no sockets. Keep real non-network Zsh modules available.
zmodload() {
    [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
    builtin zmodload "$@"
}
source "$ZACRS_PLUGIN"
unfunction zmodload
actual_binary="$ZACRS_BIN"

fail() { print -u2 -r -- "not ok: $* (buffer=${(qqq)BUFFER})"; exit 1; }
set_buffer() {
    _zacrs_reset_enter_selection
    PREBUFFER=""
    BUFFER="$1"
    CURSOR=${#BUFFER}
    LBUFFER="$BUFFER"
    RBUFFER=""
    accepted=""
    cursor_offset=""
}

unset ZACRS_ABBR_EXPAND_ON_ENTER
set_buffer gs
_zacrs_accept_line
[[ "$BUFFER" == gs && "$accepted" == gs ]] || fail 'disabled by default'

ZACRS_ABBR_EXPAND_ON_ENTER=1
set_buffer $'  gs\t-sb "日本語 a"  '
CURSOR=1
_zacrs_accept_line
[[ "$accepted" == $'  git status\t-sb "日本語 a"  ' ]] || fail 'preserve arguments and whitespace, even with cursor mid-line'

for input in g gss ./gs ../gs /tmp/gs '~/gs' '"gs"' "'gs'" '\gs' 'gs | cat' 'gs; gs' 'gs &! echo done' 'gs &| echo done' 'echo $(gs)' 'echo gs' 'A=1 gs' 'command gs' arg scoped ''; do
    set_buffer "$input"
    _zacrs_accept_line
    [[ "$BUFFER" == "$input" && "$accepted" == "$input" ]] || fail 'nonmatching/explicit shell syntax must pass through'
done

for input in 'gs() print hello' 'gs () print hello' 'gs( ) print hello' 'gs other() print hello' 'function gs print hello' 'gs() { print hello; }'; do
    set_buffer "$input"
    _zacrs_accept_line
    [[ "$BUFFER" == "$input" && "$accepted" == "$input" ]] || fail 'function declarations must not expand'
done

# Quoted parentheses are arguments, not function-declaration syntax.
set_buffer 'gs "()"'
_zacrs_accept_line
[[ "$accepted" == 'git status "()"' ]] || fail 'quoted parentheses must not block command expansion'

for prebuffer in $'cat <<\'EOF\'\n' $'print -r -- "\n'; do
    set_buffer gs
    PREBUFFER="$prebuffer"
    _zacrs_accept_line
    [[ "$BUFFER" == gs && "$accepted" == gs ]] || fail 'continuation input must remain literal'
done

set_buffer 'scoped -sb'
_zacrs_accept_line
[[ "$accepted" == 'git status -sb' ]] || fail 'when.command condition'

set_buffer next
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'only expand once'

set_buffer literal
_zacrs_accept_line
[[ "$accepted" == 'echo $(touch should-not-exist)' ]] || fail 'literal expansion'
[[ ! -e should-not-exist ]] || fail 'lookup evaluated shell code'

set_buffer multi
_zacrs_accept_line
[[ "$accepted" == $'echo one\necho two' ]] || fail 'multiline protocol'

set_buffer gcm
_zacrs_accept_line
[[ "$BUFFER" == "git commit -m ''" && -z "$accepted" && "$CURSOR" == 15 ]] || fail 'cursor placeholder must stay in editor'

for trigger in ssh connect; do
    set_buffer "$trigger"
    _zacrs_accept_line
    [[ "$BUFFER" == 'ssh user@' && -z "$accepted" && "$CURSOR" == 9 ]] || fail 'SSH placeholder must stay in editor'
    BUFFER+=host
    LBUFFER="$BUFFER"
    _zacrs_accept_line
    [[ "$accepted" == 'ssh user@host' ]] || fail 'edited placeholder must execute without expanding again'
    [[ -z "$_zacrs_enter_selected_command" ]] || fail 'completed placeholder must clear its exemption'
done

set_buffer ssh
_zacrs_accept_line
BUFFER=next
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'changing the expanded command must invalidate its exemption'

# A popup-selected native candidate must not turn into an abbreviation, and a
# selected abbreviation must not recursively expand its result.
set_buffer g
_zacrs_apply 1 2 'gs ' 0 0 '' 0
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == 'git status ' ]] || fail 'Space without a selection must allow Enter expansion'

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
LBUFFER="$BUFFER"
_zacrs_apply 3 2 'gs ' 0 0 '' 0
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == 'git status ' ]] || fail 'typed filter must replace an older explicit choice'

for selected in gs next; do
    set_buffer "$selected"
    cursor_offset=''
    _zacrs_apply ${#selected} 0 gs 0 1 '' 1
    [[ "$accepted" == gs ]] || fail 'explicit popup selection must win'

    set_buffer "$selected"
    _zacrs_apply ${#selected} 2 'gs ' 0 0 '' 1
    LBUFFER="$BUFFER"
    _zacrs_accept_line
    [[ "$accepted" == 'gs ' ]] || fail 'Space-selected command must survive deferred Enter'

    set_buffer "$selected"
    _zacrs_apply ${#selected} 2 'gs ' 0 0 '' 1
    BUFFER+='-sb'
    LBUFFER="$BUFFER"
    _zacrs_accept_line
    [[ "$accepted" == 'gs -sb' ]] || fail 'argument editing must preserve explicit command selection'
done

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
BUFFER=next
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'editing the command must invalidate the earlier selection'

set_buffer 'gs old'
_zacrs_apply 3 2 'new ' 0 0 '' 1
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == 'git status new ' ]] || fail 'selecting only an argument must not exempt the command'

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
_zacrs_send_break
BUFFER=gs
LBUFFER="$BUFFER"
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'aborted selection must not leak into the next command'

set_buffer gs
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'completed selections must not leak into a new edit'

set_buffer gs
ZACRS_BIN=false
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'lookup failure must keep the original command'

ZACRS_BIN="$actual_binary"
[[ ! -S "$XDG_RUNTIME_DIR/zacrs.sock" ]] || fail 'this test must not start a daemon'
set_buffer gs
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'expansion must work without a daemon'

print -r -- 'ok: Enter abbreviation expansion'
