# The body of the ingame-zoom mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-zoom`, which sets usage_instance, usage_zoom, usage_modmanager and usage_world and runs it from the
# repo root. Builds zoom and modmanager in their checkouts, installs only those two into a standalone test instance
# (the instance's other mods are parked for the run and put back after) and checks the zoom lines in the running game.
set -e
. scripts/ingame-instance.sh
manual=""
world="${usage_world:-}"
case "$world" in
  */* | *\\* | .* | *:*) echo "--world must be one world folder name, got '$world'"; exit 1 ;;
esac
resolve_test_instance "${usage_instance:-}"
zoom="${usage_zoom:-${AMML_ZOOM_DIR:-../zoom}}"
[ -d "$zoom" ] || { echo "no zoom folder '$zoom' (--zoom, else AMML_ZOOM_DIR, else ../zoom)"; exit 1; }
zoom="$(cd "$zoom" && pwd -P)"
[ -f "$zoom/mise.toml" ] || { echo "$zoom has no mise.toml: not a zoom checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# after the running-game check: the world is restored from its pristine copy
pick_test_world
# the posted focus loss must leave the game in play, so the zoom key can start a second zoom after it
pause_on_focus_lost_off
launched=""
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/zoom
  # the run makes the @alaydriem folder; an empty one is not left behind
  [ -z "${mods_root:-}" ] || rmdir "$mods_root/@alaydriem" 2> /dev/null || true
  pause_on_focus_lost_restore
  exit "$rc"
}
trap on_exit EXIT

cargo build -p amml-loader --release
cp target/release/amml.dll "$instance/amml.dll"
rm -rf "$instance/amml/recipes" "$instance/amml/data"
mkdir -p "$instance/amml/data" "$instance/amml/cache"
cp -r recipes "$instance/amml/recipes"
cp data/*.hcl "$instance/amml/data/"
mise --cd "$modmanager" run package
mise --cd "$zoom" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$zoom/dist/@alaydriem/zoom" ] || { echo "the zoom build wrote no $zoom/dist/@alaydriem/zoom"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$zoom/dist/@alaydriem/zoom" "$mods_root/@alaydriem/zoom"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/zoom from $zoom/dist"

for dist in "$modmanager/dist" "$zoom/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
zoom_log="$logs/@alaydriem/zoom.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
zoom_start=$(size_of "$zoom_log")
out_dir="target/ingame-zoom/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or zoom
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    zoom) since "$zoom_log" "$zoom_start" ;;
  esac
}
# the number of lines of this run of log $1 that match the extended regex $2
count_lines() { run_lines "$1" | grep -c -E "$2" || true; }
game_gone=""
# the number of `zoom ended` lines before the second zoom; empty when the second zoom did not run
ended_before_second=""
# false once the started game has exited (checked every few seconds by the waits)
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
# the zoom key: Grave (VK 0xC0), the zoom default
grave() { ps_run post-key.ps1 -ProcessId "$game_pid" -Key 192 -Phase "$1" || echo "posting Grave $1 failed"; }
shot() {
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/$1")" -ShotOnly | tr -d '\r' || echo "capturing $1 failed"
}

before=" $(echo $(game_pids)) "
game_pid=""
# the variable is set for the game process only
(cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
launched="yes"
started=$(date +%s)
while [ -z "$game_pid" ]; do
  find_game_pid "$instance"
  [ -n "$game_pid" ] && break
  if [ $(( $(date +%s) - started )) -ge 30 ]; then echo "the game did not start within 30 s"; game_gone="yes"; break; fi
  sleep 2
done
# The key feed of some builds is not ready until a real key message has shown its thread, so F24 and mouse moves go
# to the window as soon as the loader has subclassed it; F24 is posted again while the loader has not decided the key
# feed thread
if alive; then
  wait_line loader "game window subclassed" 30 || true
  for try in 1 2 3; do
    alive || break
    ps_run post-key.ps1 -ProcessId "$game_pid" -Key 135 || echo "posting F24 failed (try $try)"
    ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed (try $try)"
    wait_line loader "key feed thread|key injection not available" 5 && break
  done
fi
echo "--- start"
wait_line manager "manager enabled" 180 || true
wait_line zoom "zoom linked" 180 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

# The zoom steps: a held key zooms in, a focus loss resets the zoom, a second held key zooms again and its release
# ends the zoom
if [ -n "$joined" ] && alive; then
  echo "--- zoom"
  shot normal
  for try in 1 2 3; do
    alive || break
    grave down
    wait_line zoom "zoom started" 5 && break
  done
  wait_line zoom "zoom full" 5 || true
  shot zoomed
  # more shots of the held zoom: a frame that got the game field of view shows as a wider view than zoomed.png
  for n in 2 3 4; do
    sleep 0.3
    shot "zoomed-$n"
  done
  # mouse moves for the scaled look line
  ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed"
  echo "--- focus loss"
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
  wait_line zoom 'zoom reset .*focus_lost' 15 || true
  grave up
  sleep 2
  echo "--- second zoom"
  ended_before_second=$(count_lines zoom "zoom ended")
  grave down
  wait_line zoom "zoom started" 15 1 || true
  grave up
  wait_line zoom "zoom ended" 10 "$ended_before_second" || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/zoom.log (this run)"
run_lines zoom

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $zoom/dist"
check zoom "zoom linked" "zoom linked"
check zoom "zoom started" "two zoom started" 2
full=$(run_lines zoom | grep -m 1 -E "zoom full" || true)
# the zoom log writes the numbers into the message: `zoom full base=70.00 steady=70.00 result=17.50`
base=$(printf '%s' "$full" | sed -n 's/.* base=\([-0-9.]*\).*/\1/p')
result=$(printf '%s' "$full" | sed -n 's/.* result=\([-0-9.]*\).*/\1/p')
if [ -n "$base" ] && [ -n "$result" ] && awk -v b="$base" -v r="$result" 'BEGIN { exit !(r * 2 < b) }'; then
  echo "ok   zoom full base=$base result=$result (result * 2 < base)"
