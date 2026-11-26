# The body of the ingame-levels mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-levels -- --instance <name>`, which sets usage_instance, usage_levels and usage_modmanager and runs
# it from the repo root. Builds levels and modmanager in their checkouts, installs only those two into a standalone
# test instance (the instance's other mods are parked for the run and put back after), runs them in the world
# amml-test-levels (a copy of amml-test with its own pristine copy) and checks the
# mod's and the loader's lines. The keys are posted messages (post-chord.ps1, post-key.ps1, post-char.ps1): nothing
# reaches any other window. Posted mouse messages never reach the game in the background, so no row needs the mouse.
#
# What the run proves (the pass lines are the mod's log lines, in the levels log unless noted):
#   loader        `event offered` for render.world_rendered; `area adapter` for world.scan_surfaces and
#                 render.world_canvas (version 2)
#   manager link  `levels: manager linked`
#   lease         F3+L gives `levels: world draw lease held`
#   scan          `levels: scan columns=<n> surfaces=<m>` with n > 0 and m > 0
#   mesh build    `levels: mesh built slot=label quads=<n>` with n > 0
#   draw          `levels: world drawn meshes=<n> vibrant=<0/1>` with n > 0 (the telemetry line, every 2 s). The run
#                 writes `telemetry = true` to <mods>/@alaydriem/levels/config/config.hcl of the fresh mod copy, or
#                 copies AMML_LEVELS_TELEMETRY_CONFIG there when it is set
#   borders       F3+G gives `levels: mesh built slot=border quads=<n>`
#   spheres       F3+J gives `levels: sphere dropped count=1`, then `levels: mesh built slot=shell quads=<n>`
#   sphere clear  F3+Alt+J gives `levels: spheres cleared`
#   dimension    a posted `/execute in nether run tp @s ~ 129 ~` (above the roof), then back at y 320: a new `levels: mesh built slot=border` and two `dimension changed, rescanning` lines (stale rebuild is attended, A5)
#   off           F3+L gives `levels: world draw lease released`
#   pixels        one pixel of a screenshot before and after F3+L (reported, not a pass by itself)
# Rows only a person can see are in the attended checklist of the levels port plan.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
levels="${usage_levels:-${AMML_LEVELS_DIR:-../levels}}"
[ -d "$levels" ] || { echo "no levels folder '$levels' (--levels, else AMML_LEVELS_DIR, else ../levels)"; exit 1; }
levels="$(cd "$levels" && pwd -P)"
[ -f "$levels/mise.toml" ] || { echo "$levels has no mise.toml: not a levels checkout"; exit 1; }
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
seed_mod_world levels
# the posted keys must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/levels
  # the run makes the @alaydriem folder; an empty one is not left behind
  [ -z "${mods_root:-}" ] || rmdir "$mods_root/@alaydriem" 2> /dev/null || true
  pause_on_focus_lost_restore
  [ -z "${telemetry_copied:-}" ] || rm -f "$telemetry_copied"
  [ -z "${telemetry_aside:-}" ] || mv "$telemetry_aside" "$telemetry_copied"
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
mise --cd "$levels" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$levels/dist/@alaydriem/levels" ] || { echo "the levels build wrote no $levels/dist/@alaydriem/levels"; exit 1; }

# a compat failure or a forbidden Win32 import warning stops the run before the game starts
mkdir -p target
for dist in "$modmanager/dist" "$levels/dist"; do
  compat_out="target/ingame-levels-compat-$(basename "$(dirname "$dist")").txt"
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
cp -r "$levels/dist/@alaydriem/levels" "$mods_root/@alaydriem/levels"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/levels from $levels/dist"
# the manager reads <mod>/config/config.hcl; the mod folder is a fresh copy of dist for this run (park_mods puts the
# operator's mods back on exit), so the run config is written into it and needs no clean-up
levels_config_dir="$mods_root/@alaydriem/levels/config"
mkdir -p "$levels_config_dir"
telemetry_config="$levels_config_dir/config.hcl"
if [ -n "${AMML_LEVELS_TELEMETRY_CONFIG:-}" ]; then
  [ -f "$AMML_LEVELS_TELEMETRY_CONFIG" ] || { echo "AMML_LEVELS_TELEMETRY_CONFIG '$AMML_LEVELS_TELEMETRY_CONFIG' is not a file"; exit 1; }
  cp "$AMML_LEVELS_TELEMETRY_CONFIG" "$telemetry_config"
else
  printf 'telemetry = true\n' > "$telemetry_config"
