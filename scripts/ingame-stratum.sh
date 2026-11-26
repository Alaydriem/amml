# The body of the ingame-stratum mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-stratum -- --instance <name>`, which sets usage_instance, usage_stratum and usage_modmanager and
# runs it from the repo root. Builds modmanager and stratum (feature gate) in their checkouts, installs only those two
# into a standalone test instance (the instance's other mods are parked for the run and put back after), runs them in
# the world amml-test-stratum (a copy of amml-test with its own pristine copy) and checks the mod's and the loader's
# lines. The keys are posted messages: nothing reaches any other window. Posted mouse messages never reach the game in
# the background, so no row needs the mouse (the Easy Place hold and the library rows are attended).
#
# What the run proves (the pass lines are the mod's log lines, in the stratum log unless noted):
#   loader          `event offered` for render.world_rendered, render.block_entity_pass, world.block_changed,
#                   player.use_pressed, player.build_block and client.app_focus_lost; `area adapter` for every area of
#                   the stratum interface contract
#   core            `stratum linked`, `world joined epoch=`, `world key kind=local`
#   ghost           `gate mesh built quads=<n>` (n > 0), `gate drawn frames=<n>` (n >= 60),
#                   `gate block from_state ok name=minecraft:stone`, `gate rotated ok`
#   block entities  `gate block entity drawn kind=chest`, `gate skull drawn shape=<26.63 or 70.x>`
#   features        `feature on name=easy_place`, `feature on name=picker`
#   build           `build conditions local=true cheats=true operator=<bool> creative=true`; with operator=true:
#                   `gate build probe exact=` (> 0, no turn_mismatches), `gate build kinds count=` (> 0),
#                   `gate build command sent`, `gate block changed pos=`, `gate build block placed`; with
#                   operator=false the build rows are attended and do not fail the run
#   leave           closing the game gives `gate freed meshes=<n> reason=world_left`
# Printed, not checked: the first `app focus thread`, `actor permission read`, `build delay read`,
# `build block thread`, `kinds scan time`, `mesher build time` and `option key` lines of the run.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
stratum="${usage_stratum:-${AMML_STRATUM_DIR:-../stratum-build-replay}}"
[ -d "$stratum" ] || { echo "no stratum folder '$stratum' (--stratum, else AMML_STRATUM_DIR, else ../stratum-build-replay)"; exit 1; }
stratum="$(cd "$stratum" && pwd -P)"
[ -f "$stratum/mise.toml" ] || { echo "$stratum has no mise.toml: not a stratum checkout"; exit 1; }
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
seed_mod_world stratum
# the posted focus messages must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/stratum
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
mise --cd "$stratum" run package --features gate
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$stratum/dist/@alaydriem/stratum" ] || { echo "the stratum build wrote no $stratum/dist/@alaydriem/stratum"; exit 1; }

# a compat failure or a forbidden Win32 import warning stops the run before the game starts
mkdir -p target
for dist in "$modmanager/dist" "$stratum/dist"; do
  compat_out="target/ingame-stratum-compat-$(basename "$(dirname "$dist")").txt"
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
cp -r "$stratum/dist/@alaydriem/stratum" "$mods_root/@alaydriem/stratum"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/stratum (feature gate) from $stratum/dist"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
stratum_log="$logs/@alaydriem/stratum.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
stratum_start=$(size_of "$stratum_log")
out_dir="target/ingame-stratum/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "${out_dir:?}"/*.png
# logs rotate at about 1 MB: the lines of a run are the current log and every archive written since this marker
marker="$out_dir/.start"
touch "$marker"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or stratum
run_lines() {
  case "$1" in
    loader) since_rotated "$logs/loader.log" "$loader_start" "$marker" ;;
    manager) since_rotated "$manager_log" "$manager_start" "$marker" ;;
    stratum) since_rotated "$stratum_log" "$stratum_start" "$marker" ;;
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
wait_line stratum "stratum linked" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  wait_line stratum "world joined epoch=" 30 || true
  wait_line stratum "world key kind=local" 30 || true
  wait_line stratum "gate mesh built quads=[1-9][0-9]*" 30 || echo "FAIL wait: no 'gate mesh built' line with quads > 0"
  wait_line stratum "gate block from_state ok name=minecraft:stone" 30 || echo "FAIL wait: no 'gate block from_state ok' line"
  wait_line stratum "gate rotated ok" 30 || echo "FAIL wait: no 'gate rotated ok' line"
  wait_line stratum "gate drawn frames=(6[0-9]|[7-9][0-9]|[1-9][0-9]{2,})" 60 || echo "FAIL wait: no 'gate drawn' line with frames >= 60"
  wait_line stratum "gate block entity drawn kind=chest" 30 || echo "FAIL wait: no 'gate block entity drawn kind=chest' line"
  wait_line stratum "gate skull drawn shape=" 30 || echo "FAIL wait: no 'gate skull drawn' line"
  wait_line stratum "feature on name=easy_place" 30 || true
  wait_line stratum "feature on name=picker" 30 || true
  wait_line stratum "build conditions local=" 30 || true
  if [ "$(count_lines stratum 'build conditions .*operator=true')" -gt 0 ]; then
    wait_line stratum "gate build kinds count=[1-9][0-9]*" 60 || echo "FAIL wait: no 'gate build kinds' line with count > 0"
    wait_line stratum "gate build command sent" 30 || echo "FAIL wait: no 'gate build command sent' line"
    wait_line stratum "gate block changed pos=" 30 || echo "FAIL wait: no 'gate block changed' line"
    wait_line stratum "gate build block placed" 60 || echo "FAIL wait: no 'gate build block placed' line"
  fi
  shot ghost
  # one focus gain then one loss: a loss raises client.app_focus_lost only when the focus changes
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" -Active || echo "posting the focus gain failed"
  sleep 1
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
  sleep 2
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/stratum.log (this run)"
run_lines stratum

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $stratum/dist"
for event in render.world_rendered render.block_entity_pass world.block_changed player.use_pressed player.build_block client.app_focus_lost; do
  check loader "event offered .*\"${event//./\.}\"" "event offered $event"
