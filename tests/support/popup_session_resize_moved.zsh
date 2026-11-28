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
send_keys ls
wait_output 'the automatic popup' '┌' '└'

pty_output=''
send_keys $'\t'
wait_output 'the popup session to draw its first frame' $'\e[7m'

# The cursor is on row 2 and the popup starts on row 3. A taller terminal
# that pulls scrollback in above them moves both down 4 rows, and reports
# the cursor on row 6.
dsr_row=6
case "$ZACRS_TEST_CASE" in
    typeahead) dsr_typeahead=$'\e[B' ;;
    silent) dsr_silent=1 ;;
esac
pty_output=''
stty rows 30 < "$(<"$HOME/tty")"

if [[ "$ZACRS_TEST_CASE" == silent ]]; then
    # Without a reply the session redraws where the cursor was.
    # The popup keeps rows 3 to 7 instead of moving to rows 7 to 11.
    wait_output 'the redraw in place' $'\e[6n' $'\e[3;' '┌'
    [[ "$pty_output" != *$'\e[11;'* ]] || fail 'moved the popup without a reply'
    print -r -- 'ok: a resize without a cursor reply redraws the popup in place'
    exit 0
fi

# The popup moves from rows 3 to 7 to rows 7 to 11.
wait_output 'the redraw below the moved cursor' $'\e[6n' $'\e[7;' $'\e[11;' '┌'
[[ "$pty_output" != *$'\e[3;'* ]] || fail 'drew on the rows the popup left'

# The selected row is drawn in reverse video. Down, typed before the reply,
# moves the selection from "ls" to "lsblk" after the redraw.
expected=ls
[[ "$ZACRS_TEST_CASE" == typeahead ]] && expected=lsblk
for attempt in {1..300}; do
    drain_pty
    selected="${pty_output##*$'\e[7m'}"
    selected="${${selected%%$'\e[27m'*}//$'\e['<->m/}"
    [[ "${selected%% *}" == "$expected" ]] && break
    sleep 0.01
done
[[ "${selected%% *}" == "$expected" ]] || fail "selected ${(qqq)selected}, not $expected"

print -r -- "ok: a resize redraws the popup where the terminal moved it ($ZACRS_TEST_CASE)"
