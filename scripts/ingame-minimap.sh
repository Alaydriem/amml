# The body of the ingame-minimap mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-minimap -- --instance <name>`, which sets usage_instance, usage_minimap and usage_modmanager and
# runs it from the repo root. Builds minimap and modmanager in their checkouts, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after), runs them in the
# world amml-test-minimap (a copy of amml-test with its own pristine copy) and checks the mod's and the loader's
# lines. The keys are posted messages (post-key.ps1, post-char.ps1): nothing reaches any other window. Posted mouse
# messages never reach the game in the background, so no row needs the mouse (drag and wheel zoom of the world map
# are attended rows).
#
# What the run proves (the pass lines are the mod's log lines, in the minimap log unless noted):
#   loader          `event offered` for world.block_changed; `area adapter` for world.chunk_read, world.map_color,
#                   world.biomes, client.world_info and window.cursor_clip
#   link            `minimap linked`
#   tick, chunks    `minimap world entered dimension=0`, then `minimap chunks read loaded=<n>` with n of 32 or more
#   world identity  `minimap world named kind=local folder=1`; after the run <test world>/@alaydriem/minimap/data exists
#   colours         `minimap colours source=vanilla`
#   biomes          `minimap biome named` with a name
#   block changes   a posted `/setblock ~2 ~ ~ stone` gives `minimap block changes on`, then `minimap block change column`
#   cursor clip     attended: the clip needs the game window in front, the scripted run is in the background
#                   (world map M open / M close / Escape close and the clip lines are in the attended checklist)
#   leave           closing the game gives `minimap world left`
#   screenshot      one screenshot of the game window (overlays included: it is a window capture)
# Printed, not checked: the mean and max of `probe_us_mean` from telemetry.csv (telemetry is on in the run config). Rows only a person can see are in the attended
# checklist of the minimap port plan.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
minimap="${usage_minimap:-${AMML_MINIMAP_DIR:-../minimap}}"
[ -d "$minimap" ] || { echo "no minimap folder '$minimap' (--minimap, else AMML_MINIMAP_DIR, else ../minimap)"; exit 1; }
minimap="$(cd "$minimap" && pwd -P)"
[ -f "$minimap/mise.toml" ] || { echo "$minimap has no mise.toml: not a minimap checkout"; exit 1; }
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
seed_mod_world minimap
# the posted keys must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/minimap
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
mise --cd "$minimap" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$minimap/dist/@alaydriem/minimap" ] || { echo "the minimap build wrote no $minimap/dist/@alaydriem/minimap"; exit 1; }

# a compat failure or a forbidden Win32 import warning stops the run before the game starts
mkdir -p target
for dist in "$modmanager/dist" "$minimap/dist"; do
  compat_out="target/ingame-minimap-compat-$(basename "$(dirname "$dist")").txt"
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
cp -r "$minimap/dist/@alaydriem/minimap" "$mods_root/@alaydriem/minimap"
# the manager reads <mod>/config/config.hcl; the mod folder is a fresh copy of dist for this run (park_mods puts the
# operator's mods back on exit). Telemetry on writes probe_us_mean to telemetry.csv in the mod data folder.
mkdir -p "$mods_root/@alaydriem/minimap/config"
printf 'debug {
  telemetry = true
}
' > "$mods_root/@alaydriem/minimap/config/config.hcl"
echo "wrote the minimap run config (telemetry on) to $mods_root/@alaydriem/minimap/config/config.hcl"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/minimap from $minimap/dist"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
minimap_log="$logs/@alaydriem/minimap.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
minimap_start=$(size_of "$minimap_log")
out_dir="target/ingame-minimap/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "${out_dir:?}"/*.png
# logs rotate at about 1 MB: the lines of a run are the current log and every archive written since this marker
marker="$out_dir/.start"
touch "$marker"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or minimap
run_lines() {
  case "$1" in
    loader) since_rotated "$logs/loader.log" "$loader_start" "$marker" ;;
    manager) since_rotated "$manager_log" "$manager_start" "$marker" ;;
    minimap) since_rotated "$minimap_log" "$minimap_start" "$marker" ;;
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
map_vk="${AMML_MINIMAP_MAP_VK:-77}"
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
wait_line minimap "minimap linked" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  wait_line minimap "minimap world entered dimension=0" 30 || true
  wait_line minimap "minimap chunks read loaded=(3[2-9]|[4-9][0-9]|[1-9][0-9]{2,})" 60 || true
  wait_line minimap "minimap world named kind=local folder=1" 30 || true
  wait_line minimap "minimap colours source=vanilla" 30 || true
  wait_line minimap "minimap biome named" 30 || true
  shot map
  echo "--- block change (setblock)"
  chat_command "setblock ~2 ~ ~ stone"
  wait_line minimap "minimap block changes on" 15 || true
  wait_line minimap "minimap block change column" 15 || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/minimap.log (this run)"
run_lines minimap

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $minimap/dist"
check minimap "minimap linked" "link"
check loader 'event offered .*"world\.block_changed"' "event offered world.block_changed"
for area in world.chunk_read world.map_color world.biomes client.world_info; do
  check loader "area adapter .*\"${area//./\.}\"" "area adapter $area"
done
check minimap "minimap world entered dimension=0" "world entered (dimension 0)"
check minimap "minimap chunks read loaded=(3[2-9]|[4-9][0-9]|[1-9][0-9]{2,})" "chunks read (loaded 32 or more)"
check minimap "minimap world named kind=local folder=1" "world named (local, folder)"
folder_hit=""
for root in "${world_roots[@]}"; do
  [ -d "$root/$world/@alaydriem/minimap/data" ] && folder_hit="$root/$world/@alaydriem/minimap/data"
done
if [ -n "$folder_hit" ]; then echo "ok   world data folder $folder_hit"; else echo "FAIL no @alaydriem/minimap/data folder in the world $world"; failures+=("world data folder"); fi
check minimap "minimap colours source=vanilla" "colours source vanilla"
check minimap "minimap biome named.*name[=:\" ]+[A-Za-z_]" "biome named with a name"
check minimap "minimap block changes on" "block changes on"
check minimap "minimap block change column" "block change column"
# the clip needs the game window in front (minimap asks for it only while Window::is_foreground()); the scripted run
# keeps the game in the background, so the clip and the world map close rows are attended
echo "note cursor clip and world map close rows are attended: the background run is never the foreground window ($(count_lines minimap 'minimap cursor clip') clip lines seen; window.cursor_clip is checked by compat)"
check minimap "minimap world left" "close: world left"
telemetry_csv=$(find "${folder_hit:-/nonexistent}" "$mods_root/@alaydriem/minimap" -name telemetry.csv -newer "$marker" 2>/dev/null | head -n 1)
if [ -n "$telemetry_csv" ]; then
  echo "info probe_us_mean ($telemetry_csv): $(awk -F, 'NR==1{for(i=1;i<=NF;i++) if($i=="probe_us_mean") c=i; next} c&&$c!=""{s+=$c; n++; if($c>m) m=$c} END{if(n) printf "mean %.1f us, max %.1f us, %d rows", s/n, m, n; else print "no rows"}' "$telemetry_csv")"
else
  echo "info probe_us_mean: no telemetry.csv written in this run"
fi
for log in loader manager minimap; do
  bad=$(count_lines "$log" "event not available|area not available|loader too old|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, too-old or panic line in the $log log"; else echo "FAIL $bad unavailable, too-old or panic lines in the $log log"; failures+=("$log log lines"); fi
done
[ -f "$out_dir/map.png" ] && echo "screenshot: $out_dir/map.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-minimap: pass"
else
  echo "ingame-minimap: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
