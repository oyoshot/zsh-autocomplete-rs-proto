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
_test_ready() {
    print -r -- "$TTY" > "$HOME/tty"
    print -r -- ready > "$HOME/ready"
}
add-zle-hook-widget line-init _test_ready
PROMPT='> '
RPROMPT=''
ZSH

start_shell
case "$ZACRS_TEST_CASE" in
    grow)
        # The automatic popup recorded the column before Zsh drew "s", one
        # column left of where the cursor is once the line is drawn. A taller
        # terminal redraws the popup in place, and the redraw must return the
        # cursor to where Zsh left it, not to the recorded column, or Zsh
        # draws the next key over the "s".
        send_keys ls
        wait_output 'the automatic popup' '┌' '┘'
        pty_output=''
        stty rows 30 < "$(<"$HOME/tty")"
        wait_output 'the redrawn popup' '┌' '┘'
        wait_output 'the cursor restored after the redraw' $'\e8'
        [[ "${pty_output##*'┘'}" == *$'\e8'* ]] ||
            fail 'the redraw left the cursor at the recorded column'
        ;;
    shrink)
        # The cursor is on row 20. The popup leaves room for max_visible = 10
        # rows below it, so it scrolls the screen by 8 rows and the cursor
        # ends on row 12.
        dsr_row=20
        send_keys ls
        wait_output 'the automatic popup' '┌' '┘' $'\e8\e[8A'
        # In 10 rows the cursor stays on the last row, 10, and the redraw
        # scrolls by 9 rows to leave room for max_visible = 7 rows: the
        # cursor goes up by 9. It must not also go up by the 2 rows that row
        # 12 lies past the new last row.
        pty_output=''
        stty rows 10 < "$(<"$HOME/tty")"
        wait_output 'the end of the redraw' '┌' '┘' $'\e[?2026l'
        [[ "${pty_output##*'┘'}" == *$'\e8\e[9A'* ]] ||
            fail "the redraw moved the cursor to the wrong row: ${(qqq)${pty_output##*'┘'}}"
        ;;
esac

print -r -- "ok: a resize redraw keeps the cursor where Zsh left it ($ZACRS_TEST_CASE)"
