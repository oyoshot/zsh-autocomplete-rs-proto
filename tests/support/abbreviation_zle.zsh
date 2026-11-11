set -eu
zmodload zsh/zpty

# Both modes use real ZLE, the actual CLI, and production Enter/selection hooks.
# The socket-free mode supplies a popup result through a test widget. The daemon
# mode exercises the real Rust popup via Tab and Space as well.
cat > "$HOME/.zshrc" <<'ZSH'
unsetopt MONITOR
stty rows 24 cols 100
autoload -Uz compinit add-zle-hook-widget
compinit -i -d "$HOME/.zcompdump"
_test_load_plugin() {
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    zmodload() {
        [[ "$1" == (zsh/net/socket|zsh/net/unix) ]] && return 1
        builtin zmodload "$@"
    }
fi
source "$ZACRS_PLUGIN"
_zacrs_render() { :; }
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    unfunction zmodload
    _test_choose_command() {
        local cursor_offset=''
        _zacrs_apply ${#LBUFFER} 2 'gs ' 0 0 '' 1
    }
    zle -N _test_choose_command
    bindkey '^X' _test_choose_command
    _test_dismiss_unselected() {
        local cursor_offset=''
        _zacrs_apply ${#LBUFFER} 2 'gs ' 0 0 '' 0
    }
    zle -N _test_dismiss_unselected
    bindkey '^W' _test_dismiss_unselected
fi
functions[_test_original_apply]=$functions[_zacrs_apply]
_zacrs_apply() {
    _test_original_apply "$@"
    print -r -- "$BUFFER" > "$HOME/applied"
}
_zacrs_line_pre_redraw() {
    # Avoid unrelated auto popups while observing real ZLE readiness.
    print -r -- "$BUFFER" > "$HOME/redrawn"
}
}
if [[ "${ZACRS_TEST_DEFERRED:-0}" == 1 ]]; then
    # A readable descriptor loads the plugin after the first line-init, as an
    # idle loader does. Waiting for the marker avoids timing-based assertions.
    _test_deferred_load() {
        zle -F "$1"
        exec {1}<&-
        _test_load_plugin
        print -r -- loaded > "$HOME/loaded"
    }
    zle -N _test_deferred_load
    exec {_test_load_fd}</dev/null
    zle -F $_test_load_fd _test_deferred_load
else
    _test_load_plugin
fi
_test_choose_and_reload() {
    local cursor_offset=''
    _zacrs_apply ${#LBUFFER} 2 'gs ' 0 0 '' 1
    if [[ "${ZACRS_TEST_LEGACY_RELOAD:-0}" == 1 ]]; then
        # Older adapters have a live, protected daemon session but no flag.
        unset _zacrs_enter_session_initialized
        _zacrs_enter_enabled=${ZACRS_TEST_LEGACY_STATE:-1}
    fi
    _test_load_plugin
    _test_load_plugin
}
zle -N _test_choose_and_reload
bindkey '^V' _test_choose_and_reload
git() { print -r -- "git:$*" > "$HOME/executed"; }
gs() { print -r -- "gs:$*" > "$HOME/executed"; }
nvim() { print -r -- "nvim:$*" > "$HOME/executed"; }
clear() { print -r -- "clear:$*" > "$HOME/executed"; }
typeset -gi _test_ssh_count=0
ssh() {
    print -r -- "ssh:$*" > "$HOME/executed"
    print -r -- "$(( ++_test_ssh_count ))" > "$HOME/ssh-count"
}
zacrs_next() { print -r -- 'unexpected native selection' > "$HOME/executed"; }
_test_git_definition="$functions[git]"
preexec() { print -r -- "$1" > "$HOME/last-command"; }
typeset -gi _test_ready_count=0
_test_line_init() {
    (( ++_test_ready_count ))
    print -r -- "$_test_ready_count" > "$HOME/ready"
}
_test_observe_buffer() {
    print -r -- "$BUFFER" > "$HOME/editor"
    print -r -- "$CURSOR" > "$HOME/cursor"
}
add-zle-hook-widget line-init _test_line_init
zle -N _test_observe_buffer
bindkey '^Y' _test_observe_buffer
PROMPT='TEST> '
RPROMPT=''
ZSH

cleanup() {
    if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
        "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
    fi
    zpty -d zacrs_enter 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_enter chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            zpty -w -n zacrs_enter $'\e[1;1R'
            dsr_pending="${dsr_pending#*$'\e[6n'}"
        done
        # A terminal query may be split across reads.
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
    local filename="$1" expected="$2" attempt
    for attempt in {1..500}; do
        drain_pty
        [[ -f "$filename" && "$(<"$filename")" == "$expected" ]] && return 0
        sleep 0.01
    done
    fail "waiting for ${filename:t} = ${(qqq)expected}"
}
wait_popup() {
    setopt localoptions extendedglob
    local title="$1" attempt
    for attempt in {1..500}; do
        drain_pty
        # The title uses theme colors; ignore SGR when checking visible text.
        local plain_output=${pty_output//$'\e'\[[0-9\;]#m/}
        [[ "$plain_output" == *"┌ $title "* ]] && return 0
        sleep 0.01
    done
    fail "waiting for popup title ${(qqq)title}"
}
enter_line() {
    local next_ready=$(( $(<"$HOME/ready") + 1 ))
    zpty -w -n zacrs_enter "$1"$'\r'
    wait_file "$HOME/ready" "$next_ready"
}

if [[ "${ZACRS_TEST_DEFERRED:-0}" == 1 ]]; then
    print -rl -- '[completion]' 'expand_abbreviation_on_enter = true' >> "$XDG_CONFIG_HOME/zacrs/config.toml"
fi
zpty -b zacrs_enter zsh -di
wait_file "$HOME/ready" 1
if [[ "${ZACRS_TEST_DEFERRED:-0}" == 1 ]]; then
    wait_file "$HOME/loaded" loaded
    [[ "$(<"$HOME/ready")" == 1 ]] || fail 'deferred load started a new input line'
    enter_line gs
    wait_file "$HOME/executed" 'git:status'
    # Reloading in ZLE must preserve a command explicitly selected on this line.
    : > "$HOME/executed"
    zpty -w -n zacrs_enter $'gs\x16\x19'
    wait_file "$HOME/editor" 'gs '
    enter_line ''
    wait_file "$HOME/executed" 'gs:'
    : > "$HOME/executed"
    enter_line gs
    wait_file "$HOME/executed" 'git:status'
    print -r -- 'ok: deferred loading expands the first line and reload preserves selections'
    zpty -w -n zacrs_enter $'exit\r'
    exit 0
fi
enter_line gs
wait_file "$HOME/executed" 'gs:'
enter_line 'printf "\n[completion]\nexpand_abbreviation_on_enter = true\n" >> "$XDG_CONFIG_HOME/zacrs/config.toml"; _test_load_plugin'
if [[ "$ZACRS_TEST_DAEMON" == 0 ]]; then
    enter_line gs
    wait_file "$HOME/executed" 'gs:'
    [[ ! -S "$XDG_RUNTIME_DIR/zacrs.sock" ]] || fail 'offline shell started daemon'
    print -r -- 'ok: no daemon accepts the original input'
    exit 0
fi
enter_line 'gs -sb'
wait_file "$HOME/executed" 'git:status -sb'
enter_line 'fc -ln -1 > "$HOME/history"'
[[ "$(<"$HOME/history")" == *'git status -sb' ]] || fail 'expanded history'

for input in 'gs "$HOME"' 'gs "$HOME" ' 'gs $HOME' 'gs "${HOME}"'; do
    : > "$HOME/executed"
    enter_line "$input"
    wait_file "$HOME/executed" "git:status $HOME"
done

enter_line "alias gs='echo aliased'"
enter_line 'unsetopt completealiases'
for input in gs 'gs ' 'gs -sb'; do
    : > "$HOME/executed"
    enter_line "$input"
    if [[ "$input" == 'gs -sb' ]]; then
        wait_file "$HOME/executed" 'git:status -sb'
    else
        wait_file "$HOME/executed" 'git:status'
    fi
done
: > "$HOME/executed"
enter_line 'gs "$HOME"'
wait_file "$HOME/executed" "git:status $HOME"
enter_line 'unalias gs'

zpty -w -n zacrs_enter $'gcm\r\x19'
wait_file "$HOME/editor" "git commit -m ''"
wait_file "$HOME/cursor" 15
enter_line EDITED
wait_file "$HOME/executed" 'git:commit -m EDITED'

typeset -i ssh_count=0
for trigger in ssh connect; do
    zpty -w -n zacrs_enter "$trigger"$'\r\x19'
    wait_file "$HOME/editor" 'ssh user@'
    wait_file "$HOME/cursor" 9
    enter_line host
    wait_file "$HOME/executed" 'ssh:user@host'
    wait_file "$HOME/ssh-count" "$(( ++ssh_count ))"
done

enter_line "cat <<'EOF' > \"\$HOME/heredoc\""
enter_line gs
enter_line EOF
wait_file "$HOME/heredoc" gs
enter_line 'print -r -- "'
enter_line gs
enter_line '" > "$HOME/quoted"'
wait_file "$HOME/quoted" $'\ngs'

# Typing in the Rust popup clears its selection. Space then returns the filter,
# which must still be eligible for expansion when Enter accepts it.
: > "$HOME/applied"
: > "$HOME/redrawn"
zpty -w -n zacrs_enter $'g\x19'
wait_file "$HOME/editor" g
if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
    pty_output=''
    zpty -w -n zacrs_enter $'\t'
    wait_popup g
    zpty -w -n zacrs_enter 's '
else
    zpty -w -n zacrs_enter $'\x17'
fi
wait_file "$HOME/applied" 'gs '
wait_file "$HOME/redrawn" 'gs '
enter_line ''
wait_file "$HOME/executed" 'git:status'

for trigger in gs zacrs_next; do
    : > "$HOME/applied"
    : > "$HOME/redrawn"
    zpty -w -n zacrs_enter "$trigger"$'\x19'
    wait_file "$HOME/editor" "$trigger"
    if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
        pty_output=''
        zpty -w -n zacrs_enter $'\t'
        wait_popup "$trigger"
        if [[ "$trigger" == gs ]]; then
            zpty -w -n zacrs_enter ' '
        else
            # The native command is first; choose the abbreviation below it.
            zpty -w -n zacrs_enter $'\e[B '
        fi
    else
        zpty -w -n zacrs_enter $'\x18'
    fi
    wait_file "$HOME/applied" 'gs '
    wait_file "$HOME/redrawn" 'gs '
    enter_line '-sb'
    wait_file "$HOME/executed" 'gs:-sb'
done
enter_line gs
wait_file "$HOME/executed" 'git:status'

# Both terminal interruption and the send-break widget begin a fresh ZLE line.
# The sole line-init notification must clear a previously selected native word.
for abort_key in $'\x03' $'\x07'; do
    : > "$HOME/editor"
    : > "$HOME/applied"
    : > "$HOME/redrawn"
    zpty -w -n zacrs_enter $'gs\x19'
    wait_file "$HOME/editor" gs
    pty_output=''
    zpty -w -n zacrs_enter $'\t'
    wait_popup gs
    zpty -w -n zacrs_enter ' '
    wait_file "$HOME/applied" 'gs '
    wait_file "$HOME/redrawn" 'gs '
    next_ready=$(( $(<"$HOME/ready") + 1 ))
    zpty -w -n zacrs_enter "$abort_key"
    wait_file "$HOME/ready" "$next_ready"
    enter_line gs
    wait_file "$HOME/executed" 'git:status'
done

enter_line 'gs() print hello'
wait_file "$HOME/last-command" 'gs() print hello'
enter_line '[[ "$functions[git]" == "$_test_git_definition" && ${+functions[status]} == 0 ]] && print -r -- ok > "$HOME/function-ok"'
wait_file "$HOME/function-ok" ok
if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
    [[ "$("$ZACRS_BIN" daemon status)" == running ]] || fail 'daemon is not running'
else
    [[ ! -S "$XDG_RUNTIME_DIR/zacrs.sock" ]] || fail 'socket-free test started a daemon'
fi
# Updating stops the backend while this shell remains open. The next real
# line-init must restore Enter expansion without sourcing the plugin again.
if [[ "$ZACRS_TEST_DAEMON" == 1 ]]; then
    enter_line '"$ZACRS_BIN" daemon stop'
    : > "$HOME/executed"
    enter_line gs
    [[ "$(<"$HOME/executed")" == 'git:status' ]] || fail 'Enter expansion did not recover after update'
fi
# Re-sourcing must preserve Enter expansion for both trigger lengths. Use the
# real exit builtin last so e is verified without replacing its implementation.
enter_line '_test_load_plugin; _test_load_plugin'
for trigger expected in gs 'git:status' v 'nvim:' c 'clear:'; do
    : > "$HOME/executed"
    enter_line "$trigger"
    wait_file "$HOME/executed" "$expected"
done
zpty -w -n zacrs_enter $'e\r'
wait_file "$HOME/last-command" exit
for attempt in {1..500}; do
    drain_pty
    if ! zpty -t zacrs_enter; then
        print -r -- "ok: real ZLE Enter, history, cursor, continuation, selection, declarations and single-character triggers (daemon=$ZACRS_TEST_DAEMON)"
        exit 0
    fi
    sleep .01
done
fail 'e did not exit the interactive shell'
