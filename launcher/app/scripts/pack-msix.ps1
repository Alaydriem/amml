# Packs the launcher into an unsigned MSIX with MakeAppx. The caller builds the release exe first
# (mise run launcher-app:build).
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Publisher,
    [Parameter(Mandatory = $true)][string]$PublisherDisplay,
    [Parameter(Mandatory = $true)][string]$Out
)

$ErrorActionPreference = "Stop"

$appRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Resolve-Path (Join-Path $appRoot "..\..")
$exe = Join-Path $repoRoot "target\release\amml-launcher-app.exe"
$i18n = Join-Path $appRoot "src-tauri\resources\i18n"
$msix = Join-Path $appRoot "src-tauri\msix"

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Version must be three numeric parts (26.63.3), got '$Version'"
}
foreach ($required in @($exe, $i18n, (Join-Path $msix "AppxManifest.xml"), (Join-Path $msix "Assets"))) {
    if (-not (Test-Path $required)) { throw "Missing input: $required" }
}

$makeAppx = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
    Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $makeAppx) { throw "makeappx.exe not found under C:\Program Files (x86)\Windows Kits\10\bin" }

$stage = Join-Path $repoRoot "target\msix-stage"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $stage "resources\i18n") | Out-Null
Copy-Item $exe $stage
Copy-Item (Join-Path $i18n "*") (Join-Path $stage "resources\i18n") -Recurse
Copy-Item (Join-Path $msix "Assets") $stage -Recurse

$manifest = Get-Content (Join-Path $msix "AppxManifest.xml") -Raw
$escName = [System.Security.SecurityElement]::Escape($Name)
$escPublisher = [System.Security.SecurityElement]::Escape($Publisher)
$escDisplay = [System.Security.SecurityElement]::Escape($PublisherDisplay)
$manifest = $manifest.Replace("__NAME__", $escName).Replace("__PUBLISHER__", $escPublisher).
    Replace("__PUBLISHER_DISPLAY__", $escDisplay).Replace("__VERSION__", "$Version.0")
[System.IO.File]::WriteAllText((Join-Path $stage "AppxManifest.xml"), $manifest, (New-Object System.Text.UTF8Encoding($false)))

$outDir = Split-Path -Parent $Out
if ($outDir) { New-Item -ItemType Directory -Force $outDir | Out-Null }
& $makeAppx.FullName pack /d $stage /p $Out /o
if ($LASTEXITCODE -ne 0) { throw "makeappx pack failed with exit code $LASTEXITCODE" }
Get-Item $Out | Format-List Name, Length
