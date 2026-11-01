# Decompiles one function of a game executable with Ghidra headless.
# Usage (through mise): mise run decompile -- --exe <exe> --rva <hex> [--out <file>]
#
# The function start comes from `inspect function`. The first run for a BuildId imports the
# exe into %LOCALAPPDATA%\amml\ghidra\<BuildId> without analysis; later runs reuse that project.
# Only the process tree that this script started is ended.
$ErrorActionPreference = 'Stop'

$exe = $null; $rva = $null; $out = $null
for ($i = 0; $i -lt $args.Count; $i++) {
  switch ($args[$i]) {
    '--exe' { $exe = $args[++$i] }
    '--rva' { $rva = $args[++$i] }
    '--out' { $out = $args[++$i] }
    default { [Console]::Error.WriteLine("unknown argument: $($args[$i])"); exit 2 }
  }
}
if (-not $exe -or -not $rva) {
  [Console]::Error.WriteLine('usage: decompile --exe <exe> --rva <hex> [--out <file>]')
  exit 2
}
$exe = (Resolve-Path -LiteralPath $exe).Path

$repo = Split-Path -Parent $PSScriptRoot
$ghidraHome = if ($env:AMML_GHIDRA_HOME) { $env:AMML_GHIDRA_HOME } else { 'C:\Users\charl\tools\ghidra\ghidra_11.4.3_PUBLIC' }
$jdk = if ($env:AMML_GHIDRA_JAVA_HOME) { $env:AMML_GHIDRA_JAVA_HOME } else { 'C:\Users\charl\tools\jdk\jdk-21.0.12.1+1' }
$headless = Join-Path $ghidraHome 'support\analyzeHeadless.bat'
$scriptDir = Join-Path $repo 'ghidra'

function Invoke-Inspect([string[]]$InspectArgs) {
  $lines = & mise run inspect -- @InspectArgs 2>$null
  if ($LASTEXITCODE -ne 0) { throw "inspect $($InspectArgs -join ' ') failed" }
  return @($lines | Where-Object { $_ -and $_ -notmatch '^\[inspect\]' })
}

# Takes the import lock of a project, waiting at most WaitSeconds. The lock file holds the PID and
# creation time of its owner; a lock whose owner is gone is stale and is replaced.
function Get-ImportLock([string]$LockPath, [int]$WaitSeconds) {
  $deadline = [DateTime]::UtcNow.AddSeconds($WaitSeconds)
  $me = Get-Process -Id $PID
  while ($true) {
    try {
      $stream = [System.IO.File]::Open($LockPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
      $bytes = [System.Text.Encoding]::UTF8.GetBytes("$PID $($me.StartTime.Ticks)")
      $stream.Write($bytes, 0, $bytes.Length)
      $stream.Dispose()
      return
    }
    catch [System.IO.IOException] {
      if (-not (Test-Path -LiteralPath $LockPath)) { continue }
      $owner = ''
      try { $owner = (Get-Content -LiteralPath $LockPath -TotalCount 1) } catch { }
      $alive = $false
      if ($owner -match '^(\d+) (\d+)$') {
        $other = Get-Process -Id ([int]$Matches[1]) -ErrorAction SilentlyContinue
        if ($other -and $other.StartTime.Ticks -eq [long]$Matches[2]) { $alive = $true }
      }
      if (-not $alive -and $owner) { Remove-Item -LiteralPath $LockPath -Force -ErrorAction SilentlyContinue; continue }
      if ([DateTime]::UtcNow -ge $deadline) {
        throw "another import of this build holds $LockPath (owner: $owner); waited $WaitSeconds s. Wait for it to finish and run decompile again."
      }
      Start-Sleep -Seconds 2
    }
  }
}

# Adds the live descendants of the tracked processes to $Tracked (PID -> creation time). A child
# counts only when its parent is tracked, is alive now with the recorded creation time, and was
# created before the child, so a reused PID never joins the set.
function Update-Tracked([hashtable]$Tracked) {
  $all = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
  $live = @{}
  foreach ($p in $all) { $live[[int]$p.ProcessId] = $p.CreationDate }
  do {
    $added = $false
    foreach ($p in $all) {
      $id = [int]$p.ProcessId
      if ($Tracked.ContainsKey($id)) { continue }
      $parent = [int]$p.ParentProcessId
      if ($live.ContainsKey($parent) -and $Tracked.ContainsKey($parent) -and $live[$parent] -eq $Tracked[$parent] -and $p.CreationDate -ge $Tracked[$parent]) {
        $Tracked[$id] = $p.CreationDate; $added = $true
      }
    }
  } while ($added)
}

# Ends only the tracked processes whose creation time still matches.
function Stop-Tracked([hashtable]$Tracked) {
  foreach ($p in @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)) {
    $id = [int]$p.ProcessId
    if ($Tracked.ContainsKey($id) -and $p.CreationDate -eq $Tracked[$id]) {
      Stop-Process -Id $id -Force -ErrorAction SilentlyContinue
    }
  }
}

# Shows the last lines of a log in a failure message.
function Get-LogTail([string]$LogPath) {
  $text = ''
  foreach ($f in @($LogPath, "$LogPath.err")) {
    if (Test-Path -LiteralPath $f) { $text += "`n--- $f`n" + ((Get-Content -LiteralPath $f -Tail 15) -join "`n") }
  }
  return $text
}

