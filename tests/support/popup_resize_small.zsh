cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget
source "$ZACRS_PLUGIN"
# Keep candidate generation deterministic; rendering and ZLE are real.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    _zacrs_captured=($'ls\tcommand' $'lsblk\tcommand' $'lscpu\tcommand')
}
zle -N _zacrs_compsys
# Report when the Tab widget, and with it the popup session, has returned.
_test_tab() {
    zle _zacrs_complete_popup
    print -r -- done > "$HOME/tab-done"
}
zle -N _test_tab
bindkey '^I' _test_tab
_test_ready() {
    print -r -- "$TTY" > "$HOME/tty"
    print -r -- ready > "$HOME/ready"
}
add-zle-hook-widget line-init _test_ready
_test_probe() {
    print -r -- "$BUFFER:$CURSOR:$_zacrs_popup_visible" > "$HOME/probe"
}
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

# Give output that must not appear a moment to arrive.
settle() {
    local attempt
    for attempt in {1..20}; do
        drain_pty
        sleep 0.01
    done
}

start_shell
send_keys ls
wait_output 'the automatic popup' '┌' '└'

# Tab opens the interactive session on top of the visible popup. The session
# draws its first frame, with a selected row, only after the client has
# installed its resize handler.
pty_output=''
send_keys $'\t'
wait_output 'the popup session to draw its first frame' $'\e[7m'
[[ ! -e "$HOME/tab-done" ]] || fail 'the popup session ended before the resize'

# Shrink the terminal under the open session.
pty_output=''
stty ${=ZACRS_TEST_SHRINK} < "$(<"$HOME/tty")"
wait_file tab-done done
settle
assert_no_popup 'after the resize'
[[ "$pty_output" == *$'\e[2K'* ]] || fail 'did not clear the popup rows'

# The session ended as a cancel: the buffer is intact and no popup is left.
send_keys $'\x19'
wait_file probe 'ls:2:0'
settle
assert_no_popup 'after the session'

print -r -- 'ok: shrinking the terminal under an open popup closes it'