fi
telemetry_copied="$telemetry_config"
echo "wrote the levels run config (telemetry on) to $telemetry_config"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
levels_log="$logs/@alaydriem/levels.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
levels_start=$(size_of "$levels_log")
out_dir="target/ingame-levels/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "${out_dir:?}"/*.png
# logs rotate at about 1 MB: the lines of a run are the current log and every archive written since this marker
marker="$out_dir/.start"
touch "$marker"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or levels
run_lines() {
  case "$1" in
    loader) since_rotated "$logs/loader.log" "$loader_start" "$marker" ;;
    manager) since_rotated "$manager_log" "$manager_start" "$marker" ;;
    levels) since_rotated "$levels_log" "$levels_start" "$marker" ;;
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
# F3 (VK 0x72 = 114) then the key
f3_chord() { chord "114,$1"; }
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
# the colour "r g b" of one fixed pixel of screenshot $1, or empty
pixel() { ps_run png-pixel.ps1 -Path "$(cygpath -w "$PWD/$out_dir/$1.png")" -X 640 -Y 480 | tr -d '\r' || true; }

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
wait_line levels "levels: manager linked" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  echo "--- before the lease (screenshot)"
  shot off_before
  echo "--- lease on (F3+L)"
  for try in 1 2 3; do
    alive || break
    f3_chord 76
    wait_line levels "levels: world draw lease held" 8 && break
  done
  wait_line levels "levels: scan columns=[1-9][0-9]* surfaces=[1-9][0-9]*" 60 || true
  wait_line levels "levels: mesh built slot=label quads=[1-9][0-9]*" 60 || true
  [ -z "${telemetry_copied:-}" ] || wait_line levels "levels: world drawn meshes=[1-9][0-9]* vibrant=[01]" 60 || true
  sleep 1
  shot lease_on
  echo "--- borders (F3+G)"
  f3_chord 71
  wait_line levels "levels: mesh built slot=border quads=" 20 || true
  echo "--- sphere (F3+J)"
  f3_chord 74
  wait_line levels "levels: sphere dropped count=1" 15 || true
  wait_line levels "levels: mesh built slot=shell quads=" 20 || true
  echo "--- sphere clear (F3+Alt+J)"
  chord "114,18,74"
  wait_line levels "levels: spheres cleared" 15 || true
  echo "--- dimension change (nether and back): the plan changes, the border mesh is built again"
  built_before=$(count_lines levels "levels: mesh built slot=border")
  dim_before=$(count_lines levels "dimension changed, rescanning")
  # y 129 is above the Nether roof and y 320 is open air in the overworld: no suffocation, lava or fall damage in creative
  # a dimension change does not make meshes stale (the loader frees them at world leave, device reset, a graphics mode
  # switch and disable), so the proof here is a new border build and two rescans; the stale rebuild is attended (A5)
  chat_command "execute in nether run tp @s ~ 129 ~"
  # the Nether load screen can take about 20 s; the mod compares dimensions only on frames with a player
  sleep 20
  chat_command "execute in overworld run tp @s ~ 320 ~"
  wait_line levels "levels: mesh built slot=border" 40 "$built_before" || true
  built_after=$(count_lines levels "levels: mesh built slot=border")
  dim_after=$(count_lines levels "dimension changed, rescanning")
  echo "--- lease off (F3+L)"
  f3_chord 76
  wait_line levels "levels: world draw lease released" 15 || true
  sleep 1
  shot lease_off
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/levels.log (this run)"
run_lines levels

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $levels/dist"
check levels "levels: manager linked" "manager link"
check loader 'event offered .*"render\.world_rendered"' "event offered render.world_rendered"
check loader 'area adapter .*"world\.scan_surfaces"' "area adapter world.scan_surfaces"
# the catalog has one render.world_canvas row, version 2: the loader line names the area, compat checks the version
check loader 'area adapter .*"render\.world_canvas"' "area adapter render.world_canvas (version 2)"
check levels "levels: world draw lease held" "F3+L: world draw lease held"
check levels "levels: scan columns=[1-9][0-9]* surfaces=[1-9][0-9]*" "scan columns and surfaces above 0"
check levels "levels: mesh built slot=label quads=[1-9][0-9]*" "label mesh built (quads above 0)"
if [ -n "${telemetry_copied:-}" ]; then
  check levels "levels: world drawn meshes=[1-9][0-9]* vibrant=[01]" "world drawn (meshes above 0)"
else
  echo "note world drawn is not checked: it is the telemetry line; set AMML_LEVELS_TELEMETRY_CONFIG to a levels config with telemetry on to check it ($(count_lines levels 'levels: world drawn') seen)"
fi
check levels "levels: mesh built slot=border quads=" "F3+G: border mesh built"
check levels "levels: sphere dropped count=1" "F3+J: sphere dropped"
check levels "levels: mesh built slot=shell quads=" "sphere shell mesh built"
check levels "levels: spheres cleared" "F3+Alt+J: spheres cleared"
if [ "${built_after:-0}" -gt "${built_before:-0}" ]; then
  echo "ok   dimension change: border mesh built again ($built_before -> $built_after)"
else
  echo "FAIL dimension change: border mesh built again (levels log: ${built_before:-0} -> ${built_after:-0})"
  failures+=("dimension change: border mesh built again")
fi
# the mod rescans when LocalPlayer::dimension().id() changes: two lines (to the Nether and back) prove the loader's id
if [ "${dim_after:-0}" -ge $(( ${dim_before:-0} + 2 )) ]; then
  echo "ok   dimension change: rescans seen ($dim_before -> $dim_after)"
else
  echo "FAIL dimension change: rescans seen (levels log: ${dim_before:-0} -> ${dim_after:-0}, want 2 more)"
  failures+=("dimension change: rescans seen")
fi
echo "note stale rebuild is attended (A5: Vibrant Visuals on/off makes every mesh stale; $(count_lines levels 'levels: meshes stale, rebuilt') stale lines seen)"
check levels "levels: world draw lease released" "F3+L: world draw lease released"
for log in loader manager levels; do
  bad=$(count_lines "$log" "event not available|area not available|loader too old|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, too-old or panic line in the $log log"; else echo "FAIL $bad unavailable, too-old or panic lines in the $log log"; failures+=("$log log lines"); fi
done
# reported, not a pass by itself: the frame at one point differs with the lease on
if [ -f "$out_dir/off_before.png" ] && [ -f "$out_dir/lease_on.png" ]; then
  echo "pixel (640,480) before the lease: $(pixel off_before); with the lease: $(pixel lease_on); after: $(pixel lease_off)"
fi
for name in off_before lease_on lease_off; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot: $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-levels: pass"
else
  echo "ingame-levels: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
