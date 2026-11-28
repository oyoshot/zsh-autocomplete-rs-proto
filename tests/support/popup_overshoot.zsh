mkdir -p "$HOME/project"
: > "$HOME/project/README.md"
: > "$HOME/project/Rakefile"

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget compinit
compinit -D -u
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    zmodload() {
        [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
        builtin zmodload "$@"
    }
fi
cd "$HOME/project"
# Candidates come from the real completion system and file system.
source "$ZACRS_PLUGIN"
_test_ready() {
    print -r -- "$_zacrs_daemon_available" > "$HOME/daemon"
    print -r -- ready > "$HOME/ready"
}
add-zle-hook-widget line-init _test_ready
_test_probe() { print -r -- "$BUFFER" > "$HOME/probe"; }
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

# Probes are sent only once the redraw they follow is complete: a key that
# arrives while the plugin waits for the cursor position defers that redraw.
probe() {
    : > "$HOME/probe"
    send_keys $'\x19'
    wait_file probe "$1"
}
plain_output() {
    setopt local_options extended_glob
    REPLY="${pty_output//$'\e'(\[[0-9;?]#[[:alpha:]]|[78])/}"
}

start_shell
# The rescue reads candidates cached by the daemon, so say so if it never came up.
[[ "$(<"$HOME/daemon")" == "$ZACRS_TEST_DAEMON" ]] \
    || fail "daemon availability was $(<"$HOME/daemon") at the first prompt"

# The popup for the existing file also leaves its candidates in the cache.
send_keys 'cat README.md'
# The popup title is the query, so this skips popups for shorter prefixes.
wait_output 'the popup for the existing file' ' README.md ' '└'
pty_output=''
# README.mdx matches no file, so compsys returns nothing for it.
send_keys x
if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
    wait_output 'the popup for the name typed past' ' README.mdx ' '└'
    plain_output
    # Its only row is the existing file.
    [[ "$REPLY" == *'│  README.md │'* ]] || fail "missing the rescued row: ${(qqq)REPLY}"
    [[ "$REPLY" != *Rakefile* ]] || fail 'rescued a name the query does not extend'
    # The cached name is unconfirmed: Tab opens a popup with nothing selected,
    # and Enter keeps the typed text.
    pty_output=''
    send_keys $'\t'
    wait_output 'the confirmation popup' ' README.mdx ' '└'
    [[ "$pty_output" != *'▸'* ]] || fail 'preselected the name typed past'
    send_keys $'\r'
    probe 'cat README.mdx'
    # Moving to the name explicitly applies it.
    pty_output=''
    send_keys $'\t'
    wait_output 'the confirmation popup' ' README.mdx ' '└'
    send_keys $'\t'
    wait_output 'the selected name' '▸'
    send_keys ' '
    probe 'cat README.md '
    print -r -- 'ok: a file name typed past is offered and applied only when chosen'
else
    # Without the daemon there is no earlier candidate set: the popup closes
    # and Tab keeps the typed text.
    probe 'cat README.mdx'
    assert_no_popup 'without the daemon'
    send_keys $'\t'
    probe 'cat README.mdx'
    print -r -- 'ok: without the daemon a name typed past keeps the typed text'
fi

# Shell syntax after a cached name is not a name typed past: Tab still hands
# the word to Zsh's own expansion.
send_keys $'\x15'
probe ''
pty_output=''
send_keys 'cat README.md'
wait_output 'the popup for the existing file' ' README.md ' '└'
send_keys '{,.bak}'
probe 'cat README.md{,.bak}'
send_keys $'\t'
probe 'cat README.md README.md.bak '
print -r -- 'ok: Tab expands shell syntax after a cached name'
