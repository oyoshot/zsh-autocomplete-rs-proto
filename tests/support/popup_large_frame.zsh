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
# Every candidate starts with the whole typed word, so each visible row
# highlights every character of it. The escape sequences for that make one
# frame larger than a single 8 KiB read.
_zacrs_compsys() {
    _zacrs_ctx_valid=0
    local i
    _zacrs_captured=()
    for i in {10..39}; do
        _zacrs_captured+=("${ZACRS_TEST_WORD}${i}"$'\tcommand')
    done
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

typeset -x ZACRS_TEST_WORD=abcdefghijklmnopqrstuvwxyzabcdefghijkl

start_shell
send_keys "$ZACRS_TEST_WORD"
: > "$HOME/probe"
send_keys $'\x19'
wait_file probe "$ZACRS_TEST_WORD"

# The frame for the final buffer must arrive whole, up to its bottom border
# with the scroll range. Highlighting separates every typed character with
# escape sequences, so the rows themselves are not matched as text.
# Earlier frames for shorter buffers also show the range, so wait on the
# latest frame only.
for attempt in {1..300}; do
    drain_pty
    frame="${pty_output##*'┌'}"
    [[ "$frame" == *' 1–10 / 30 '*'┘'* ]] && break
    sleep 0.01
done
[[ "$frame" == *' 1–10 / 30 '*'┘'* ]] || fail 'the last frame lost its tail'
# Keep the fixture meaningful if rendering ever emits fewer bytes.
() {
    setopt localoptions nomultibyte
    (( ${#frame%'┘'*} > 8192 ))
} || fail 'the popup frame fits in one read, so this test proves nothing'

print -r -- "ok: popup frames larger than one read are drawn whole (daemon=$ZACRS_TEST_DAEMON)"
