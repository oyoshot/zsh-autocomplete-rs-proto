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

# Run a widget that leaves the buffer alone, and wait for the redraw after it.
probe() {
    : > "$HOME/probe"
    send_keys $'\x19'
    wait_file probe "$1"
    : > "$HOME/probe"
    send_keys $'\x19'
    wait_file probe "$1"
}
assert_later_hook_ran() {
    [[ -s "$HOME/before" ]] || fail "no redraw was observed $1"
    # The redraw after the last probe may still be between the two hooks. A
    # skipped redraw never catches up, so waiting cannot hide one.
    local attempt before after
    for attempt in {1..300}; do
        drain_pty
        before="$(<"$HOME/before")" after="$(<"$HOME/after")"
        [[ "$after" == "$before" ]] && return 0
        sleep 0.01
    done
    fail "the hook after ours missed a redraw $1: before=${(qqq)before} after=${(qqq)after}"
}

start_shell

# A burst of type-ahead: the popup is drawn from the last key's widget, so
# the redraw that follows finds the buffer unchanged.
send_keys ls
probe ls
assert_later_hook_ran 'after type-ahead'

# Moving the cursor back and deleting forward changes only the text after
# the cursor, which our hook does not react to.
send_keys $'\C-b'
send_keys $'\C-d'
probe l
assert_later_hook_ran 'after an edit behind the cursor'
[[ "$(tail -n 1 "$HOME/after")" == 'l:1' ]] || fail "last redraw seen: $(tail -n 1 "$HOME/after")"

print -r -- 'ok: redraw hooks registered after ours run on every redraw'
