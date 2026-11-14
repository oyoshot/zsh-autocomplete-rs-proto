PROMPT=''
RPROMPT=''
autoload -Uz compinit add-zle-hook-widget
compinit -C -d "$HOME/compdump"
zmodload zsh/datetime
typeset -F 9 _bench_start=$EPOCHREALTIME
if [[ -n $BENCH_PLUGIN ]]; then
    source "$BENCH_PLUGIN" || exit 1
fi
typeset -F 6 _bench_source_ms=$(( (EPOCHREALTIME - _bench_start) * 1000 ))
# Validate outside the source interval. A failed connection must not look fast.
if [[ -n $BENCH_PLUGIN ]]; then
    (( _zacrs_daemon_available == 1 &&
       _zacrs_enter_expansion == BENCH_ENTER &&
       _zacrs_space_enabled == BENCH_SPACE )) || {
        print -u2 -- "zacrs initialization failed: daemon=$_zacrs_daemon_available enter=$_zacrs_enter_expansion space=$_zacrs_space_enabled"
        exit 2
    }
fi
_bench_ready() {
    if [[ -n $BENCH_PLUGIN ]] && (( BENCH_ENTER || BENCH_SPACE )); then
        (( _zacrs_enter_session_initialized == 1 )) || {
            print -u2 -- 'zacrs selection session was not initialized'
            exit 3
        }
    fi
    print -r -- "__ZACRS_READY__:${_bench_source_ms}"
}
# Follow zacrs's line-init callback, including session reset.
add-zle-hook-widget line-init _bench_ready
