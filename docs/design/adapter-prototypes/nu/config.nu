# Minimal nushell adapter prototype. Candidates, their kind and the range they
# replace all come from the native `commandline complete`.
$env.config.show_banner = false
$env.config.keybindings = [
    { name: zacrs_tab, modifier: none, keycode: tab, mode: [emacs vi_insert vi_normal], event: { send: executehostcommand, cmd: "zacrs-tab" } }
    { name: zacrs_probe, modifier: control, keycode: char_y, mode: [emacs vi_insert vi_normal], event: { send: executehostcommand, cmd: "zacrs-probe" } }
]
def zacrs-tab [] {
    let dir = $env.ZACRS_PROTO_DIR
    let line = (commandline)
    let cur = (commandline get-cursor)
    let before = ($line | str substring 0..<$cur)
    let cands = ($before | commandline complete --detailed)
    ($cands | length | into string) | save -f $"($dir)/gathered"
    if ($cands | is-empty) { return }
    let span = $cands.0.span
    let prefix = ($before | str substring $span.start..<$span.end)
    let tsv = ($cands | each {|c| $"($c.value)\t($c.description? | default '')\t($c.kind? | default '')" } | str join "\n")
    let pos = (python3 $"($dir)/cursor_pos.py" | str trim | split row ' ')
    let size = (term size)
    let reply = ($"($tsv)\nEND\n" | ^$env.ZACRS_BIN complete --prefix $prefix --cursor-row $pos.0 --cursor-col $pos.1 --cols $size.columns --rows $size.rows | lines)
    $reply | str join "\n" | save -f $"($dir)/reply"
    let done = ($reply.0 | split row -n 3 ' ')
    if ($done | length) > 2 and $done.0 == 'DONE' and ($done.1 in ['0' '2']) and ($done.2 | is-not-empty) {
        commandline edit --replace $"($before | str substring 0..<$span.start)($done.2)($line | str substring $cur..)"
        commandline set-cursor ($span.start + ($done.2 | str length))
    }
}
def zacrs-probe [] { $"(commandline)|(commandline get-cursor)" | save -f $"($env.ZACRS_PROTO_DIR)/probe" }
$env.PROMPT_COMMAND = {|| '' }
$env.PROMPT_COMMAND_RIGHT = {|| '' }
$env.PROMPT_INDICATOR = '> '