# Runs analyzeHeadless and waits at most TimeoutSeconds. The process tree that it started is
# ended in `finally`, also on Ctrl+C or an error.
function Invoke-Headless([string[]]$HeadlessArgs, [int]$TimeoutSeconds, [string]$LogPath) {
  $env:JAVA_HOME = $jdk
  $quoted = ($HeadlessArgs | ForEach-Object { '"' + $_ + '"' }) -join ' '
  $tracked = @{}
  $proc = $null
  try {
    $proc = Start-Process -FilePath 'cmd.exe' -ArgumentList "/c `"`"$headless`" $quoted`"" `
      -PassThru -NoNewWindow -RedirectStandardOutput $LogPath -RedirectStandardError "$LogPath.err"
    $root = Get-CimInstance Win32_Process -Filter "ProcessId=$($proc.Id)" -ErrorAction SilentlyContinue
    if ($root) { $tracked[[int]$proc.Id] = $root.CreationDate }
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $done = $false
    while (-not $done -and [DateTime]::UtcNow -lt $deadline) {
      $done = $proc.WaitForExit(1000)
      if (-not $done) { Update-Tracked $tracked }
    }
    if (-not $done) { throw "ghidra timed out after $TimeoutSeconds s" }
    return $proc.ExitCode
  }
  finally {
    if ($proc) { Update-Tracked $tracked; Stop-Tracked $tracked }
  }
}

$buildId = (@(Invoke-Inspect @('build-id', '--exe', $exe)) | Select-Object -Last 1)
if (-not $buildId -or $buildId -notmatch '^[0-9a-f]{8}-[0-9a-f]{16}$') { throw "unexpected build id from inspect build-id: '$buildId'" }
# `inspect function` lists the primary range first and each chained fragment after it.
$listing = @(Invoke-Inspect @('function', '--exe', $exe, $rva))
$primary = @($listing | Where-Object { $_ -match '^primary 0x([0-9a-fA-F]+)\.\.0x' })
if ($primary.Count -ne 1) { throw "unexpected inspect function output: $($listing -join '; ')" }
$null = $primary[0] -match '^primary 0x([0-9a-fA-F]+)\.\.0x'
$startRva = '0x' + $Matches[1]

$projectDir = Join-Path $env:LOCALAPPDATA "amml\ghidra\$buildId"
$exeName = Split-Path -Leaf $exe
$marker = Join-Path $projectDir 'imported.txt'
New-Item -ItemType Directory -Force -Path $projectDir | Out-Null
$stamp = Join-Path $projectDir 'run'
New-Item -ItemType Directory -Force -Path $stamp | Out-Null

if (-not (Test-Path -LiteralPath $marker)) {
  $lock = Join-Path $stamp 'import.lock'
  Get-ImportLock $lock 1800
  try {
    # Another run may have finished the import while this one waited.
    if (-not (Test-Path -LiteralPath $marker)) {
      # A project left by a failed import is removed before a new import.
      Get-ChildItem -LiteralPath $projectDir -Force | Where-Object { $_.Name -ne 'run' } | Remove-Item -Recurse -Force
      $sw = [System.Diagnostics.Stopwatch]::StartNew()
      try { $code = Invoke-Headless @($projectDir, 'amml', '-import', $exe, '-noanalysis') 3600 (Join-Path $stamp 'import.log') }
      catch {
        $failure = $_
        try { Get-ChildItem -LiteralPath $projectDir -Force | Where-Object { $_.Name -ne 'run' } | Remove-Item -Recurse -Force } catch { }
        throw $failure
      }
      $sw.Stop()
      if ($code -ne 0) {
        Get-ChildItem -LiteralPath $projectDir -Force | Where-Object { $_.Name -ne 'run' } | Remove-Item -Recurse -Force
        throw "ghidra import failed with exit code $code$(Get-LogTail (Join-Path $stamp 'import.log'))"
      }
      Set-Content -LiteralPath $marker -Value $exeName
      [Console]::Error.WriteLine(("import: {0:N1} s" -f $sw.Elapsed.TotalSeconds))
    }
  }
  finally { Remove-Item -LiteralPath $lock -Force -ErrorAction SilentlyContinue }
}
$program = (Get-Content -LiteralPath $marker -TotalCount 1).Trim()

$target = if ($out) { $out } else { Join-Path $stamp 'decompiled.c' }
if (Test-Path -LiteralPath $target) { Remove-Item -LiteralPath $target -Force }
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$code = Invoke-Headless @($projectDir, 'amml', '-process', $program, '-noanalysis', '-readOnly',
    '-scriptPath', $scriptDir, '-postScript', 'DecompileAt.py', $startRva, $target) 600 (Join-Path $stamp 'decompile.log')
$sw.Stop()
if ($code -ne 0 -or -not (Test-Path -LiteralPath $target)) {
  throw "decompile failed (exit code $code)$(Get-LogTail (Join-Path $stamp 'decompile.log'))"
}
[Console]::Error.WriteLine(("decompile: {0:N1} s" -f $sw.Elapsed.TotalSeconds))
if (-not $out) { Get-Content -LiteralPath $target }
