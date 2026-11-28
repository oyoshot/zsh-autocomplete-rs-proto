mkdir -p "$HOME/project"
: > "$HOME/project/README.md"

# A binary from before the file provider: it rejects the file arguments the
# way clap does, on stderr.
cat > "$HOME/old-binary" <<'SH'
#!/bin/sh
for argument do
    case "$argument" in
        --filesystem-*)
            echo "error: unexpected argument '$argument' found" >&2
            echo "Usage: zsh-autocomplete-rs render [OPTIONS]" >&2
            exit 2
            ;;
    esac
done
exec "$ZACRS_REAL_BIN" "$@"
SH
chmod +x "$HOME/old-binary"

cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
zmodload zsh/zle
autoload -Uz add-zle-hook-widget compinit
compinit -D -u
# Without a daemon the adapter runs the binary for every popup.
zmodload() {
    [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
    builtin zmodload "$@"
}
cd "$HOME/project"
export ZACRS_REAL_BIN="$ZACRS_BIN"
ZACRS_BIN="$HOME/old-binary"
source "$ZACRS_PLUGIN"
_test_ready() { print -r -- ready > "$HOME/ready"; }
add-zle-hook-widget line-init _test_ready
_test_probe() { print -r -- "$(( ++_test_probes )):$BUFFER" > "$HOME/probe"; }
zle -N _test_probe
bindkey '^Y' _test_probe
PROMPT='> '
RPROMPT=''
ZSH

start_shell
# No file starts with "zzzq", so native completion returns nothing and the
# adapter asks the binary for file candidates.
send_keys 'ls zzzq'
# The adapter asks for the cursor position just before it runs the binary.
wait_output 'the cursor query before the render' $'\e[6n'
# A probe is handled only once that redraw has returned. The second covers a
# first one read while the adapter was still waiting for the cursor reply.
send_keys $'\x19'
wait_file probe '1:ls zzzq'
send_keys $'\x19'
wait_file probe '2:ls zzzq'
drain_pty
[[ "$pty_output" != *'unexpected argument'* && "$pty_output" != *Usage:* ]] \
    || fail 'the binary'"'"'s usage error reached the terminal'
print -r -- 'ok: an older binary that rejects file arguments prints nothing on the edit line'
