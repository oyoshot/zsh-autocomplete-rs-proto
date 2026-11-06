set -eu
TRAPZERR() { print -u2 -r -- "unexpected failure: ${funcfiletrace[*]}"; }

# Exercise the actual daemon and shell adapter, observing acceptance without
# executing commands from the test. Real ZLE keyboard input is checked separately.
zle() {
    if [[ "$1" == _zacrs_abbreviation_context || ( "$1" == (-C|-D) && "$2" == _zacrs_abbreviation_context ) ]]; then
        builtin zle "$@"
    elif [[ "$1" == .accept-line ]]; then
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
source "$ZACRS_PLUGIN"
actual_binary="$ZACRS_BIN"

fail() { print -u2 -r -- "not ok: $* (buffer=${(qqq)BUFFER})"; exit 1; }
set_buffer() {
    _zacrs_reset_enter_selection
    BUFFER="$1"
    CURSOR=${#BUFFER}
    CURSOR=${#BUFFER}
    RBUFFER=""
    accepted=""
    cursor_offset=""
}

ZACRS_ABBR_EXPAND_ON_ENTER=1 # Legacy variables must not override TOML.
_test_unexpected_cli() { print -r -- called >> "$HOME/unexpected-cli"; return 1; }
ZACRS_BIN=_test_unexpected_cli
set_buffer gs
_zacrs_accept_line
[[ "$BUFFER" == gs && "$accepted" == gs ]] || fail 'disabled by default'
[[ ! -e "$HOME/unexpected-cli" ]] || fail 'disabled Enter must not start a helper'
ZACRS_BIN="$actual_binary"

print -r -- $'\n[completion]\nexpand_abbreviation_on_enter = true' >> "$XDG_CONFIG_HOME/zacrs/config.toml"
source "$ZACRS_PLUGIN"
# Count actual adapter requests. Redraw and line-init are driven separately in
# the real ZLE test; acceptance itself must not observe or reset a second time.
typeset -a test_requests=()
functions[_test_original_abbreviation_request]=$functions[_zacrs_abbreviation_request]
_zacrs_abbreviation_request() {
    test_requests+=("$1")
    _test_original_abbreviation_request "$@"
}
set_buffer gs
test_requests=()
_zacrs_accept_line
[[ "${(j:,:)test_requests}" == accept && "$accepted" == 'git status' ]] || fail 'Enter must send only acceptance'
set_buffer gs
test_requests=()
_zacrs_apply 2 2 'gs ' 0 0 '' 1
[[ "${(j:,:)test_requests}" == select ]] || fail 'candidate application must send only selection'
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "${(j:,:)test_requests}" == select,accept && "$accepted" == 'gs ' ]] || fail 'selected Enter must not send a duplicate observation or reset'
functions[_zacrs_abbreviation_request]=$functions[_test_original_abbreviation_request]
unfunction _test_original_abbreviation_request
set_buffer $'  gs\t-sb "日本語 a"  '
CURSOR=1
_zacrs_accept_line
[[ "$accepted" == $'  git status\t-sb "日本語 a"  ' ]] || fail 'preserve arguments and whitespace, even with cursor mid-line'

for input in g gss ./gs ../gs /tmp/gs '~/gs' '"gs"' "'gs'" '\gs' 'gs | cat' 'gs; gs' 'gs &! echo done' 'gs &| echo done' 'echo $(gs)' 'echo gs' 'A=1 gs' 'command gs' if 'if gs' for function arg scoped ''; do
    set_buffer "$input"
    _zacrs_accept_line
    [[ "$BUFFER" == "$input" && "$accepted" == "$input" ]] || fail 'nonmatching/explicit shell syntax must pass through'
done

for input in 'gs() print hello' 'gs () print hello' 'gs( ) print hello' 'gs other() print hello' 'function gs print hello' 'gs() { print hello; }'; do
    set_buffer "$input"
    _zacrs_accept_line
    [[ "$BUFFER" == "$input" && "$accepted" == "$input" ]] || fail 'function declarations must not expand'
done

set_buffer 'gs>out'
_zacrs_accept_line
[[ "$accepted" == 'gs>out' ]] || fail 'unverified redirection context must pass through'

# Quoted parentheses are arguments, not function-declaration syntax.
set_buffer 'gs "()"'
_zacrs_accept_line
[[ "$accepted" == 'git status "()"' ]] || fail 'quoted parentheses must not block command expansion'

set_buffer 'gs $(touch should-not-exist)'
_zacrs_accept_line
[[ "$accepted" == 'git status $(touch should-not-exist)' && ! -e should-not-exist ]] || fail 'native context query must not evaluate arguments'
set_buffer 'gs() print hello'
CURSOR=1
if _zacrs_expand_command_abbreviation; then
    fail 'function declaration unexpectedly yielded a target'
fi
[[ "$BUFFER" == 'gs() print hello' && "$CURSOR" == 1 ]] || fail 'context query must restore editor state when no edit is available'

# Continuation input is exercised with real prompts in abbreviation_zle.zsh.

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
    CURSOR=${#BUFFER}
    _zacrs_accept_line
    [[ "$accepted" == 'ssh user@host' ]] || fail 'edited placeholder must execute without expanding again'
done

set_buffer ssh
_zacrs_accept_line
BUFFER=next
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'changing the expanded command must invalidate its exemption'

# A popup-selected native candidate must not turn into an abbreviation, and a
# selected abbreviation must not recursively expand its result.
set_buffer g
_zacrs_apply 1 2 'gs ' 0 0 '' 0
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'git status ' ]] || fail 'Space without a selection must allow Enter expansion'

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
CURSOR=${#BUFFER}
_zacrs_apply 3 2 'gs ' 0 0 '' 0
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'git status ' ]] || fail 'typed filter must replace an older explicit choice'

