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
# Another plugin's redraw hooks, one registered on each side of ours.
_test_before() { print -r -- "$BUFFER:$CURSOR" >> "$HOME/before"; }
add-zle-hook-widget line-pre-redraw _test_before
source "$ZACRS_PLUGIN"
_test_after() { print -r -- "$BUFFER:$CURSOR" >> "$HOME/after"; }
add-zle-hook-widget line-pre-redraw _test_after
# Keep candidate generation deterministic; rendering and ZLE are real.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    _zacrs_captured=($'ls\tcommand' $'lsblk\tcommand' $'lscpu\tcommand')
}
zle -N _zacrs_compsys
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
_test_probe() { print -r -- "$BUFFER" > "$HOME/probe"; }
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

# wait_redraw <buffer:cursor>: wait until the hook after ours has seen this
# state. Our hook has then returned, so its cursor query is no longer reading
# the terminal and the next keys cannot be taken for the reply.
wait_redraw() {
    local attempt
    for attempt in {1..300}; do
        drain_pty
        [[ "$(tail -n 1 "$HOME/after")" == "$1" ]] && return 0
        sleep 0.01
    done
    fail "the hook after ours did not see $1: before=${(qqq)$(<"$HOME/before")} after=${(qqq)$(<"$HOME/after")}"
}
# Run a widget that leaves the buffer alone.
probe() {
    : > "$HOME/probe"
    send_keys $'\x19'
    wait_file probe "$1"
}
assert_later_hook_ran() {
    # The redraw after the probe may still be between the two hooks. A
    # skipped redraw never catches up, so waiting cannot hide one.
    local attempt before after
    for attempt in {1..300}; do
        drain_pty
        before="$(<"$HOME/before")" after="$(<"$HOME/after")"
        [[ -n "$before" && "$after" == "$before" ]] && return 0
        sleep 0.01
    done
    fail "the hook after ours missed a redraw $1: before=${(qqq)before} after=${(qqq)after}"
}

: > "$HOME/before"
: > "$HOME/after"
start_shell

# Keys sent together: when they arrive as type-ahead, the popup is drawn from
# the last key's widget and the redraw that follows finds the buffer unchanged.
send_keys ls
wait_redraw ls:2
probe ls
assert_later_hook_ran 'after typing'

# Deleting forward changes only the text behind the cursor, which our hook
# does not react to.
send_keys $'\C-b'
wait_redraw ls:1
send_keys $'\C-d'
wait_redraw l:1
probe l
assert_later_hook_ran 'after an edit behind the cursor'

print -r -- 'ok: redraw hooks registered after ours run on every redraw'
