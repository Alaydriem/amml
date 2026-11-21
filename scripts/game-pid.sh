# Shared by the ingame-* mise tasks (source it from the repo root).
# Needs $before (" pid pid ") set before the launch. Sets game_pid, game_created, game_dir and game_lookup_rc.

# every Minecraft.Windows.exe PID that runs now
game_pids() { tasklist //FO CSV //NH //FI "IMAGENAME eq Minecraft.Windows.exe" | grep -o '^"Minecraft.Windows.exe","[0-9]*"' | grep -o '[0-9]*"$' | tr -d '"' || true; }

# runs the helper; keeps its exit code (a pipe into tr would hide it) and drops the CR characters
game_ps() {
  local out="" rc=0
  out=$(powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/find-game-pid.ps1")" "$@") || rc=$?
  [ -z "$out" ] || printf '%s
' "${out//$'\r'/}"
  return "$rc"
}

# records the game spawned from folder $1 (new since $before, path inside $1, with its creation time); sets nothing when none.
# Sets game_lookup_rc to the lookup's exit code: a failed lookup (not 0) does not mean that no game runs
find_game_pid() {
  local out="" rc=0
  out=$(game_ps -Folder "$(cygpath -w "$1")" -Before "$before" 2> /dev/null) || rc=$?
  game_lookup_rc=$rc
  if [ "$rc" -ne 0 ]; then echo "PID lookup failed (powershell exit $rc); no game PID recorded"; return 0; fi
  out=$(echo "$out" | head -n 1)
  if [ -n "$out" ]; then game_pid="${out%% *}"; game_created="${out#* }"; game_dir="$1"; fi
}

# succeeds only while the recorded PID still is our Minecraft.Windows.exe: same name, path inside the
# launch folder and same creation time. A reused PID, or a lookup that fails, is not ours.
# exits the script unless a lookup ran and found no game spawned from folder $1; for the check before a run
require_no_game() {
  find_game_pid "$1"
  if [ "$game_lookup_rc" -ne 0 ]; then echo "cannot tell whether a game runs from $1 (the PID lookup failed); the run does not start"; exit 1; fi
  if [ -n "$game_pid" ]; then echo "a game already runs from $1 (pid $game_pid); close it first"; exit 1; fi
}

pid_alive() {
  [ -n "$game_pid" ] || return 1
  local rc=0
  game_ps -Folder "$(cygpath -w "$game_dir")" -ProcessId "$game_pid" -Created "$game_created" 2> /dev/null || rc=$?
  if [ "$rc" -eq 2 ]; then echo "PID check failed (powershell exit 2); treating pid $game_pid as not ours"; fi
  [ "$rc" -eq 0 ]
}
