cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
# Too few rows for the cursor line, the frame and a candidate, or too few
# columns for a readable one.
stty rows "$ZACRS_TEST_ROWS" cols "$ZACRS_TEST_COLS"
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

wait_cursor_query() {
    wait_output 'the plugin to ask for the cursor position' $'\e[6n'
}
probe() {
    : > "$HOME/probe"
    send_keys $'\x19'
    wait_file probe "ls:2:0:$ZACRS_TEST_DAEMON"
}

start_shell

if [[ "$ZACRS_TEST_FITS" == 1 ]]; then
    # The smallest terminal that fits: both size checks must let it through.
    send_keys ls
    wait_cursor_query
    wait_output 'a popup in a terminal that fits' '┌' '└'
    print -r -- 'ok: the smallest terminal that fits shows the popup'
    exit 0
fi

# Typing must neither show the automatic popup nor do the work for one.
send_keys ls
# The first probe may arrive as type-ahead, which defers the redraw hook. The
# second one runs only after the hook has seen the final buffer.
probe
probe
assert_no_popup 'while typing'
[[ "$pty_output" != *$'\e[6n'* ]] || fail 'asked for the cursor position while typing'

# Tab cannot show a popup either, so it hands over to native completion.
send_keys $'\t'
# The engine decides this one: the plugin asks for the cursor position and
# then gets "no candidates" back.
wait_cursor_query
wait_file native ls
probe
assert_no_popup 'on Tab'

print -r -- 'ok: a terminal too small for the popup falls back to native completion'
