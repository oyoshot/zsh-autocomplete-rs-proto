set -eu
zmodload zsh/zpty
# Run buffer-level checks inside actual ZLE so native completion context is
# available. Acceptance is intercepted by the test; shell code is not executed.
cat > "$HOME/.zshrc" <<'ZSH'
PROMPT='adapter> '
_test_adapter() {
    source "$ZACRS_ADAPTER_TEST_FILE" > "$HOME/adapter-output" 2>&1
    print -r -- done > "$HOME/adapter-done"
    BUFFER=''
    CURSOR=0
}
zle -N _test_adapter
bindkey '^X' _test_adapter
print -r -- ready > "$HOME/adapter-ready"
ZSH
trap '"$ZACRS_BIN" daemon stop >/dev/null 2>&1 || true; zpty -d adapter 2>/dev/null || true' EXIT
zpty -b adapter zsh -di
for attempt in {1..500}; do
    [[ -e "$HOME/adapter-ready" ]] && break
    sleep .01
done
zpty -w -n adapter $'\x18'
for attempt in {1..1000}; do
    while zpty -r -t adapter chunk 2>/dev/null; do :; done
    if [[ -e "$HOME/adapter-done" ]]; then
        cat "$HOME/adapter-output"
        exit 0
    fi
    sleep .01
done
cat "$HOME/adapter-output" >&2
print -u2 -r -- 'not ok: native ZLE adapter checks did not complete'
exit 1
