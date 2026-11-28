set -eu
zmodload zsh/zpty

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget
source "$ZACRS_PLUGIN"
typeset -gi _test_gathers=0
# Native completion that is slow until the test makes it fast.
_zacrs_compsys() {
    (( ++_test_gathers ))
    [[ -e "$HOME/fast" ]] || sleep 0.2
    _zacrs_ctx_valid=0
    _zacrs_captured=($'alpha\tword' $'alpine\tword' $'altair\tword')
}
zle -N _zacrs_compsys
_test_probe() {
    print -r -- "$BUFFER:$_zacrs_popup_visible:$_test_gathers:$_zacrs_daemon_available" > "$HOME/probe"
}
_test_fast() {
    : > "$HOME/fast"
    print -r -- done > "$HOME/fast-done"
}
_test_ready() { print -r -- ready > "$HOME/ready"; }
zle -N _test_probe
zle -N _test_fast
bindkey '^Y' _test_probe
bindkey '^P' _test_fast
add-zle-hook-widget line-init _test_ready
PROMPT='> '
RPROMPT=''
ZSH

cleanup() {
    zpty -d zacrs_slow 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_slow chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            zpty -w -n zacrs_slow $'\e[3;6R'
            dsr_pending="${dsr_pending#*$'\e[6n'}"
        done
        dsr_pending="${dsr_pending[-3,-1]}"
    done
    return 0
}
fail() {
    print -u2 -r -- "not ok: $*"
    print -u2 -r -- "probe: $(<"$HOME/probe")"
    print -u2 -r -- "pty: ${(qqq)pty_output}"
    exit 1
}
wait_file() {
    local file="$1" expected="$2" attempt
    for attempt in {1..500}; do
        drain_pty
        [[ -f "$HOME/$file" && "$(<"$HOME/$file")" == "$expected" ]] && return 0
        sleep 0.01
    done
    fail "waiting for $file = ${(qqq)expected}"
}
# Wait until the terminal shows $1, ignoring colors.
wait_output() {
    setopt localoptions extendedglob
    local attempt
    for attempt in {1..500}; do
        drain_pty
        [[ "${pty_output//$'\e'\[[0-9\;]#m/}" == *"$1"* ]] && return 0
        sleep 0.01
    done
    fail "waiting for output ${(qqq)1}"
}
probe() {
    : > "$HOME/probe"
    zpty -w -n zacrs_slow $'\x19'
    wait_file probe "$1"
}
type_and_wait() {
    pty_output=''
    zpty -w -n zacrs_slow "$1"
    wait_output "$2"
}

: > "$HOME/probe"
zpty -b zacrs_slow zsh -di
wait_file ready ready

# Two slow gathers in a row for the context "cmd ".
type_and_wait 'cmd ' altair
probe 'cmd :1:1:1'
type_and_wait a altair
probe 'cmd a:1:2:1'

# The engine now answers SLOW: the popup is cleared and nothing is gathered.
type_and_wait l $'\e[2K'
probe 'cmd al:0:2:1'

# Tab still gathers. That gather is fast, so the context is automatic again.
zpty -w -n zacrs_slow $'\x10'
wait_file fast-done done
type_and_wait $'\t' altair
# Let the picker read the cancel key on its own before the probe key follows.
type_and_wait $'\e' $'\e[2K'
probe 'cmd al:0:3:1'
type_and_wait t altair
probe 'cmd alt:1:4:1'
print -r -- 'ok: a slow command context stops automatic popups until a fast gather'
