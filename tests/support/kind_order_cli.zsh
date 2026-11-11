set -eu
zmodload zsh/zpty

cat > "$HOME/cli-child.zsh" <<'ZSH'
stty rows 24 cols 80
print -r -- $'same\tnative-command\tcommand\nsame\tnative-file\tfile' |
    "$ZACRS_BIN" "$ZACRS_TEST_MODE" --prefix same --command-position
print -r -- "$?" > "$HOME/cli-status"
ZSH

for kind first second inserted in \
    command native-command expanded-result 'same ' \
    abbreviation expanded-result native-command 'expanded-result ' \
    file native-file native-command 'same '; do
    print -rl -- '[completion]' "kind_order = [\"$kind\"]" \
        '[abbreviations]' 'same = "expanded-result"' > "$XDG_CONFIG_HOME/zacrs/config.toml"
    for mode in render complete; do
        export ZACRS_TEST_MODE=$mode
        : > "$HOME/cli-status"
        zpty -b zacrs_order zsh -df "$HOME/cli-child.zsh"
        output=''
        sent=0
        for attempt in {1..500}; do
            chunk=''
            while zpty -r -t zacrs_order chunk 2>/dev/null; do
                output+="$chunk"
            done
            if [[ "$mode" == complete && "$sent" == 0 && "$output" == *'┘'* ]]; then
                zpty -w -n zacrs_order ' '
                sent=1
            fi
            [[ -s "$HOME/cli-status" ]] && break
            sleep .01
        done
        while zpty -r -t zacrs_order chunk 2>/dev/null; do
            output+="$chunk"
        done
        zpty -d zacrs_order
        # These paths run with a private runtime and no daemon. The direct
        # renderer and subprocess complete session must both honor the setting.
        setopt localoptions extendedglob
        plain=${output//$'\e'\[[0-9\;]#m/}
        [[ "$plain" == *"$first"*"$second"* ]] || {
            print -u2 -r -- "wrong $mode order for $kind: ${(qqq)plain}"
            exit 1
        }
        if [[ "$mode" == complete ]]; then
            [[ "$plain" == *"DONE 2 $inserted"* ]] || exit 1
            [[ "$(<"$HOME/cli-status")" == 0 ]] || exit 1
        else
            [[ "$(<"$HOME/cli-status")" == 0 ]] || exit 1
        fi
    done
done
print -r -- 'ok: custom kind order in direct render and subprocess complete'
