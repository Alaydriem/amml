# Two modes. Exit code 2 and one line on stderr when the process list cannot be read.
#
# Find (default): prints "<pid> <creation>" of the first (oldest) Minecraft.Windows.exe that
#   (a) is not in -Before (space separated PIDs seen before the launch) and
#   (b) has an executable path inside -Folder.
#   Prints nothing when none matches.
# Check (-ProcessId and -Created): exit 0 when that PID still is a Minecraft.Windows.exe with a path
#   inside -Folder and the same creation time, else exit 1. This catches a reused PID.
#
# The folder is compared with a trailing backslash, so "x-ll" does not match "x-llx".
# A process whose path cannot be read is not ours.
param(
  [Parameter(Mandatory = $true)][string]$Folder,
  [string]$Before = '',
  [int]$ProcessId = 0,
  [string]$Created = ''
)
$prefix = $Folder.TrimEnd('\', '/') + '\'
try {
  $procs = @(Get-CimInstance Win32_Process -Filter "Name='Minecraft.Windows.exe'" -ErrorAction Stop)
} catch {
  [Console]::Error.WriteLine("process list failed: $($_.Exception.Message)")
  exit 2
}
$inside = $procs | Where-Object {
  $_.ExecutablePath -and $_.ExecutablePath.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)
}
function Stamp($p) { $p.CreationDate.ToUniversalTime().ToString('o') }
if ($ProcessId -gt 0) {
  $hit = $inside | Where-Object { $_.ProcessId -eq $ProcessId -and (Stamp $_) -eq $Created }
  if ($hit) { exit 0 } else { exit 1 }
}
$known = @($Before -split '\s+' | Where-Object { $_ })
$found = $inside | Where-Object { $known -notcontains [string]$_.ProcessId } |
  Sort-Object CreationDate | Select-Object -First 1
if ($found) { "$($found.ProcessId) $(Stamp $found)" }
