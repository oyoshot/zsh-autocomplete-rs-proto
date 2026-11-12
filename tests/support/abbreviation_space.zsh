# Sourced by the real-ZLE harness with an isolated, running daemon.
if [[ "${ZACRS_TEST_SEARCH_CUSTOM:-0}" == 1 ]]; then
    enter_line 'bindkey -M isearch " " > "$HOME/search-binding"'
    [[ "$(<"$HOME/search-binding")" == *' backward-delete-char' ]] || fail 'preexisting isearch binding overwritten'
    enter_line 'bindkey -M vicmd "/" > "$HOME/vi-alias-binding"'
    [[ "$(<"$HOME/vi-alias-binding")" == *' _test_saved_search_backward' ]] || fail 'preexisting vi search alias binding changed'
    enter_line 'bindkey -M isearch " " undefined-key; _test_load_plugin'
fi

# Preserve the native insertion key name for every built-in search reader.
enter_line 'bindkey -M emacs " " > "$HOME/binding"; bindkey -M viins " " > "$HOME/vi-binding"'
[[ "$(<"$HOME/binding")" == *' self-insert' && "$(<"$HOME/vi-binding")" == *' self-insert' ]] || fail 'native Space bindings changed'

# Represent an existing suggestion insertion widget. Ordinary expanded Space
# must delegate its insertion and refresh; marker edits must skip insertion.
enter_line '_test_suggestions_enabled=1; _test_suggestion_history="gs old-suggestion"; _test_accept_suggestion() { if (( CURSOR == ${#BUFFER} )); then BUFFER+="$POSTDISPLAY"; POSTDISPLAY=""; CURSOR=${#BUFFER}; else zle _test_saved_forward; fi; }; zle -A forward-char _test_saved_forward; zle -N forward-char _test_accept_suggestion'
zpty -w -n zacrs_enter $'gs\x19'
wait_file "$HOME/editor" gs
wait_file "$HOME/postdisplay" ' old-suggestion'
zpty -w -n zacrs_enter $' \x19'
wait_file "$HOME/editor" 'git status '
wait_file "$HOME/inserted" 'git status '
wait_file "$HOME/postdisplay" ''
zpty -w -n zacrs_enter $'\e[C\x19'
wait_file "$HOME/editor" 'git status '
enter_line $'\x15'
enter_line '_test_suggestion_history="gcm old-suggestion"'
zpty -w -n zacrs_enter $'gcm\x19'
wait_file "$HOME/editor" gcm
wait_file "$HOME/postdisplay" ' old-suggestion'
: > "$HOME/inserted"
zpty -w -n zacrs_enter $' \x19'
wait_file "$HOME/editor" "git commit -m ''"
wait_file "$HOME/postdisplay" ''
[[ ! -s "$HOME/inserted" ]] || fail 'marker expansion invoked insertion'
enter_line $'\x15'
enter_line '_test_suggestions_enabled=0; zle -A _test_saved_forward forward-char'

# Deferred redraw and persistent Space dispatch share one saved delegate.
enter_line '_test_defer_space() { _zacrs_defer_redraw; _zacrs_restore_self_insert; }; zle -N _test_defer_space; bindkey "^O" _test_defer_space'
zpty -w -n zacrs_enter $'gs\x0f \x19'
wait_file "$HOME/editor" 'git status '
wait_file "$HOME/inserted" 'git status '
enter_line $'\x15'

# Zsh's bundled replacement widget edits two recursive minibuffers using main.
# Neither prompt is a command line, including when its text matches a trigger.
enter_line 'functions[_test_saved_request]=$functions[_zacrs_abbreviation_request]; _zacrs_abbreviation_request() { (( ZLE_RECURSIVE > 0 )) && print -r -- "$1" >> "$HOME/recursive-requests"; _test_saved_request "$@"; }'
enter_line 'autoload -Uz replace-string; zle -N replace-string; _test_replace() { BUFFER="gs tail"; CURSOR=${#BUFFER}; zle replace-string; print -r -- "$_replace_string_src" > "$HOME/replace-source"; print -r -- "$_replace_string_rep" > "$HOME/replace-result"; }; zle -N _test_replace; bindkey "^O" _test_replace'
zpty -w -n zacrs_enter $'\x0fgs \rgs value \r\x19'
wait_file "$HOME/editor" 'gs value tail'
wait_file "$HOME/replace-source" 'gs '
wait_file "$HOME/replace-result" 'gs value '
enter_line $'\x15'
# Cancelling the minibuffer must return to ordinary command editing.
: > "$HOME/editor"
zpty -w -n zacrs_enter $'\x0fgs \x07\x19'
wait_file "$HOME/editor" 'gs tail'
enter_line $'\x15'
zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'git status '
enter_line $'\x15'

