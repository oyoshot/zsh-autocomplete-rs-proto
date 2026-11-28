# Shared driver for PTY popup tests. The test script that follows writes
# "$HOME/.zshrc" and then starts the shell under test with `start_shell`.
set -eu
zmodload zsh/zpty

cleanup() {
    zpty -d zacrs_popup 2>/dev/null || true
    "$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
typeset pty_output='' dsr_pending=''
# Keys the user types after the next cursor position query and before the
# terminal's reply to it.
typeset dsr_typeahead=''
# The 1-based row and column the terminal reports for the cursor, and
# whether it leaves cursor position queries unanswered.
typeset -i dsr_row=2 dsr_col=5 dsr_silent=0
# Collect terminal output and answer each cursor position query.
drain_pty() {
    local chunk=''
    while zpty -r -t zacrs_popup chunk 2>/dev/null; do
        pty_output+="$chunk"
        dsr_pending+="$chunk"
        while [[ "$dsr_pending" == *$'\e[6n'* ]]; do
            (( dsr_silent )) ||
                zpty -w -n zacrs_popup "$dsr_typeahead"$'\e['"$dsr_row;$dsr_col"'R'
            dsr_typeahead=''
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
send_keys() {
    zpty -w -n zacrs_popup "$1"
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
# wait_output <what> <text>...: wait until the output contains every text.
wait_output() {
    local what="$1" attempt text missing
    shift
    for attempt in {1..300}; do
        drain_pty
        missing=0
        for text in "$@"; do
            [[ "$pty_output" == *"$text"* ]] || missing=1
        done
        (( missing )) || return 0
        sleep 0.01
    done
    fail "waiting for $what"
}
# Wait until the terminal has answered a query, with `dsr_typeahead` first.
wait_reply() {
    local attempt
    for attempt in {1..300}; do
        drain_pty
        [[ -z "$dsr_typeahead" ]] && return 0
        sleep 0.01
    done
    fail 'waiting for the plugin to ask for the cursor position'
}
assert_no_popup() {
    drain_pty
    [[ "$pty_output" != *'┌'* && "$pty_output" != *'└'* ]] || fail "drew a popup frame $1"
}
# The shell under test reports its first prompt through "$HOME/ready".
start_shell() {
    zpty -b zacrs_popup zsh -di
    wait_file ready ready
}
