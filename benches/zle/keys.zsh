HISTFILE="$HOME/history"
autoload -Uz compinit add-zle-hook-widget
compinit -D
source "$BENCH_PLUGIN" || exit 1
if [[ $BENCH_MODE == hot-disabled ]]; then
    zle -A self-insert _bench_saved
    _bench_outer() { zle _bench_saved -w; }
    zle -N self-insert _bench_outer
    print -rl -- '[completion]' 'expand_abbreviation_on_space = false' '[abbreviations]' 'gs = "git status"' > "$XDG_CONFIG_HOME/zacrs/config.toml"
    source "$BENCH_PLUGIN" || exit 1
fi
_zacrs_line_pre_redraw() { :; }
_bench_probe() { print -r -- "$(( ++_bench_count )):$BUFFER" > "$HOME/probe"; }
_bench_ready() { print ready > "$HOME/ready"; }
_bench_defer() { _zacrs_defer_redraw; }
zle -N _bench_probe
zle -N _bench_defer
bindkey '^]' _bench_probe
bindkey '^O' _bench_defer
add-zle-hook-widget line-init _bench_ready
PROMPT='B> '
RPROMPT=''
