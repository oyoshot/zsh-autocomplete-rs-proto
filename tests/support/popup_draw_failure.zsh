cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget
source "$ZACRS_PLUGIN"
# Keep candidate generation deterministic; rendering and ZLE are real.
# "l" shows 10 of 13 rows; "lb" shows one.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    _zacrs_captured=($'lb\tcommand')
    local i
    for i in {01..12}; do
        _zacrs_captured+=("la$i"$'\tcommand')
    done
}
zle -N _zacrs_compsys
# Fail every frame read once the test asks for it, as a daemon that stalls
# mid-frame would.
sysread() {
    [[ -e "$HOME/fail" ]] && return 4
    builtin sysread "$@"
}
_test_ready() {
    print -r -- "$TTY" > "$HOME/tty"
    print -r -- ready > "$HOME/ready"
}
add-zle-hook-widget line-init _test_ready
_test_probe() {
    print -r -- "$BUFFER:$_zacrs_popup_visible:$_zacrs_daemon_available" > "$HOME/probe"
}
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

# The cursor position reply puts the cursor on row 2, so the first popup
# takes 1-based rows 3 to 14: a frame around 10 rows.
typeset -r last_popup_row=$'\e[14;1H\e[2K'

start_shell
send_keys l
wait_output 'the first popup' '┌' ' 1–10 / 13 ' '┘'
[[ "$ZACRS_TEST_CASE" == shrink ]] || : > "$HOME/fail"
pty_output=''

case "$ZACRS_TEST_CASE" in
    shrink)
        # In 13 rows the popup's old row 14 no longer exists, and a terminal
        # would apply a clear of it to row 13, which may hold the edit line.
        # Row 13 itself is still cleared before the redraw.
        stty rows 13 < "$(<"$HOME/tty")"
        wait_output 'the redrawn popup' '┌' '┘' $'\e[13;1H\e[2K'
        [[ "$pty_output" != *$'\e[14;1H'* ]] || fail 'cleared a row below the terminal'
        print -r -- "ok: a resize redraw clears no row below the terminal"
        exit 0
        ;;
    render)
        # The next frame would be 3 rows. Its header arrives, then the read
        # fails: the 12 rows on screen must still be cleared.
        send_keys b
        wait_output 'the old popup rows to be cleared' "$last_popup_row"
        : > "$HOME/probe"
        send_keys $'\x19'
        wait_file probe 'lb:0:0'
        ;;
    resize)
        # A resize redraw that fails clears the popup and stops using the
        # daemon, as a failed render does.
        stty rows 30 < "$(<"$HOME/tty")"
        wait_output 'the old popup rows to be cleared' "$last_popup_row"
        : > "$HOME/probe"
        send_keys $'\x19'
        wait_file probe 'l:0:0'
        ;;
esac
assert_no_popup 'after the failed read'

print -r -- "ok: a failed frame read clears the popup on screen ($ZACRS_TEST_CASE)"