[[ ! -s "$HOME/recursive-requests" ]] || fail 'recursive minibuffer sent abbreviation requests'
enter_line 'functions[_zacrs_abbreviation_request]=$functions[_test_saved_request]'

# Native completion suffixes must reach self-insert without a context query.
# Test removal and retention, including a completed word that is an abbr trigger.
enter_line '_test_native_suffix() { if (( _test_keep_suffix )); then compadd -S / -r : -- "$_test_suffix_word"; else compadd -S / -q -- "$_test_suffix_word"; fi; }; zle -C _test_native_suffix complete-word _test_native_suffix; bindkey "^O" _test_native_suffix'
for suffix_word in abc gs; do
    for keep_suffix in 0 1; do
        enter_line "_test_suffix_word=$suffix_word; _test_keep_suffix=$keep_suffix"
        : > "$HOME/editor"
        zpty -w -n zacrs_enter "${suffix_word[1]}"$'\x0f \x19'
        if (( keep_suffix )); then
            expected="$suffix_word/ "
        else
            expected="$suffix_word "
        fi
        wait_file "$HOME/editor" "$expected"
        wait_file "$HOME/cursor" "${#expected}"
        # The competing self-insert wrapper must still receive the Space.
        wait_file "$HOME/inserted" "$expected"
        enter_line $'\x15'
    done
done
enter_line '_test_suffix_word=abc; _test_keep_suffix=0'
zpty -w -n zacrs_enter $'echo a\x0f \x19'
wait_file "$HOME/editor" 'echo abc '
enter_line $'\x15'

# Numeric insertion counts retain native self-insert semantics.
for count expected in 0 gs 3 'gs   '; do
    : > "$HOME/editor"
    : > "$HOME/inserted"
    zpty -w -n zacrs_enter "gs"$'\e'"$count"$' \x19'
    wait_file "$HOME/editor" "$expected"
    wait_file "$HOME/cursor" "${#expected}"
    wait_file "$HOME/inserted" "$expected"
    enter_line $'\x15'
done

# Space extends incremental history search instead of invoking an edit widget.
# Push an isolated history list so only these entries can satisfy the search.
enter_line 'fc -p; print -s -- "echo needle hay"; print -s -- "gs needle"'
for query expected in 'echo ne' 'echo needle hay' 'gs ne' 'gs needle'; do
    : > "$HOME/editor"
    zpty -w -n zacrs_enter $'\x12'"$query"$'\x06\x19'
    wait_file "$HOME/editor" "$expected"
    enter_line $'\x15'
done
enter_line 'fc -P'

# An explicit isearch binding survives reload and disabling. Restore the
# plugin-owned search binding afterward to check ordinary disable cleanup too.
enter_line 'bindkey -M isearch " " backward-delete-char; _test_load_plugin; bindkey -M isearch " " > "$HOME/search-binding"'
[[ "$(<"$HOME/search-binding")" == *' backward-delete-char' ]] || fail 'custom isearch binding overwritten'
enter_line 'sed -i.bak "s/expand_abbreviation_on_space = true/expand_abbreviation_on_space = false/" "$XDG_CONFIG_HOME/zacrs/config.toml"; _test_load_plugin; bindkey -M isearch " " > "$HOME/search-binding"'
[[ "$(<"$HOME/search-binding")" == *' backward-delete-char' ]] || fail 'custom isearch binding changed on disable'
enter_line 'sed -i.bak "s/expand_abbreviation_on_space = false/expand_abbreviation_on_space = true/" "$XDG_CONFIG_HOME/zacrs/config.toml"; _test_load_plugin'
enter_line 'bindkey -M isearch " " undefined-key; _test_load_plugin; _test_load_plugin'

# Enter remains disabled throughout the first part.
enter_line gs
wait_file "$HOME/executed" 'gs:'
zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'git status '
wait_file "$HOME/cursor" 11
enter_line '-sb'
wait_file "$HOME/executed" 'git:status -sb'

# Ordinary spaces still delegate through another plugin's self-insert.
for input expected in 'echo arg' 'echo arg ' 'echo 日本語' 'echo 日本語 ' "'gs'" "'gs' " 'gs\' 'gs\ '; do
    : > "$HOME/editor"
    zpty -w -n zacrs_enter "$input"$' \x19'
    wait_file "$HOME/editor" "$expected"
    wait_file "$HOME/inserted" "$expected"
    enter_line $'\x15'
done

# Cursor in the middle of a command must not expand; at its end, preserve
# existing arguments and insert only the separator at that cursor.
zpty -w -n zacrs_enter $'gs-sb\x01\e[C \x19'
wait_file "$HOME/editor" 'g s-sb'
enter_line $'\x15'
enter_line '_test_middle() { BUFFER="gs -sb"; CURSOR=2; }; zle -N _test_middle; bindkey "^O" _test_middle'
zpty -w -n zacrs_enter $'\x0f \x19'
wait_file "$HOME/editor" 'git status  -sb'
wait_file "$HOME/cursor" 11
enter_line $'\x15'

