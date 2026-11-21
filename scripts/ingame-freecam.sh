# The body of the ingame-freecam mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-freecam -- --instance <name>`, which sets usage_instance, usage_freecam and usage_modmanager and
# runs it from the repo root. Builds freecam and modmanager in their checkouts, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after), runs them in the
# world amml-test-freecam (a copy of amml-test with its own pristine copy; cheats are on) and checks the mod's and the
# loader's lines. The keys are posted messages (post-key.ps1, post-chord.ps1, post-char.ps1, post-focus-loss.ps1):
# nothing reaches any other window. Held keys (W) reach the game as real key events, so the bind matcher sees them
# under the lease. Posted mouse messages never reach the game in the background, so no row needs the mouse.
#
# What the run proves (the pass lines are the mod's log lines, in the freecam log unless noted):
#   loader            `event offered` for camera.setup and camera.instruction_received; `camera setup thread` with game=true
#   link              `freecam linked`
#   camera watch      `camera watch kind=first_person`
#   flight            F3+P gives `freecam on free_camera=set`
#   keys under lease  W held for 1 s gives `flight moved distance=<d>` with d above 1
#   pose              `camera pose applied frames=<n>` with n above 0
#   markers           Ctrl+B twice gives `path point added count=2` and `markers applied delivered=<n>` with n above 0
#   look-at           the second Ctrl+B: `look-at ray answered hit=` is printed, not checked
#   server camera     attended (the flight lease blocks the chat keys); was: a posted `/camera @s set minecraft:third_person` while flying gives
#                     `flight stopped reason=server_camera origin=game`
#   screenshot        attended: the playback bar Screenshot button is a mouse click (no screenshot bind)
#   focus             post-focus-loss.ps1 while flying gives `flight keys cleared reason=focus_lost`
#   off               F3+P gives `freecam off free_camera=released`
# Rows only a person can see are in the attended checklist of the freecam port plan.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
freecam="${usage_freecam:-${AMML_FREECAM_DIR:-../freecam}}"
[ -d "$freecam" ] || { echo "no freecam folder '$freecam' (--freecam, else AMML_FREECAM_DIR, else ../freecam)"; exit 1; }
freecam="$(cd "$freecam" && pwd -P)"
[ -f "$freecam/mise.toml" ] || { echo "$freecam has no mise.toml: not a freecam checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# the shared amml-test world is only read here (restored from its pristine copy); the run plays in its own copy
seed_mod_world freecam
# the posted keys must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/freecam
  # the run makes the @alaydriem folder; an empty one is not left behind
  [ -z "${mods_root:-}" ] || rmdir "$mods_root/@alaydriem" 2> /dev/null || true
  pause_on_focus_lost_restore
  exit "$rc"
}
trap on_exit EXIT

