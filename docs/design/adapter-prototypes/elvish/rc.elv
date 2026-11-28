# Minimal elvish adapter prototype. Words come from the native `edit:wordify`;
# candidates from a native completer the adapter has to choose itself.
use str
var dir = $E:ZACRS_PROTO_DIR
fn zacrs-tab {
    var line = $edit:current-command
    var dot = $edit:-dot
    var before = $line[..$dot]
    var words = [(edit:wordify $before)]
    if (or (== (count $words) 0) (str:has-suffix $before ' ')) { set words = [$@words ''] }
    var completer = $edit:complete-filename~
    if (has-key $edit:completion:arg-completer $words[0]) { set completer = $edit:completion:arg-completer[$words[0]] }
    var prefix = $words[-1]
    var tsv = ''
    var n = 0
    for c [($completer $@words)] {
        var text = $c
        if (not-eq (kind-of $c) string) { set text = $c[stem] }
        if (str:has-prefix $text $prefix) {
            set tsv = $tsv$text"\t\t\n"
            set n = (+ $n 1)
        }
    }
    print $n > $dir/gathered
    if (== $n 0) { return }
    var pos = [(str:split ' ' (python3 $dir/cursor_pos.py))]
    var reply = [(print $tsv"END\n" | (external $E:ZACRS_BIN) complete --prefix $prefix --cursor-row $pos[0] --cursor-col $pos[1] --cols 100 --rows 24)]
    print (str:join "\n" $reply) > $dir/reply
    var done = [(str:split &max=3 ' ' $reply[0])]
    if (and (> (count $done) 2) (eq $done[0] DONE) (has-value [0 2] $done[1]) (not-eq $done[2] '')) {
        var start = (- (count $before) (count $prefix))
        set edit:current-command = $before[..$start]$done[2]$line[$dot..]
        set edit:-dot = (+ $start (count $done[2]))
    }
    edit:redraw &full=$true
}
set edit:insert:binding[Tab] = $zacrs-tab~
set edit:insert:binding[Ctrl-Y] = { print $edit:current-command'|'$edit:-dot > $dir/probe }
set edit:prompt = { put '> ' }
set edit:rprompt = { }