for selected in gs next; do
    set_buffer "$selected"
    cursor_offset=''
    _zacrs_apply ${#selected} 0 gs 0 1 '' 1
    [[ "$accepted" == gs ]] || fail 'explicit popup selection must win'

    set_buffer "$selected"
    _zacrs_apply ${#selected} 2 'gs ' 0 0 '' 1
    CURSOR=${#BUFFER}
    _zacrs_accept_line
    [[ "$accepted" == 'gs ' ]] || fail 'Space-selected command must survive deferred Enter'

    set_buffer "$selected"
    _zacrs_apply ${#selected} 2 'gs ' 0 0 '' 1
    BUFFER+='-sb'
    CURSOR=${#BUFFER}
    _zacrs_accept_line
    [[ "$accepted" == 'gs -sb' ]] || fail 'argument editing must preserve explicit command selection'
done

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
BUFFER=next
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'editing the command must invalidate the earlier selection'

for intermediate in g gsx; do
    set_buffer gs
    _zacrs_apply 2 2 'gs ' 0 0 '' 1
    BUFFER="$intermediate"
    _zacrs_update_enter_selection
    BUFFER='gs '
    CURSOR=${#BUFFER}
    _zacrs_accept_line
    [[ "$accepted" == 'git status ' ]] || fail 'retyping the selected command must not restore its exemption'
done

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
LBUFFER='gs old'
BUFFER="$LBUFFER"
_zacrs_apply 3 2 'new ' 0 0 '' 1
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'gs new ' ]] || fail 'selecting an argument must retain an earlier command choice'

for suffix in '; echo hello' '| cat' '|| echo fallback' '& echo done' '&& echo done' '&! echo done' '&| echo done' '() print hello'; do
    set_buffer gs
    _zacrs_apply 2 2 'gs ' 0 0 '' 1
    BUFFER="gs$suffix"
    _zacrs_update_enter_selection
    BUFFER=gs
    CURSOR=${#BUFFER}
    _zacrs_accept_line
    [[ "$accepted" == gs ]] || fail 'separator edits must preserve the chosen command'
done

set_buffer 'gs old'
_zacrs_apply 3 2 'new ' 0 0 '' 1
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'git status new ' ]] || fail 'selecting only an argument must not exempt the command'

set_buffer gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
_zacrs_send_break
BUFFER=gs
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'aborted selection must not leak into the next command'

set_buffer gs
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'completed selections must not leak into a new edit'

set_buffer gs
_zacrs_daemon_available=0
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'daemon failure must keep the original command'
# A fallback choice is not recorded while transport is down. Reconnecting to
# the same daemon must not reactivate its older, unprotected editing session.
LBUFFER=gs
_zacrs_apply 2 2 'gs ' 0 0 '' 1
_zacrs_daemon_available=0
_zacrs_daemon_next_retry=0
_zacrs_maybe_retry_daemon
(( _zacrs_daemon_available == 1 )) || fail 'daemon retry did not reconnect'
(( _zacrs_enter_enabled == -1 )) || fail 'daemon retry cleared the failure latch'
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'gs ' ]] || fail 'reconnection must preserve a fallback selection'
set_buffer gs
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'new lines must not clear the failure latch'


source "$ZACRS_PLUGIN"
set_buffer gs
_zacrs_daemon_available=0
_zacrs_apply 2 2 'gs ' 0 0 '' 1
_zacrs_daemon_available=1
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == 'gs ' ]] || fail 'failed state update must preserve an explicit choice'

source "$ZACRS_PLUGIN"
set_buffer gs
_zacrs_accept_line
[[ "$accepted" == 'git status' ]] || fail 'expansion must recover after plugin reload'

# Argument bytes are not needed for a native word-identity observation.
set_buffer gs
_zacrs_apply 2 2 $'gs \xff' 0 0 '' 1
BUFFER=gs
CURSOR=${#BUFFER}
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'removing the invalid filename must preserve the native command'
# Acceptance still requires a decodable buffer for edits; failure is explicit.
set_buffer $'gs \xff'
_zacrs_accept_line
(( _zacrs_enter_enabled == -1 )) || fail 'undecodable acceptance must disable expansion'
source "$ZACRS_PLUGIN"

config_text=$(<"$XDG_CONFIG_HOME/zacrs/config.toml")
print -r -- "${config_text/expand_abbreviation_on_enter = true/expand_abbreviation_on_enter = false}" > "$XDG_CONFIG_HOME/zacrs/config.toml"
set_buffer gs
_zacrs_accept_line
[[ "$accepted" == gs ]] || fail 'TOML changes must disable expansion on the next Enter'

print -r -- 'ok: Enter abbreviation expansion'
