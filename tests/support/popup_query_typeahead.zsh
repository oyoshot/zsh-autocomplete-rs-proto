cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows "$ZACRS_TEST_ROWS" cols 100
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
    if [[ "$ZACRS_TEST_CASE" == single ]]; then
        _zacrs_captured=($'lsblk\tcommand')
    else
        _zacrs_captured=($'ls\tcommand' $'lsblk\tcommand' $'lscpu\tcommand')
    fi
}
zle -N _zacrs_compsys
# Stand in for native completion so the fallback is observable.
_test_native() { print -r -- "$BUFFER" > "$HOME/native"; }
zle -N expand-or-complete _test_native
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
_test_probe() { print -r -- "$BUFFER" > "$HOME/probe"; }
zle -N _test_probe
bindkey '^Y' _test_probe
# Hide the popup, so that Tab starts a session without one on screen.
_test_hide() { _zacrs_clear_popup; }
zle -N _test_hide
bindkey '^O' _test_hide
PROMPT='> '
RPROMPT=''
ZSH

# wait_buffer <text>: probe until the edit buffer is exactly this.
wait_buffer() {
    local attempt
    : > "$HOME/probe"
    send_keys $'\x19'
    for attempt in {1..300}; do
        drain_pty
        [[ "$(<"$HOME/probe")" == "$1" ]] && return 0
        # Ask again after a wrong answer, and now and then in case the
        # probe key itself was typed ahead of a reply.
        if [[ -s "$HOME/probe" ]] || (( attempt % 20 == 0 )); then
            : > "$HOME/probe"
            send_keys $'\x19'
        fi
        sleep 0.01
    done
    fail "buffer is ${(qqq)$(<"$HOME/probe")}, expected ${(qqq)1}"
}

start_shell

case "$ZACRS_TEST_CASE" in
    typing)
        # The automatic popup asks for the cursor position after "l"; "s"
        # arrives before the terminal answers.
        dsr_typeahead=s
        send_keys l
        wait_reply
        wait_buffer ls
        # The popup then follows the buffer that the queued key produced.
        wait_output 'the popup for the final buffer' '┌' 'blk'
        ;;
    single)
        # Tab inserts the only candidate without opening a popup session.
        send_keys lsb
        wait_buffer lsb
        dsr_typeahead=x
        send_keys $'\t'
        wait_reply
        wait_buffer 'lsblk x'
        ;;
    native)
        # Tab hands over to native completion without opening a session.
        send_keys ls
        wait_buffer ls
        dsr_typeahead=x
        send_keys $'\t'
        wait_reply
        wait_file native ls
        wait_buffer lsx
        ;;
    session)
        # The first key ends the popup session; the rest belongs to the line.
        send_keys ls
        wait_buffer ls
        dsr_typeahead=' ab'
        send_keys $'\t'
        wait_reply
        wait_buffer 'ls ab'
        ;;
    hidden)
        # As "session", with no popup on screen when Tab starts the session.
        send_keys ls
        wait_buffer ls
        send_keys $'\C-o'
        wait_buffer ls
        dsr_typeahead=' ab'
        send_keys $'\t'
        wait_reply
        wait_buffer 'ls ab'
        ;;
    passthrough)
        # The session hands its first key back to the shell. It must run
        # before the key typed after it.
        send_keys ls
        wait_buffer ls
        dsr_typeahead=$'\C-a'x
        send_keys $'\t'
        wait_reply
        wait_buffer xls
        ;;
    navigation)
        # Keys pressed in a session already on screen. Only the session
        # marks a selected row; the popup shown while typing does not.
        send_keys ls
        wait_buffer ls
        pty_output=''
        send_keys $'\t'
        wait_output 'the popup session' '▸'
        # Space inserts the row that ZACRS_TEST_KEYS selected.
        send_keys "$ZACRS_TEST_KEYS "
        wait_buffer "$ZACRS_TEST_EXPECT"
        print -r -- "ok: ${(qqq)ZACRS_TEST_KEYS} selects ${(qqq)ZACRS_TEST_EXPECT}"
        exit 0
        ;;
esac

print -r -- "ok: keys typed before the cursor reply survive ($ZACRS_TEST_CASE)"
