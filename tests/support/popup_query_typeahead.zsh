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
# Tab reuses the cursor position of a visible popup. Hide it so that Tab asks
# the terminal again.
_test_hide() { _zacrs_clear_popup; }
zle -N _test_hide
bindkey '^O' _test_hide
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

# wait_buffer <text>: probe until the edit buffer is exactly this.
wait_buffer() {
    local attempt
    for attempt in {1..30}; do
        : > "$HOME/probe"
        send_keys $'\x19'
        local inner
        for inner in {1..20}; do
            drain_pty
            [[ "$(<"$HOME/probe")" == "$1" ]] && return 0
            [[ -s "$HOME/probe" ]] && break
            sleep 0.01
        done
    done
    fail "buffer is ${(qqq)$(<"$HOME/probe")}, expected ${(qqq)1}"
}

# Wait until the terminal has answered the query, with the typed keys first.
wait_reply() {
    local attempt
    for attempt in {1..300}; do
        drain_pty
        [[ -z "$dsr_typeahead" ]] && return 0
        sleep 0.01
    done
    fail 'waiting for the plugin to ask for the cursor position'
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
        send_keys $'\C-o'
        wait_buffer ls
        dsr_typeahead=' ab'
        send_keys $'\t'
        wait_reply
        wait_buffer 'ls ab'
        ;;
esac

print -r -- "ok: keys typed before the cursor reply survive ($ZACRS_TEST_CASE)"
