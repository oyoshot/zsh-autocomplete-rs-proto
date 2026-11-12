autoload -Uz compinit
compinit

HISTFILE="$HOME/.zsh_history"
HISTSIZE=1000
SAVEHIST=1000
bindkey -e

source "$HOME/.local/share/zsh-autocomplete-rs/zsh-autocomplete-rs.plugin.zsh"
PROMPT='%F{cyan}[zacrs-container]%f %~ %# '
