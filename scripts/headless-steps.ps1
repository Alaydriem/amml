# Drives one game through LeviHeadlessInput (LeviLamina host only): waits for its pipe, waits, clicks each point,
# then presses each key and saves a screenshot after the last one. The game never takes the foreground or the real
# mouse and keyboard.
#
# -Clicks is "rx,ry;rx,ry" in fractions of the client area, with -ClickWait seconds after each. -Keys is "F5;..."
# (a LeviHeadlessInput key name, or look:<dy> for a camera turn). -Shot is the PNG written after the keys.
# Only call it for the game the ingame-* task started; the pipe name is the one in that instance's mod config.
param(
  [Parameter(Mandatory = $true)][string]$ModDir,
  [Parameter(Mandatory = $true)][string]$PipeName,
  [int]$WaitFirst = 0,
  [string]$Clicks = "",
  [int]$ClickWait = 6,
  [string]$Keys = "",
  [string]$Shot = ""
)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $ModDir 'HeadlessInput.psm1')
Connect-HeadlessInput -TimeoutSeconds 180 -PipeName $PipeName | Out-Null
if ($WaitFirst -gt 0) { Start-Sleep -Seconds $WaitFirst }
foreach ($pair in ($Clicks -split ';' | Where-Object { $_ })) {
  $xy = $pair -split ','
  Send-HeadlessMouse -Action click -RX ([double]$xy[0]) -RY ([double]$xy[1]) | Out-Null
  Write-Host "clicked $pair"
  Start-Sleep -Seconds $ClickWait
}
foreach ($key in ($Keys -split ';' | Where-Object { $_ })) {
  if ($key -like 'look:*') { Send-HeadlessRaw move -Dy ([int]($key.Substring(5))) | Out-Null } else { Send-HeadlessKey $key | Out-Null }
  Write-Host "pressed $key"
  Start-Sleep -Seconds 2
}
if ($Shot) {
  Save-HeadlessScreenshot $Shot | Out-Null
  Write-Host "saved $Shot"
}
