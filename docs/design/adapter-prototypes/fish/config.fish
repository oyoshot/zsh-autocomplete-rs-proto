# Minimal fish adapter prototype: facts in, one instruction out, existing engine binary.
function _zacrs_tab
    set -l token (commandline -ct)
    # Fact: native candidates for the command line up to the cursor ("text<TAB>description").
    set -l tsv
    for line in (complete -C (commandline -cp))
        set -l parts (string split -m1 \t -- $line)
        set -a tsv (printf '%s\t%s\t' $parts[1] "$parts[2]")
    end
    echo (count $tsv) > $ZACRS_PROTO_DIR/gathered
    test (count $tsv) -gt 0; or return
    # Fact: terminal cursor position.
    set -l pos (string split ' ' -- (python3 $ZACRS_PROTO_DIR/cursor_pos.py))
    set -l reply (begin; printf '%s\n' $tsv; echo END; end | $ZACRS_BIN complete --prefix "$token" \
        --cursor-row $pos[1] --cursor-col $pos[2] --cols $COLUMNS --rows $LINES)
    printf '%s\n' $reply > $ZACRS_PROTO_DIR/reply
    # Instruction: DONE <code> <text> / APPLY ...
    set -l done (string split -m2 ' ' -- $reply[1])
    if test "$done[1]" = DONE; and contains -- "$done[2]" 0 2; and test -n "$done[3]"
        commandline -rt -- $done[3]
    end
    commandline -f repaint
end
function _zacrs_probe
    printf '%s|%s\n' (commandline -b) (commandline -C) > $ZACRS_PROTO_DIR/probe
end
bind \t _zacrs_tab
bind ctrl-y _zacrs_probe
function fish_prompt; echo -n '> '; end
function fish_right_prompt; end
set -g fish_greeting
