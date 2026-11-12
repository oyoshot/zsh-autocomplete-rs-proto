set -eu
zmodload zsh/zpty

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget
typeset -gi _test_other_count=0 _test_ready_count=0
_test_other_init() { (( ++_test_other_count )); }
if [[ "$ZACRS_HOOK_ORDER" == before ]]; then
    add-zle-hook-widget line-init _test_other_init
fi
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    zmodload() {
        [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
        builtin zmodload "$@"
    }
fi
_test_load_plugin() {
    source "$ZACRS_PLUGIN"
    # Keep candidate generation deterministic; rendering and ZLE are real.
    _zacrs_compsys() {
        _zacrs_ctx_valid=0
        _zacrs_captured=($'ls\tcommand' $'lsblk\tcommand' $'lscpu\tcommand')
    }
    zle -N _zacrs_compsys
}
_test_load_plugin
if [[ "$ZACRS_HOOK_ORDER" == after ]]; then
    add-zle-hook-widget line-init _test_other_init
fi
_test_ready() {
    (( ++_test_ready_count ))
    print -r -- "$_zacrs_popup_visible:$_zacrs_popup_row:$_zacrs_popup_height:$_zacrs_popup_cursor_row:$_zacrs_last_render_cursor_row:$_zacrs_last_render_cursor_col:$_zacrs_popup_snapshot_lbuffer:$_zacrs_prev_lbuffer:$_zacrs_redraw_deferred:$_zacrs_self_insert_wrapped" > "$HOME/state"
    print -r -- "$_test_other_count" > "$HOME/other-count"
    print -r -- "$_test_ready_count" > "$HOME/ready"
}
add-zle-hook-widget line-init _test_ready
_test_probe() {
    print -r -- "$BUFFER:$CURSOR:$_zacrs_popup_visible:$_zacrs_enter_enabled:$_zacrs_daemon_available" > "$HOME/probe"
}
_test_outer_init() { zle _test_saved_init -f nolast; }
_test_reload() {
    # A competing plugin can wrap the dispatcher before repeated reloads.
    zle -A zle-line-init _test_saved_init
    zle -N zle-line-init _test_outer_init
    _test_load_plugin
    _test_load_plugin
    print -r -- reloaded > "$HOME/reloaded"
}
_test_defer() {
    _zacrs_defer_redraw
    print -r -- deferred > "$HOME/deferred"
}
_test_accept() {
    # A different plugin can accept without using zacrs's accept-line wrapper.
    BUFFER=''
    zle .accept-line
}
zle -N _test_probe
zle -N _test_reload
zle -N _test_defer
zle -N _test_accept
bindkey '^Y' _test_probe
bindkey '^R' _test_reload
bindkey '^T' _test_defer
bindkey '^O' _test_accept
bindkey '^G' send-break
bindkey '^X' .send-break
PROMPT=$'┌── プロンプト\n└─▶ '
RPROMPT=''
ZSH

cleanup() {
    zpty -d zacrs_lifecycle 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_lifecycle chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            # Change position on each new line, so stale cursor reuse differs.
            local row=$(( 2 + $(<"$HOME/ready") ))
            zpty -w -n zacrs_lifecycle $'\e['"$row"';6R'
            dsr_pending="${dsr_pending#*$'\e[6n'}"
        done
        dsr_pending="${dsr_pending[-3,-1]}"
    done
    return 0
}
fail() {
    print -u2 -r -- "not ok: $*"
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    exit 1
}
wait_file() {
    local file="$1" expected="$2" attempt
    for attempt in {1..300}; do
        drain_pty
        [[ -f "$HOME/$file" && "$(<"$HOME/$file")" == "$expected" ]] && return 0
        sleep 0.01
    done
    fail "waiting for $file = ${(qqq)expected}"
}
assert_fresh_line() {
    local expected="$1"
    wait_file ready "$expected"
    [[ "$(<"$HOME/state")" == '0:0:0:0:0:0:::0:0' ]] \
        || fail "stale state at line $expected: $(<"$HOME/state")"
    [[ "$(<"$HOME/other-count")" == "$expected" ]] \
        || fail "competing line-init hook count: $(<"$HOME/other-count"), expected $expected"
    local after_prompt="${pty_output##*└─▶ }"
    [[ "$after_prompt" != *$'\e[2K'* ]] || fail 'erased a row after displaying the new prompt'
}
wait_popup() {
    setopt localoptions extendedglob
    local attempt
    for attempt in {1..300}; do
        drain_pty
        local plain_output=${pty_output//$'\e'\[[0-9\;]#m/}
        [[ "$plain_output" == *'┌ ls '* && "$plain_output" == *'lscpu'* ]] && return 0
        sleep 0.01
    done
    fail 'waiting for a real completion popup'
}

zpty -b zacrs_lifecycle zsh -di
assert_fresh_line 1
typeset -i next_line=1
typeset abort_key
for abort_key in $'\x03' $'\x07' $'\x18' $'\x0f'; do
    # Include a reload while another plugin wraps line-init.
    if (( next_line == 2 )); then
        zpty -w -n zacrs_lifecycle $'\x12'
        wait_file reloaded reloaded
    fi
    pty_output=''
    zpty -w -n zacrs_lifecycle ls
    wait_popup
    : > "$HOME/probe"
    zpty -w -n zacrs_lifecycle $'\x19'
    wait_file probe "ls:2:1:$ZACRS_TEST_ENTER:$ZACRS_TEST_DAEMON"
    # Terminal interruption and acceptance must both restore a deferred widget.
    if (( next_line == 1 || next_line == 4 )); then
        : > "$HOME/deferred"
        zpty -w -n zacrs_lifecycle $'\x14'
        wait_file deferred deferred
        # The deferred marker is written before its redraw hook runs. Probe
        # again so VINTR cannot race terminal setup in that pending redraw.
        : > "$HOME/probe"
        zpty -w -n zacrs_lifecycle $'\x19'
        wait_file probe "ls:2:1:$ZACRS_TEST_ENTER:$ZACRS_TEST_DAEMON"
    fi
    zpty -w -n zacrs_lifecycle "$abort_key"
    (( ++next_line ))
    assert_fresh_line "$next_line"
done
print -r -- 'ok: new ZLE lines discard stale popup state without erasing the prompt'