# Marker expansions do not insert an unwanted space inside the template.
zpty -w -n zacrs_enter $'gcm \x19'
wait_file "$HOME/editor" "git commit -m ''"
wait_file "$HOME/cursor" 15
enter_line EDITED
wait_file "$HOME/executed" 'git:commit -m EDITED'

# An expansion that is itself a trigger remains protected on Space and Enter.
zpty -w -n zacrs_enter $'next \x19'
wait_file "$HOME/editor" 'gs '
enter_line ''
wait_file "$HOME/executed" 'gs:'
enter_line 'print -r -- "expand_abbreviation_on_enter = true" >> "$XDG_CONFIG_HOME/zacrs/config.toml"; _test_load_plugin'
zpty -w -n zacrs_enter $'next \x19'
wait_file "$HOME/editor" 'gs '
enter_line ''
wait_file "$HOME/executed" 'gs:'

# Enter expansion also declines recursive prompts, preserving the outer line's
# explicit native selection and its protection against a second expansion.
enter_line '_test_read_prompt() { local REPLY; read-from-minibuffer "Test: "; print -r -- "$REPLY" > "$HOME/minibuffer-reply"; }; zle -N _test_read_prompt; bindkey "^O" _test_read_prompt'
zpty -w -n zacrs_enter $'gs\x16\x0fgs \r\x19'
wait_file "$HOME/editor" 'gs '
wait_file "$HOME/minibuffer-reply" 'gs '
enter_line ''
wait_file "$HOME/executed" 'gs:'

# A non-recursive vared editor also edits text rather than a command.
zpty -w -n zacrs_enter '_test_vared_value=gs; vared _test_vared_value; print -r -- "$_test_vared_value" > "$HOME/vared-result"'$'\r \r'
wait_file "$HOME/vared-result" 'gs '

# Real popup Space keeps its own selection action with the new option active.
: > "$HOME/applied"
zpty -w -n zacrs_enter $'gs\x19'
wait_file "$HOME/editor" gs
pty_output=''
zpty -w -n zacrs_enter $'\t'
wait_popup gs
zpty -w -n zacrs_enter $'\e[B '
wait_file "$HOME/applied" 'gs '
enter_line ''
wait_file "$HOME/executed" 'gs:'

# Explicit popup selection protection survives Space, reload and Enter.
zpty -w -n zacrs_enter $'gs\x16 \x19'
wait_file "$HOME/editor" 'gs  '
enter_line ''
wait_file "$HOME/executed" 'gs:'

# A wrapper installed after zacrs must also receive fallback insertion.
enter_line 'zle -A self-insert _test_after_saved; _test_after() { zle _test_after_saved -w; print -r -- "$BUFFER" > "$HOME/after"; }; zle -N self-insert _test_after'
zpty -w -n zacrs_enter $'echo arg \x19'
wait_file "$HOME/editor" 'echo arg '
wait_file "$HOME/after" 'echo arg '
enter_line $'\x15'

# User Space bindings survive reloads; disabling restores our ordinary binding.
enter_line '_test_restore_space() { bindkey -M emacs " " self-insert; }; zle -N _test_restore_space; bindkey "^O" _test_restore_space; bindkey -M emacs " " _test_observe_buffer; _test_load_plugin; bindkey -M emacs " " > "$HOME/binding"'
[[ "$(<"$HOME/binding")" == *' _test_observe_buffer' ]] || fail 'custom Space binding overwritten'
zpty -w -n zacrs_enter $'\x0f'
enter_line '_test_load_plugin; _test_load_plugin'
for abort_key in $'\x03' $'\x07'; do
    zpty -w -n zacrs_enter $'next \x19'
    wait_file "$HOME/editor" 'gs '
    next_ready=$(( $(<"$HOME/ready") + 1 ))
    zpty -w -n zacrs_enter "$abort_key"
    wait_file "$HOME/ready" "$next_ready"
    zpty -w -n zacrs_enter $'gs \x19'
    wait_file "$HOME/editor" 'git status '
    enter_line $'\x15'
done
# Stop transport in the current line: preserve ordinary insertion, then
# recover only at the next line-init.
enter_line '_test_stop_daemon() { "$ZACRS_BIN" daemon stop; }; zle -N _test_stop_daemon; bindkey "^O" _test_stop_daemon'
zpty -w -n zacrs_enter $'gs\x0f \x19'
wait_file "$HOME/editor" 'gs '
enter_line $'\x15'

zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'git status '
enter_line $'\x15'

