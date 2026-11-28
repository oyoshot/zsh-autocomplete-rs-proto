# Minimal PowerShell adapter prototype (PSReadLine). All facts come from native APIs.
$proto = $env:ZACRS_PROTO_DIR
Set-PSReadLineKeyHandler -Chord Tab -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    # Fact: native candidates, with the range of the word they replace.
    $c = [System.Management.Automation.CommandCompletion]::CompleteInput($line, $cursor, $null)
    Set-Content -Path "$proto/gathered" -Value $c.CompletionMatches.Count
    if ($c.CompletionMatches.Count -eq 0) { return }
    $prefix = $line.Substring($c.ReplacementIndex, $c.ReplacementLength)
    $tsv = ($c.CompletionMatches | ForEach-Object {
        $tip = ($_.ToolTip -replace "[`r`n`t]", ' ')
        if ($tip.Length -gt 60) { $tip = $tip.Substring(0, 60) }
        "$($_.CompletionText)`t$tip`t"
    }) -join "`n"
    # Fact: terminal cursor position, native.
    $pos = $Host.UI.RawUI.CursorPosition
    $size = $Host.UI.RawUI.WindowSize
    $reply = @(($tsv + "`nEND`n") | & $env:ZACRS_BIN complete --prefix $prefix `
        --cursor-row $pos.Y --cursor-col $pos.X --cols $size.Width --rows $size.Height)
    Set-Content -Path "$proto/reply" -Value $reply
    $done = $reply[0] -split ' ', 3
    if ($done[0] -eq 'DONE' -and ($done[1] -eq '0' -or $done[1] -eq '2') -and $done[2]) {
        # Instruction: edit. Native replace of the native range.
        [Microsoft.PowerShell.PSConsoleReadLine]::Replace($c.ReplacementIndex, $c.ReplacementLength, $done[2])
    }
    [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
}
Set-PSReadLineKeyHandler -Chord Ctrl+y -ScriptBlock {
    $line = $null; $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
    Set-Content -Path "$proto/probe" -Value "$line|$cursor"
}
function prompt { '> ' }
