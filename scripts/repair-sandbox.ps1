# Makes an isolated copy of the repo for a timed test of a repair run.
# Usage (through mise): mise run repair-sandbox -- --name <name> --recipes <dir>
#                         [--hide-key <key.hcl> --hide-ids <id,id,...> [--hide-slots <n,n,...>]] [--force]
#
# The copy has no .git, target, reports, .superpowers, .claude or data. Junctions
# are not followed. recipes/ is
# replaced by <dir>. CLAUDE.md is copied. mise.local.toml points CARGO_TARGET_DIR at the main
# repo's target folder, so the copy does not rebuild the dependencies.
# With --hide-key, the sandbox is searched for the hidden answers and the script fails on a hit.
# --hide-slots adds slot numbers (decimal, or hex with 0x) that docs/recipes.md and CLAUDE.md must not hold,
# for example the vtable slots of hidden function entries; the vslot values of hidden ids are always
# checked.
$ErrorActionPreference = 'Stop'

$name = $null; $recipes = $null; $hideKey = $null; $hideIds = $null; $hideSlots = $null; $force = $false
for ($i = 0; $i -lt $args.Count; $i++) {
  switch ($args[$i]) {
    '--name' { $name = $args[++$i] }
    '--recipes' { $recipes = $args[++$i] }
    '--hide-key' { $hideKey = $args[++$i] }
    '--hide-ids' { $hideIds = $args[++$i] }
    '--hide-slots' { $hideSlots = $args[++$i] }
    '--force' { $force = $true }
    default { [Console]::Error.WriteLine("unknown argument: $($args[$i])"); exit 2 }
  }
}
if (-not $name -or -not $recipes) {
  [Console]::Error.WriteLine('usage: repair-sandbox --name <name> --recipes <dir> [--hide-key <key.hcl> --hide-ids <ids> [--hide-slots <n,...>]] [--force]')
  exit 2
}
if ($name -notmatch '^[A-Za-z0-9._-]+$' -or $name -match '^\.+$') { [Console]::Error.WriteLine("bad name: $name"); exit 2 }
if ([bool]$hideKey -ne [bool]$hideIds) { [Console]::Error.WriteLine('--hide-key and --hide-ids go together'); exit 2 }
if ($hideSlots -and -not $hideKey) { [Console]::Error.WriteLine('--hide-slots needs --hide-key and --hide-ids'); exit 2 }
$extraSlots = @()
foreach ($token in @($hideSlots -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
  if ($token -match '^0x([0-9a-fA-F]+)$') { $extraSlots += [Convert]::ToInt64($Matches[1], 16) }
  elseif ($token -match '^[0-9]+$') { $extraSlots += [int64]$token }
  else { [Console]::Error.WriteLine("bad slot in --hide-slots: $token"); exit 2 }
}
$recipes = (Resolve-Path -LiteralPath $recipes).Path

$repo = Split-Path -Parent $PSScriptRoot
$leakCheck = 'C:\Users\charl\notebook\Alaydriem\Minecraft\Addons\Levi Mods\AMML\findings\plan-8\test-sets\leak-check.ps1'
$root = Join-Path $env:LOCALAPPDATA 'amml\repair-sandbox'
$sandbox = Join-Path $root $name
# The sandbox must be a direct child of the sandbox root, or nothing is deleted.
$rootFull = [System.IO.Path]::GetFullPath($root).TrimEnd([char]92)
$sandboxFull = [System.IO.Path]::GetFullPath($sandbox).TrimEnd([char]92)
if ([System.IO.Path]::GetDirectoryName($sandboxFull) -ne $rootFull -or $sandboxFull -eq $rootFull) {
  [Console]::Error.WriteLine("sandbox path is not a direct child of $rootFull"); exit 2
}

if (Test-Path -LiteralPath $sandbox) {
  if (-not $force) { [Console]::Error.WriteLine("sandbox exists: $sandbox (use --force to replace it)"); exit 1 }
  Remove-Item -LiteralPath $sandbox -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $sandbox | Out-Null

$sw = [System.Diagnostics.Stopwatch]::StartNew()
# .claude holds a junction to skill/; the sandbox gets its own junction from `mise run install-skill`.
$skip = @('.git', 'target', 'reports', 'data', 'recipes', '.superpowers', '.claude') | ForEach-Object { Join-Path $repo $_ }
& robocopy $repo $sandbox /E /XJ /NFL /NDL /NJH /NJS /NP /XD @skip | Out-Null
if ($LASTEXITCODE -ge 8) { throw "robocopy failed with exit code $LASTEXITCODE" }
& robocopy $recipes (Join-Path $sandbox 'recipes') /E /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "robocopy failed with exit code $LASTEXITCODE" }
Copy-Item -LiteralPath (Join-Path $repo 'CLAUDE.md') -Destination (Join-Path $sandbox 'CLAUDE.md') -Force

$targetDir = (Join-Path $repo 'target').Replace('\', '/')
$localLines = @('[env]')
$mainLocal = Join-Path $repo 'mise.local.toml'
if (Test-Path -LiteralPath $mainLocal) {
  # Keep the core limits of the main repo (CARGO_BUILD_JOBS, NEXTEST_TEST_THREADS) in the sandbox.
  foreach ($line in Get-Content -LiteralPath $mainLocal) {
    if ($line -match '^\s*(CARGO_BUILD_JOBS|NEXTEST_TEST_THREADS)\s*=') { $localLines += $line.Trim() }
  }
}
$localLines += "CARGO_TARGET_DIR = `"$targetDir`""
Set-Content -LiteralPath (Join-Path $sandbox 'mise.local.toml') -Value (($localLines -join "`n") + "`n")

Push-Location $sandbox
try { & mise trust 2>&1 | Out-Null; if ($LASTEXITCODE -ne 0) { throw 'mise trust failed' } } finally { Pop-Location }

if ($hideKey) {
  $ids = @($hideIds -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
  & pwsh -NoProfile -File $leakCheck -Root $sandbox -Key $hideKey -Ids ($ids -join ',')
  if ($LASTEXITCODE -ne 0) { [Console]::Error.WriteLine('leak check failed'); exit 1 }

  # A bare slot number in the guide or in CLAUDE.md is also an answer: the vslot values of the hidden
  # ids, and the slots given with --hide-slots.
  $keyText = Get-Content -LiteralPath $hideKey -Raw
  $slots = @()
  foreach ($b in [regex]::Matches($keyText, 'resolved\s+"([^"]+)"\s*\{(.*?)\n\}', 'Singleline')) {
    if ($ids -notcontains $b.Groups[1].Value) { continue }
    if ($b.Groups[2].Value -notmatch 'kind\s*=\s*"vslot"') { continue }
    $hex = [regex]::Match($b.Groups[2].Value, 'value\s*=\s*"0x([0-9a-fA-F]+)"').Groups[1].Value
    if (-not $hex) { continue }
    $slots += [pscustomobject]@{ Label = $b.Groups[1].Value; Value = [Convert]::ToInt64($hex, 16) }
  }
  foreach ($n in $extraSlots) { $slots += [pscustomobject]@{ Label = '--hide-slots'; Value = $n } }
  $files = @(Get-Item -LiteralPath (Join-Path $sandbox 'docs/recipes.md'), (Join-Path $sandbox 'CLAUDE.md'))
  $found = 0
  foreach ($slot in $slots) {
    $re = [regex]::new("(?<![0-9A-Za-z])$($slot.Value)(?![0-9A-Za-z])")
    foreach ($f in $files) {
      $text = Get-Content -LiteralPath $f.FullName -Raw
      if ($text -and $re.IsMatch($text)) {
        Write-Output "$($f.FullName): $($slot.Label) (decimal $($slot.Value))"
        $found++
      }
    }
  }
  if ($found -gt 0) { [Console]::Error.WriteLine("$found decimal slot hit(s) in docs or CLAUDE.md"); exit 1 }
}
$sw.Stop()
Write-Output ("sandbox ready: {0} ({1:N1} s)" -f $sandbox, $sw.Elapsed.TotalSeconds)
