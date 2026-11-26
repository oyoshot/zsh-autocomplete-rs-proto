set -eu
zmodload zsh/zpty

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
# Three rows leave no room for the cursor line, the frame and a candidate.
stty rows 3 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    zmodload() {
        [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
        builtin zmodload "$@"
    }
fi
source "$ZACRS_PLUGIN"
# Keep candidate generation deterministic; rendering and ZLE are real.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    _zacrs_captured=($'ls\tcommand' $'lsblk\tcommand' $'lscpu\tcommand')
}
zle -N _zacrs_compsys
# Stand in for native completion so the fallback is observable.
_test_native() { print -r -- "$BUFFER" > "$HOME/native"; }
zle -N expand-or-complete _test_native
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
_test_probe() {
    print -r -- "$BUFFER:$CURSOR:$_zacrs_popup_visible:$_zacrs_daemon_available" > "$HOME/probe"
}
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

cleanup() {
    zpty -d zacrs_small 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_small chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            zpty -w -n zacrs_small $'\e[2;5R'
            dsr_pending="${dsr_pending#*$'\e[6n'}"
        done
        dsr_pending="${dsr_pending[-3,-1]}"
    done
    return 0
}
fail() {
    print -u2 -r -- "not ok: $*"
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    exit 1
}
wait_file() {
    local file="$1" expected="$2" attempt
    for attempt in {1..300}; do
        drain_pty
        [[ -f "$HOME/$file" && "$(<"$HOME/$file")" == "$expected" ]] && return 0
        sleep 0.01
    done
    fail "waiting for $file = ${(qqq)expected}"
}
wait_cursor_query() {
    local attempt
    for attempt in {1..300}; do
        drain_pty
        [[ "$pty_output" == *$'\e[6n'* ]] && return 0
        sleep 0.01
    done
    fail 'waiting for the plugin to ask for the cursor position'
}
assert_no_popup() {
    drain_pty
    [[ "$pty_output" != *'┌'* && "$pty_output" != *'└'* ]] || fail "drew a popup frame $1"
}

zpty -b zacrs_small zsh -di
wait_file ready ready

# Typing triggers the automatic popup, which must stay hidden.
zpty -w -n zacrs_small ls
# The plugin asks for the cursor position right before it renders. ZLE runs
# the probe only after that redraw hook returns.
wait_cursor_query
zpty -w -n zacrs_small $'\x19'
wait_file probe "ls:2:0:$ZACRS_TEST_DAEMON"
assert_no_popup 'while typing'

# Tab cannot show a popup either, so it hands over to native completion.
zpty -w -n zacrs_small $'\t'
wait_file native ls
: > "$HOME/probe"
zpty -w -n zacrs_small $'\x19'
wait_file probe "ls:2:0:$ZACRS_TEST_DAEMON"
assert_no_popup 'on Tab'

print -r -- 'ok: a terminal too small for the popup falls back to native completion'
