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
# The harness sends this key and waits for the file before it starts a timer.
# The file is written from a redraw hook registered after the adapter's, so
# the adapter has finished with the key by then.
_bench_idle() { :; }
_bench_idle_done() { [[ $LASTWIDGET != _bench_idle ]] || print idle > "$HOME/idle-$BENCH_MODE"; }
zle -N _bench_idle
bindkey '^]' _bench_idle
# One key each to leave for another word position and to come back. Coming
# back is a key the daemon answers from its cache.
_bench_away() { _bench_saved=$BUFFER; BUFFER="${BUFFER%% *} away "; CURSOR=${#BUFFER}; }
_bench_back() { BUFFER=$_bench_saved; CURSOR=${#BUFFER}; }
zle -N _bench_away
zle -N _bench_back
bindkey '^O' _bench_away
bindkey '^P' _bench_back
add-zle-hook-widget line-pre-redraw _bench_idle_done
add-zle-hook-widget line-init _bench_ready
PROMPT='B> '
RPROMPT=''
