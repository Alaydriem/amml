# Shared by the ingame-* mise tasks (source it from the repo root, after scripts/game-pid.sh or before it).
# size_of and since read the lines a log got during one run; end_game ends the game the task started.

# the byte size of file $1; 0 when it does not exist
size_of() { stat -c %s "$1" 2> /dev/null || echo 0; }

# the bytes of file $1 after offset $2; the whole file when it is now shorter than $2 (it was replaced)
since() {
  [ -f "$1" ] || return 0
  local now
  now=$(size_of "$1")
  if [ "$now" -lt "$2" ]; then cat "$1"; else tail -c +$(( $2 + 1 )) "$1"; fi
}

# Ends the game this task started, and only that one: the PID recorded by find_game_pid (scripts/game-pid.sh)
# and checked again by pid_alive. It first posts WM_CLOSE so the game shuts down its own way, waits up to 20 s,
# then ends the PID. Safe to call twice (the explicit call and the EXIT trap).
end_game() {
  local folder="${game_dir:-${instance:-${game:-}}}"
  if [ -z "${game_pid:-}" ] && [ -n "${launched:-}" ] && [ -n "$folder" ]; then
    find_game_pid "$folder" || true
    if [ -z "${game_pid:-}" ]; then
      echo "no game PID recorded; if a game runs from $folder, close it by hand"
    fi
  fi
  if [ -n "${game_pid:-}" ] && pid_alive; then
    powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass \
      -File "$(cygpath -w "$PWD/scripts/close-game-window.ps1")" -ProcessId "$game_pid" > /dev/null 2>&1 || true
    local waited=0
    while [ "$waited" -lt 20 ] && pid_alive; do
      sleep 2
      waited=$(( waited + 2 ))
    done
    if pid_alive; then
      taskkill //PID "$game_pid" //F > /dev/null 2>&1 || true
      echo "ended game pid $game_pid"
    else
      echo "game pid $game_pid closed"
    fi
  fi
  game_pid=""
  launched=""
}
