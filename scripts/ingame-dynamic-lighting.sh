# The body of the ingame-dynamic-lighting mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-dynamic-lighting`, which sets usage_instance, usage_dynamic_lighting and usage_modmanager and runs
# it from the repo root. Builds dynamic-lighting and modmanager in their checkouts, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after), runs them in the
# world amml-test-dynamic-lighting (a copy of amml-test with its own pristine copy) and checks the mod's and the
# loader's lighting lines.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
light="${usage_dynamic_lighting:-${AMML_DYNAMIC_LIGHTING_DIR:-../dynamic-lighting}}"
[ -d "$light" ] || { echo "no dynamic-lighting folder '$light' (--dynamic-lighting, else AMML_DYNAMIC_LIGHTING_DIR, else ../dynamic-lighting)"; exit 1; }
light="$(cd "$light" && pwd -P)"
[ -f "$light/mise.toml" ] || { echo "$light has no mise.toml: not a dynamic-lighting checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
# the grid fill hook the loader must pick on this build
case "$(lower "$(basename "$instance")")" in
  1.26.70.*) grid_fill="fillSubChunkGridNoMode" ;;
  *) grid_fill="fillSubChunkGrid" ;;
esac
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# the shared amml-test world is only read here (restored from its pristine copy); the run plays in its own copy
if declare -F seed_mod_world > /dev/null; then
  seed_mod_world dynamic-lighting
else
  pick_test_world
  own_world="amml-test-dynamic-lighting"
  for root in "${world_roots[@]}"; do
    if [ -d "$root/amml-test" ] && [ ! -d "$root/$own_world" ]; then
      cp -r "$root/amml-test" "$root/$own_world"
      echo "copied world amml-test to $root/$own_world"
    fi
  done
  world="$own_world"
  restore_test_world
fi
# the posted keys must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/dynamic-lighting
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
mise --cd "$light" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$light/dist/@alaydriem/dynamic-lighting" ] || { echo "the dynamic-lighting build wrote no $light/dist/@alaydriem/dynamic-lighting"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$light/dist/@alaydriem/dynamic-lighting" "$mods_root/@alaydriem/dynamic-lighting"
# the settings of the run: the defaults, written out, and the toggle on F3+D
mkdir -p "$mods_root/@alaydriem/dynamic-lighting/config"
printf 'lighting {\n  enabled = true\n  round = true\n  level_override = 0\n}\nkeys {\n  toggle = "F3+D"\n}\n' \
  > "$mods_root/@alaydriem/dynamic-lighting/config/config.hcl"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/dynamic-lighting from $light/dist"

for dist in "$modmanager/dist" "$light/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
light_log="$logs/@alaydriem/dynamic-lighting.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
light_start=$(size_of "$light_log")
out_dir="target/ingame-dynamic-lighting/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or light
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    light) since "$light_log" "$light_start" ;;
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
hand() { chat_command "replaceitem entity @s slot.weapon.$1 0 $2"; }
# the toggle chord: F3 (VK 0x72) then D (VK 0x44)
toggle() { ps_run post-chord.ps1 -ProcessId "$game_pid" -Keys 114,68 || echo "posting F3+D failed"; }
shot() {
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/$1")" -ShotOnly | tr -d '\r' || echo "capturing $1 failed"
}
set_line="light source set level=14 falloff=round block=yes"
# set counts and cleared counts seen at each step, for the summary
offhand_cleared=""
toggle_cleared_before=""

before=" $(echo $(game_pids)) "
game_pid=""
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
wait_line light "enabled" 60 || true
wait_line light "manager attached" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  echo "--- torch in the main hand"
  hand mainhand air
  hand offhand air
  # at night the held light shows on the terrain; the test world is in daylight
  chat_command "gamerule dodaylightcycle false"
  chat_command "time set midnight"
  sleep 2
  shot before
  hand mainhand torch
  wait_line light "$set_line" 20 || true
  wait_line loader "lighting rebuild" 20 || true
  sleep 2
  shot torch
  echo "--- torch in the off hand"
  hand offhand torch
  hand mainhand air
  sleep 3
  offhand_cleared=$(count_lines light "light source cleared")
  hand offhand air
  wait_line light "light source cleared" 20 "$offhand_cleared" || true
  echo "--- lava bucket (item name)"
  hand mainhand lava_bucket
  wait_line light "light source set level=15 falloff=round block=no" 20 || true
  echo "--- toggle"
  hand mainhand torch
  wait_line light "$set_line" 20 1 || true
  toggle_cleared_before=$(count_lines light "light source cleared")
  for try in 1 2 3; do
    alive || break
    toggle
    wait_line light "toggled off" 5 && break
  done
  wait_line light "light source cleared" 10 "$toggle_cleared_before" || true
  toggle
  wait_line light "toggled on" 10 || true
  wait_line light "$set_line" 20 2 || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/dynamic-lighting.log (this run)"
run_lines light

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $light/dist"
check light "enabled" "load"
check light "manager attached" "manager link"
check loader 'lighting adapter .*"terrain":"on"' "lighting adapter terrain=on"
check loader 'lighting adapter .*"entity":"on"' "lighting adapter entity=on"
check loader "lighting adapter .*\"vibrant\":\"$grid_fill\"" "lighting adapter vibrant=$grid_fill"
check loader 'lighting render thread .*"game":true' "renderLevel runs on the game thread"
check loader 'area adapter .*"inventory\.read"' "area adapter inventory.read"
check light "entity light: on" "entity light: on"
check light "vibrant colour: on" "vibrant colour: on"
check light "item names: on" "item names: on"
check light "$set_line" "light source set (torch, main hand), then twice more" 3
check loader 'lighting rebuild .*"boxes":[1-9]' "lighting rebuild boxes > 0"
cleared_after_off=$(count_lines light "light source cleared")
if [ -n "$offhand_cleared" ] && [ "$offhand_cleared" -eq 0 ] && [ "$cleared_after_off" -ge 1 ]; then
  echo "ok   off hand: the light stayed with the torch in the off hand only and cleared when it went"
else
  echo "FAIL off hand (cleared ${offhand_cleared:-not run} with the off-hand torch, $cleared_after_off at the end)"
  failures+=("off hand")
fi
check light "light source set level=15 falloff=round block=no" "lava bucket by name: level 15, no block"
check light "toggled off" "toggle off (F3+D)"
check light "toggled on" "toggle on (F3+D)"
if [ -n "$toggle_cleared_before" ] && [ "$(count_lines light "light source cleared")" -gt "$toggle_cleared_before" ]; then
  echo "ok   light source cleared after the toggle"
else
  echo "FAIL light source cleared after the toggle"
  failures+=("toggle cleared")
fi
# The run disables the mod only at shutdown, after the world leave has put every light out, so the loader's
# `lighting cleared` line cannot show here, and no light clear at shutdown is checked in game. The loader tests
# `disabling_a_mod_that_holds_a_light_clears_it` and `leaving_the_world_puts_the_lights_out_before_a_disable` cover
# both clears; a disable in a world is also an attended check in the manager.
echo "note no light clear is checked in game: loader tests, and attended (disable in the manager while holding a torch)"
for log in loader manager light; do
  bad=$(count_lines "$log" "event not available|area not available|area withdrawn|lighting rebuild refused|light source refused|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, withdrawn, refused or panic line in the $log log"; else echo "FAIL $bad unavailable, withdrawn, refused or panic lines in the $log log"; failures+=("$log log lines"); fi
done
for name in before torch; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot: $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-dynamic-lighting: pass"
else
  echo "ingame-dynamic-lighting: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
