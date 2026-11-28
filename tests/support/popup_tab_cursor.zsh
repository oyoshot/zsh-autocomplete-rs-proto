cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
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
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
PROMPT='> '
RPROMPT=''
ZSH

start_shell
if [[ "$ZACRS_TEST_CASE" == continuation ]]; then
    # A continuation line is not top-level command editing, so its popup has
    # no reusable candidates. The session must still redraw over it.
    rm -f "$HOME/ready"
    send_keys $'echo \\\r'
    wait_file ready ready
fi
# The automatic popup asks for the cursor before Zsh draws the key just
# typed, so a terminal reports the column before "s": 4.
dsr_col=4
# Start from here: the line before a continuation may have drawn a popup too.
drain_pty
pty_output=''
send_keys ls
# The title is the query, so a late frame for an earlier line does not count.
wait_output 'the automatic popup' ' ls ' '┘'

if [[ "$ZACRS_TEST_CASE" == (silent|continuation) ]]; then
    # A terminal that does not answer leaves the popup where it is, at row 3
    # and column 4, instead of a fallback position at the bottom left.
    dsr_silent=1
    pty_output=''
    send_keys $'\t'
    wait_output 'the popup session in place' $'\e[7m' $'\e[3;4H'
    print -r -- "ok: a popup session stays in place without a cursor reply ($ZACRS_TEST_CASE, daemon=$ZACRS_TEST_DAEMON)"
    exit 0
fi

# By the time Tab runs, "ls" is drawn and the cursor is in column 5. The
# session redraws the word at the cursor minus its width: column 3.
dsr_col=5
pty_output=''
send_keys $'\t'
wait_output 'the popup session' $'\e[7m'
# The session's popup follows the cursor one column right of the automatic
# popup, so it erases the old popup's left column on the rows it covers.
wait_output 'the old left column to be erased' $'\e[3;4H\e[1K'
send_keys $'\e[B'
wait_output 'the redrawn word' $'\e[2;3Hls'
[[ "$pty_output" != *$'\e[2;2Hls'* ]] || fail 'redrew the word a column too far left'

print -r -- "ok: a popup session redraws the word where it is (daemon=$ZACRS_TEST_DAEMON)"
