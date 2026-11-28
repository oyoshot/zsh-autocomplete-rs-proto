HISTFILE="$HOME/history-$BENCH_MODE"
autoload -Uz compinit add-zle-hook-widget
compinit -i -D
source "$BENCH_PLUGIN" || exit 1
# Completers with a fixed list, so the timed interval is the adapter and the
# engine rather than whatever a system completer does.
typeset -ga _bench_small=( word{001..100} ) _bench_large=( word{00001..10000} )
_bench_small() { compadd -a _bench_small; }
_bench_large() { compadd -a _bench_large; }
compdef _bench_small bench-small
compdef _bench_large bench-large
_bench_ready() { print ready > "$HOME/ready-$BENCH_MODE"; }
add-zle-hook-widget line-init _bench_ready
PROMPT='B> '
RPROMPT=''