# vi insert mode has its own Space binding.
enter_line 'bindkey -v; bindkey -M viins "^Y" _test_observe_buffer; bindkey -M viins "^U" kill-whole-line; bindkey -M vicmd "/" vi-history-search-backward; bindkey -M vicmd "?" vi-history-search-forward'
zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'git status '
enter_line $'\x15'
# Vi search minibuffers use viins but reject user-defined insertion widgets.
# A later wrapper may capture ours. Reload must preserve it without recursion.
enter_line 'zle -A vi-history-search-backward _test_after_search_saved; _test_after_search() { zle _test_after_search_saved; print -r -- after > "$HOME/search-after"; }; zle -N vi-history-search-backward _test_after_search; _test_load_plugin; _test_load_plugin'
# Build history in an isolated list, then exercise both search directions.
enter_line 'bindkey -M vicmd "^B" beginning-of-buffer-or-history; fc -p; print -s -- "echo needle hay"; print -s -- "gs needle hay"'
for direction in / '?'; do
    : > "$HOME/editor"
    if [[ "$direction" == '?' ]]; then
        zpty -w -n zacrs_enter $'\e\x02'
    else
        zpty -w -n zacrs_enter $'\e'
    fi
    zpty -w -n zacrs_enter "$direction"$'needle h\ri\x19'
    wait_file "$HOME/editor" 'gs needle hay'
    if [[ "$direction" == / ]]; then
        wait_file "$HOME/search-before" backward
        wait_file "$HOME/search-after" after
    else
        wait_file "$HOME/search-before" forward
    fi
    # Cancel the recalled line and verify that ordinary expansion was restored.
    next_ready=$(( $(<"$HOME/ready") + 1 ))
    zpty -w -n zacrs_enter $'\x03'
    wait_file "$HOME/ready" "$next_ready"
    zpty -w -n zacrs_enter $'gs \x19'
    wait_file "$HOME/editor" 'git status '
    enter_line $'\x15'
done
# Native aliases captured before plugin load must also retain native Space.
# These names were saved by the competing wrapper before our first source.
enter_line 'bindkey -M vicmd "/" _test_saved_search_backward; bindkey -M vicmd "?" _test_saved_search_forward; _test_load_plugin; _test_load_plugin'
for direction in / '?'; do
    : > "$HOME/editor"
    if [[ "$direction" == '?' ]]; then
        zpty -w -n zacrs_enter $'\e\x02'
    else
        zpty -w -n zacrs_enter $'\e'
    fi
    zpty -w -n zacrs_enter "$direction"$'needle h\ri\x19'
    wait_file "$HOME/editor" 'gs needle hay'
    next_ready=$(( $(<"$HOME/ready") + 1 ))
    zpty -w -n zacrs_enter $'\x03'
    wait_file "$HOME/ready" "$next_ready"
done
enter_line 'bindkey -M vicmd "/" vi-history-search-backward; bindkey -M vicmd "?" vi-history-search-forward'

# Interrupt the minibuffer itself; the borrowed Space binding must be restored.
zpty -w -n zacrs_enter $'\e/needle '
next_ready=$(( $(<"$HOME/ready") + 1 ))
zpty -w -n zacrs_enter $'\x03'
wait_file "$HOME/ready" "$next_ready"
zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'git status '
enter_line $'\x15'

# User viins Space bindings are not borrowed or replaced during a search.
enter_line 'bindkey -M viins " " self-insert'
: > "$HOME/editor"
zpty -w -n zacrs_enter $'\e/needle h\ri\x19'
wait_file "$HOME/editor" 'gs needle hay'
zpty -w -n zacrs_enter $'\x15'
enter_line '_test_load_plugin'
enter_line 'fc -P'
enter_line 'bindkey -e'

enter_line 'sed -i.bak "s/expand_abbreviation_on_space = true/expand_abbreviation_on_space = false/" "$XDG_CONFIG_HOME/zacrs/config.toml"; _test_load_plugin; bindkey -M emacs " " > "$HOME/binding"'
[[ "$(<"$HOME/binding")" == *' self-insert' ]] || fail 'disabled Space binding not restored'
zpty -w -n zacrs_enter $'gs \x19'
wait_file "$HOME/editor" 'gs '
enter_line $'\x15'
enter_line 'bindkey -M isearch " " > "$HOME/search-binding"'
[[ "$(<"$HOME/search-binding")" == *' undefined-key' ]] || fail 'disabled isearch binding not restored'
# Disabling also removes our search wrappers from another plugin's saved aliases.
enter_line 'bindkey -v; fc -p; print -s -- "echo disabled search"'
: > "$HOME/editor"
zpty -w -n zacrs_enter $'\e/disabled s\ri\x19'
wait_file "$HOME/editor" 'echo disabled search'
enter_line $'\x15'
enter_line 'fc -P; bindkey -e'
print -r -- 'ok: Space expansion, independent flags, cursor, markers, selection, competing widgets, reload and cancellation'
zpty -w -n zacrs_enter $'exit\r'
