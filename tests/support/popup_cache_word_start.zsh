mkdir -p "$HOME/project/foodir"
: > "$HOME/project/foofile"

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget compinit
compinit -D -u
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

plain_output() {
    setopt local_options extended_glob
    REPLY="${pty_output//$'\e'(\[[0-9;?]#[[:alpha:]]|[78])/}"
}

# wait_plain <what> <text>...: like wait_output, ignoring colors, which
# separate the matched characters of a candidate from the rest.
wait_plain() {
    local what="$1" attempt text missing
    shift
    for attempt in {1..300}; do
        drain_pty
        plain_output
        missing=0
        for text in "$@"; do
            [[ "$REPLY" == *"$text"* ]] || missing=1
        done
        (( missing )) || return 0
        sleep 0.01
    done
    fail "waiting for $what"
}

start_shell
[[ "$(<"$HOME/daemon")" == 1 ]] || fail 'the daemon was not available at the first prompt'

# After a redirection the word being completed starts at ">", not at the last
# space, and any file may follow.
send_keys 'cd >fo'
wait_plain 'the popup after the redirection' foodir foofile '└'

# Remove the ">" and return to the end, as one batch so only the final line is
# drawn. The line now ends in the same text after the same last space, but the
# word is an argument of cd.
pty_output=''
send_keys $'\x02\x02\x08\x05'
wait_plain 'the popup for the edited line' '┌' foodir '└'
: > "$HOME/probe"
send_keys $'\x19'
wait_file probe 'cd fo'
plain_output
[[ "$REPLY" == *foodir* ]] || fail "missing the directory: ${(qqq)REPLY}"
[[ "$REPLY" != *foofile* ]] || fail "offered a file gathered for the redirection: ${(qqq)REPLY}"
print -r -- 'ok: a set gathered for a word that starts after the last space is not served for another line'
