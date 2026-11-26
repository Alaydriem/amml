# Calls one tool of the headless-input MCP hub on 127.0.0.1 and prints the text of its result on one line, or
# `error: <text>` (exit 1). Used by scripts/ingame-headless-input.sh. Each call is its own MCP session: initialize,
# the initialized notice, then tools/call. With -Pair it asks POST /pair for a token instead and prints the token.
param(
    [Parameter(Mandatory = $true)][int]$Port,
    [string]$Token = "",
    [string]$Tool = "",
    [string]$ArgsJson = "{}",
    [switch]$Pair,
    [int]$TimeoutSec = 30
)
$ErrorActionPreference = "Stop"
$base = "http://127.0.0.1:$Port"

function Read-Rpc([string]$content) {
    # A Streamable HTTP answer is JSON or an event stream whose `data:` lines hold the JSON
    $text = $content.Trim()
    if ($text.StartsWith("{")) { return $text | ConvertFrom-Json }
    $data = ($content -split "`n") | Where-Object { $_ -like "data:*" } | ForEach-Object { $_.Substring(5).Trim() }
    foreach ($line in $data) {
        if ($line.StartsWith("{")) { return $line | ConvertFrom-Json }
    }
    return $null
}

try {
    if ($Pair) {
        $body = '{"client":"ingame-headless-input"}'
        $reply = Invoke-RestMethod -Method Post -Uri "$base/pair" -Body $body -ContentType "application/json" -TimeoutSec $TimeoutSec
        Write-Output $reply.token
        exit 0
    }
    $headers = @{ "Authorization" = "Bearer $Token"; "Accept" = "application/json, text/event-stream" }
    $init = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"ingame-headless-input","version":"1"}}}'
    $first = Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/mcp" -Headers $headers -Body $init -ContentType "application/json" -TimeoutSec $TimeoutSec
    $session = $first.Headers["Mcp-Session-Id"]
    if ($session -is [array]) { $session = $session[0] }
    if ($session) { $headers["Mcp-Session-Id"] = $session }
    $notice = '{"jsonrpc":"2.0","method":"notifications/initialized"}'
    Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/mcp" -Headers $headers -Body $notice -ContentType "application/json" -TimeoutSec $TimeoutSec | Out-Null
    $call = '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"' + $Tool + '","arguments":' + $ArgsJson + '}}'
    $answer = Invoke-WebRequest -UseBasicParsing -Method Post -Uri "$base/mcp" -Headers $headers -Body $call -ContentType "application/json" -TimeoutSec $TimeoutSec
    $rpc = Read-Rpc ([System.Text.Encoding]::UTF8.GetString($answer.RawContentStream.ToArray()))
    if ($null -eq $rpc) { Write-Output "error: no answer"; exit 1 }
    if ($rpc.error) { Write-Output ("error: " + $rpc.error.message); exit 1 }
    $texts = @($rpc.result.content | Where-Object { $_.type -eq "text" } | ForEach-Object { $_.text })
    $line = ($texts -join " ") -replace "[`r`n]+", " "
    if ($rpc.result.isError) { Write-Output "error: $line"; exit 1 }
    Write-Output $line
    exit 0
} catch {
    Write-Output ("error: " + $_.Exception.Message)
    exit 1
}