else
  echo "FAIL zoom full with result * 2 < base (line: '${full:-none}')"
  failures+=("zoom full")
fi
check zoom 'zoom reset .*focus_lost' "zoom reset reason=focus_lost"
# the first zoom already ends after the focus-loss reset, so the release of the second zoom must add a line
ended_after=$(count_lines zoom "zoom ended")
if [ -n "$ended_before_second" ] && [ "$ended_after" -gt "$ended_before_second" ]; then
  echo "ok   zoom ended after the second release ($ended_before_second before it, $ended_after after)"
else
  echo "FAIL zoom ended after the second release (${ended_before_second:-no second zoom} before it, $ended_after after)"
  failures+=("second zoom ended")
fi
check loader 'event offered .*"camera\.fov"' "event offered camera.fov"
check loader 'event offered .*"world\.dimension_changing"' "event offered world.dimension_changing"
check loader "camera fov thread" "camera fov thread"
for log in loader manager zoom; do
  bad=$(count_lines "$log" "event not available|area not available|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no event, area or panic line in the $log log"; else echo "FAIL $bad event not available, area not available or panic lines in the $log log"; failures+=("$log log lines"); fi
done
echo "camera fov thread: $(run_lines loader | grep -m 1 -o -E "camera fov thread.*" || echo none)"
echo "look speed (checked by hand): $(run_lines zoom | grep -m 1 -o -E "zoom look scaled.*" || echo "no zoom look scaled line")"
echo "look speed at full zoom (checked by hand): $(run_lines zoom | grep -m 1 -o -E "zoom look full.*" || echo "no zoom look full line")"
echo "dimension change (checked by hand): $(count_lines zoom 'zoom reset .*dimension') zoom reset reason=dimension, $(count_lines loader "dimension change thread") dimension change thread"
for name in normal zoomed zoomed-2 zoomed-3 zoomed-4; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot: $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-zoom: pass"
else
  echo "ingame-zoom: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
