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
# Return at most 256 bytes per read, as a socket may, so that every frame
# read from the daemon takes several reads whatever the renderer emits for
# it. Without the daemon, frames reach the terminal without sysread.
sysread() {
    local -a args=("$@")
    local i=${args[(i)-s]}
    if (( i < $#args )); then
        (( args[i+1] > 256 )) && args[i+1]=256
    else
        args=(-s 256 "${args[@]}")
    fi
    builtin sysread "${args[@]}"
}
# Keep candidate generation deterministic; rendering and ZLE are real.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    local i
    _zacrs_captured=()
    for i in {10..39}; do
        _zacrs_captured+=("ls$i"$'\tcommand')
    done
}
zle -N _zacrs_compsys
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
PROMPT='> '
RPROMPT=''
ZSH

start_shell
send_keys ls

# The frame must arrive whole, up to its bottom border with the scroll range.
# The frame for "l" also shows the range, so wait on the latest frame only.
for attempt in {1..300}; do
    drain_pty
    frame="${pty_output##*'┌'}"
    [[ "$frame" == *' 1–10 / 30 '*'┘'* ]] && break
    sleep 0.01
done
[[ "$frame" == *' 1–10 / 30 '*'┘'* ]] || fail 'the last frame lost its tail'
() {
    setopt localoptions nomultibyte
    local bytes=${#frame%'┘'*}
    (( bytes > 256 )) ||
        fail "the popup frame ($bytes bytes) fits in one read, so this test proves nothing"
}

print -r -- "ok: popup frames larger than one read are drawn whole (daemon=$ZACRS_TEST_DAEMON)"
