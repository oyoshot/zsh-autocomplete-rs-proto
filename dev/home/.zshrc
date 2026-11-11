autoload -Uz compinit
compinit

HISTFILE="$HOME/.zsh_history"
HISTSIZE=1000
SAVEHIST=1000
bindkey -e

source "$HOME/.local/share/zacrs/init.zsh"
PROMPT='%F{cyan}[zacrs-container]%f %~ %# '
