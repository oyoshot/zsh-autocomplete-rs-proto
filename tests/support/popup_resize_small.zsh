set -eu
zmodload zsh/zpty

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
    print -r -- "$BUFFER:$CURSOR:$_zacrs_popup_visible:$LINES" > "$HOME/probe"
}
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

cleanup() {
    zpty -d zacrs_resize 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_resize chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            zpty -w -n zacrs_resize $'\e[2;5R'
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
settle() {
    local attempt
    for attempt in {1..20}; do
        drain_pty
        sleep 0.01
    done
}
assert_no_frame() {
    [[ "$pty_output" != *'┌'* && "$pty_output" != *'└'* ]] || fail "drew a popup frame $1"
}

zpty -b zacrs_resize zsh -di
wait_file ready ready
zpty -w -n zacrs_resize ls
wait_popup

# Tab opens the interactive session on top of the visible popup.
zpty -w -n zacrs_resize $'\t'
settle
[[ ! -e "$HOME/tab-done" ]] || fail 'the popup session ended before the resize'

# Shrink the terminal under the open session.
pty_output=''
stty rows 3 < "$(<"$HOME/tty")"
wait_file tab-done done
settle
assert_no_frame 'after the resize'
[[ "$pty_output" == *$'\e[2K'* ]] || fail 'did not clear the popup rows'

# The session ended as a cancel: the buffer is intact and no popup is left.
zpty -w -n zacrs_resize $'\x19'
wait_file probe 'ls:2:0:3'
settle
assert_no_frame 'after the session'

print -r -- 'ok: shrinking the terminal under an open popup closes it'