done
for area in client.read client.world_info actor.read options.view_distance \
  render.world_canvas render.block_mesher world.block_states render.water_tint \
  render.block_entities render.chest_pair render.skull_pose \
  world.blocks player.build player.pick_range hud.select_hotbar inventory.read inventory.request \
  client.send_command actor.game_type actor.permission world.kinds_in_box input.lease player.hit_result; do
  check loader "area adapter .*\"${area//./\.}\"" "area adapter $area"
done
check stratum "stratum linked" "link"
check stratum "world joined epoch=" "world joined epoch"
check stratum "world key kind=local" "world key kind=local"
check stratum "gate mesh built quads=[1-9][0-9]*" "gate mesh built (quads > 0)"
check stratum "gate drawn frames=(6[0-9]|[7-9][0-9]|[1-9][0-9]{2,})" "gate drawn (frames 60 or more)"
check stratum "gate block from_state ok name=minecraft:stone" "gate block from_state ok"
check stratum "gate rotated ok" "gate rotated ok"
check stratum "gate block entity drawn kind=chest" "gate block entity drawn chest"
case "$(basename "$instance")" in
  *1.26.63*) skull_shape='26\.63' ;;
  *1.26.70*) skull_shape='70\.' ;;
  *) skull_shape='(26\.63|70\.)' ;;
esac
check stratum "gate skull drawn shape=$skull_shape" "gate skull drawn (shape for $(basename "$instance"))"
check stratum "feature on name=easy_place" "feature on easy_place"
check stratum "feature on name=picker" "feature on picker"
check stratum "build conditions local=true cheats=true operator=(true|false) creative=true" "build conditions (local, cheats, creative)"
if [ "$(count_lines stratum 'build conditions .*operator=true')" -gt 0 ]; then
  check stratum "gate build probe exact=[1-9][0-9]*" "gate build probe (exact > 0)"
  turn_all=$(count_lines stratum "gate build probe .*turn_mismatches=")
  turn_ok=$(count_lines stratum "gate build probe .*turn_mismatches=(0|\[\])([^0-9]|$)")
  turn_bad=$(( turn_all - turn_ok ))
  if [ "$turn_bad" -eq 0 ]; then echo "ok   gate build probe: no turn_mismatches"; else echo "FAIL gate build probe has turn_mismatches"; failures+=("turn_mismatches"); fi
  check stratum "gate build kinds count=[1-9][0-9]*" "gate build kinds (count > 0)"
  check stratum "gate build command sent" "gate build command sent"
  check stratum "gate block changed pos=" "gate block changed"
  check stratum "gate build block placed" "gate build block placed"
else
  echo "note build rows attended (operator=false)"
fi
check stratum "gate freed meshes=[0-9]+ reason=world_left" "close: gate freed (world_left)"
for log in loader manager stratum; do
  bad=$(count_lines "$log" "event not available|area not available|loader too old|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, too-old or panic line in the $log log"; else echo "FAIL $bad unavailable, too-old or panic lines in the $log log"; failures+=("$log log lines"); fi
done
# the first value of each lead-check line, loader log first then stratum log
for label in "app focus thread" "actor permission read" "build delay read" "build block thread" "kinds scan time" "mesher build time" "option key"; do
  first=$( { run_lines loader; run_lines stratum; } | grep -F -m 1 "$label" || true)
  if [ -n "$first" ]; then echo "info $label: $first"; else echo "info $label: no line in this run"; fi
done
[ -f "$out_dir/ghost.png" ] && echo "screenshot: $out_dir/ghost.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-stratum: pass"
else
  echo "ingame-stratum: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