cargo build -p amml-loader --release
cp target/release/amml.dll "$instance/amml.dll"
rm -rf "${instance:?}/amml/recipes" "${instance:?}/amml/data"
mkdir -p "$instance/amml/data" "$instance/amml/cache"
cp -r recipes "$instance/amml/recipes"
cp data/*.hcl "$instance/amml/data/"
mise --cd "$modmanager" run package
mise --cd "$freecam" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$freecam/dist/@alaydriem/freecam" ] || { echo "the freecam build wrote no $freecam/dist/@alaydriem/freecam"; exit 1; }

# a compat failure or a forbidden Win32 import warning stops the run before the game starts
mkdir -p target
for dist in "$modmanager/dist" "$freecam/dist"; do
  compat_out="target/ingame-freecam-compat-$(basename "$(dirname "$dist")").txt"
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" > "$compat_out" 2>&1 || compat_rc=$?
  cat "$compat_out"
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
  if grep -q -E "^warning: .* imports " "$compat_out"; then
    echo "compat warns of a forbidden import in $dist; the game is not started"
    exit 1
  fi
done

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$freecam/dist/@alaydriem/freecam" "$mods_root/@alaydriem/freecam"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/freecam from $freecam/dist"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
freecam_log="$logs/@alaydriem/freecam.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
freecam_start=$(size_of "$freecam_log")
out_dir="target/ingame-freecam/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "${out_dir:?}"/*.png
# logs rotate at about 1 MB: the lines of a run are the current log and every archive written since this marker
marker="$out_dir/.start"
touch "$marker"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or freecam
run_lines() {
  case "$1" in
    loader) since_rotated "$logs/loader.log" "$loader_start" "$marker" ;;
    manager) since_rotated "$manager_log" "$manager_start" "$marker" ;;
    freecam) since_rotated "$freecam_log" "$freecam_start" "$marker" ;;
  esac
}
# the number of lines of this run of log $1 that match the extended regex $2
count_lines() { run_lines "$1" | grep -c -E "$2" || true; }
game_gone=""
alive() {
  [ -z "$game_gone" ] || return 1
  if [ -n "$game_pid" ] && ! pid_alive; then
    game_gone="yes"
    game_pid=""
    echo "the game exited"
    return 1
  fi
  return 0
}
# waits up to $3 s until more than $4 lines of this run of log $1 match $2; false, with a line, when none came or the
# game exited
wait_line() {
  local waited=0 min="${4:-0}"
  while [ "$waited" -lt "$3" ]; do
    [ "$(count_lines "$1" "$2")" -gt "$min" ] && return 0
    if [ $(( waited % 5 )) -eq 0 ] && ! alive; then return 1; fi
    sleep 1
    waited=$(( waited + 1 ))
  done
  echo "no new '$2' line in the $1 log within $3 s"
  return 1
}
key() { ps_run post-key.ps1 -ProcessId "$game_pid" -Key "$1" || echo "posting key $1 failed"; }
chord() { ps_run post-chord.ps1 -ProcessId "$game_pid" -Keys "$1" || echo "posting chord $1 failed"; }
key_phase() { ps_run post-key.ps1 -ProcessId "$game_pid" -Key "$1" -Phase "$2" || echo "posting key $1 $2 failed"; }
# types one chat command: `/` opens the chat with the slash, then the text and Enter
chat_command() {
  local text="$1" i
  key 191
  sleep 1
  for (( i = 0; i < ${#text}; i++ )); do
    ps_run post-char.ps1 -ProcessId "$game_pid" -Char "${text:$i:1}" > /dev/null || echo "posting a character failed"
  done
  key 13
  sleep 1
}
shot() {
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/$1")" -ShotOnly | tr -d '\r' || echo "capturing $1 failed"
}

before=" $(echo $(game_pids)) "
game_pid=""
launched="yes"
(cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
started=$(date +%s)
while [ -z "$game_pid" ]; do
  find_game_pid "$instance"
  [ -n "$game_pid" ] && break
  if [ $(( $(date +%s) - started )) -ge 30 ]; then echo "the game did not start within 30 s"; game_gone="yes"; break; fi
  sleep 2
done
# The key feed of some builds is not ready until a real key message has shown its thread
if alive; then
  wait_line loader "game window subclassed" 30 || true
  for try in 1 2 3; do
    alive || break
    key 135
    ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed (try $try)"
    wait_line loader "key feed thread|key injection not available" 5 && break
  done
fi
echo "--- start"
wait_line manager "manager enabled" 180 || true
wait_line freecam "freecam linked" 120 || true
joined=""
off_before=0
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  wait_line freecam "camera watch kind=first_person" 30 || true
  echo "--- flight on (F3+P)"
  for try in 1 2 3; do
    alive || break
    chord "114,80"
    wait_line freecam "freecam on free_camera=set" 8 && break
  done
  echo "--- hold W for 1 s"
  key_phase 87 down
  sleep 1
  key_phase 87 up
  wait_line freecam "flight moved distance=" 10 || true
  wait_line freecam "camera pose applied frames=[1-9]" 15 || true
  echo "--- path markers (Ctrl+B twice)"
  chord "17,66"
  sleep 1
  chord "17,66"
  wait_line freecam "path point added count=2" 10 || true
  wait_line freecam "markers applied delivered=[1-9]" 15 || true
  wait_line freecam "look-at ray answered hit=" 10 || true
  echo "--- flight off (F3+P): the state is checked before each toggle"
  off_before=$(count_lines freecam "freecam off free_camera=released")
  chord "114,80"
  wait_line freecam "freecam off free_camera=released" 10 "$off_before" || true
  off_first=$(count_lines freecam "freecam off free_camera=released")
  echo "--- flight on again, then a focus loss"
  on_before=$(count_lines freecam "freecam on free_camera=set")
  chord "114,80"
  wait_line freecam "freecam on free_camera=set" 8 "$on_before" || true
  key_phase 87 down
  sleep 1
  # the background hand-back already left the game unfocused; a posted gain first makes the posted loss a change
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" -Active || echo "posting the focus gain failed"
  sleep 1
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
  wait_line freecam "flight keys cleared reason=focus_lost" 10 || true
  key_phase 87 up
  echo "--- flight off again (F3+P)"
  chord "114,80"
  wait_line freecam "freecam off free_camera=released" 10 "$off_first" || true
  sleep 1
  shot after
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/freecam.log (this run)"
run_lines freecam

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $freecam/dist"
check freecam "freecam linked" "link"
check loader 'event offered .*"camera\.setup"' "event offered camera.setup"
check loader 'event offered .*"camera\.instruction_received"' "event offered camera.instruction_received"
check loader 'camera setup thread.*game[=":]*true' "camera setup thread game=true"
check freecam "camera watch kind=first_person" "camera watch (first_person)"
check freecam "freecam on free_camera=set" "F3+P: flight on"
check freecam "flight moved distance=(1\.[0-9]*[1-9][0-9]*|[2-9][0-9.]*|[1-9][0-9]+)" "W held: flight moved above 1"
check freecam "camera pose applied frames=[1-9]" "camera pose applied (frames above 0)"
check freecam "path point added count=2" "Ctrl+B twice: two path points"
check freecam "markers applied delivered=[1-9]" "markers applied (delivered above 0)"
echo "info look-at: $(run_lines freecam | grep -E 'look-at ray answered hit=' | head -n 1 || true)"
echo "note screenshot is attended: freecam has no screenshot bind; the playback bar Screenshot button (bar mode F3+E, then a click) is a mouse row ($(count_lines freecam 'screenshot written ok=1') seen)"
echo "note server camera is attended: the flight lease blocks the chat keys, so a posted /camera command cannot be sent while flying ($(count_lines freecam 'flight stopped reason=server_camera') seen)"
check freecam "flight keys cleared reason=focus_lost" "focus loss: flight keys cleared"
check freecam "freecam off free_camera=released" "F3+P: flight off twice" $(( off_before + 2 ))
for log in loader manager freecam; do
  bad=$(count_lines "$log" "event not available|area not available|loader too old|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, too-old or panic line in the $log log"; else echo "FAIL $bad unavailable, too-old or panic lines in the $log log"; failures+=("$log log lines"); fi
done
[ -f "$out_dir/after.png" ] && echo "screenshot: $out_dir/after.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-freecam: pass"
else
  echo "ingame-freecam: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
