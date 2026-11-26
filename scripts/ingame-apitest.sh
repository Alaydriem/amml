# The body of the ingame-apitest mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-apitest`, which sets usage_instance, usage_world, usage_manual_world and usage_focus_loss and
# runs it from the repo root.
set -e
. scripts/ingame-instance.sh
manual=""
[ "${usage_manual_world:-false}" = "true" ] && manual="yes"
focus_loss=""
[ "${usage_focus_loss:-false}" = "true" ] && focus_loss="yes"
world="${usage_world:-}"
if [ -n "$manual" ] && [ -n "$world" ]; then echo "--world and --manual-world exclude each other"; exit 1; fi
case "$world" in
  */* | *\\* | .* | *:*) echo "--world must be one world folder name, got '$world'"; exit 1 ;;
esac
resolve_test_instance "${usage_instance:-}"
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# after the running-game check: the world is restored from its pristine copy
pick_test_world
# the game opens the pause screen on focus loss (the loader's open_pause_screen path) only with this option on;
# only the options files under data_dir are changed
for options in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftpe/options.txt; do
  [ -f "$options" ] || continue
  if grep -q '^pause_menu_on_focus_lost:0' "$options"; then
    sed -i 's/^pause_menu_on_focus_lost:0/pause_menu_on_focus_lost:1/' "$options"
    echo "set pause_menu_on_focus_lost:1 in $options"
  fi
done
cargo build -p amml-loader --release
cp target/release/amml.dll "$instance/amml.dll"
rm -rf "$instance/amml/recipes" "$instance/amml/data"
mkdir -p "$instance/amml/data" "$instance/amml/cache"
cp -r recipes "$instance/amml/recipes"
cp data/*.hcl "$instance/amml/data/"
mise run package-apitest --game "$instance"
logs="$instance/amml/logs"
mod_log="$logs/@amml/apitest.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
mod_start=$(size_of "$mod_log")
before=" $(echo $(game_pids)) "
game_pid=""
launched=""
trap 'end_game || echo "ending the game failed"' EXIT
# the variable is set for the game process only
if [ -z "$manual" ]; then
  (cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
else
  (cd "$instance" && AMML_TEST_BACKGROUND=1 ./Minecraft.Windows.exe > /dev/null 2>&1) &
fi
launched="yes"
started=$(date +%s)
world_wait=120
[ -n "$manual" ] && world_wait=300
ok_wait=480
outcome="timeout"
key_posted=""
joined_at=""
prompted=""
window_done=""
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the number of apitest lines of this run that match the extended regex $1
count_lines() { since "$mod_log" "$mod_start" | grep -c -E "$1" || true; }
# waits up to $2 s until more than $3 apitest lines of this run match $1; false, with a line, when none came
wait_line() {
  local waited=0
  while [ "$waited" -lt "$2" ]; do
    [ "$(count_lines "$1")" -gt "$3" ] && return 0
    sleep 1
    waited=$(( waited + 1 ))
  done
  echo "no new '$1' line within $2 s"
  return 1
}
# The window steps, once the apitest's own overlay checks and the in-world input checks are done (a minimise
# opens the pause screen and suspends the input lease): two resizes, a minimise and a restore, a posted click and
# WM_CHAR 'a', then the original window size again. Each step waits a bounded time for its apitest line.
window_steps() {
  local out outer n
  echo "--- window steps"
  n=$(count_lines " resized ok=true")
  out=$(ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width 1280 -Height 760) || echo "resizing failed"
  echo "$out"
  outer=$(printf '%s\n' "$out" | sed -n 's/.*client before=[^ ]* outer=\([0-9]*\)x\([0-9]*\) after=.*/\1 \2/p' | head -n 1)
  wait_line " resized ok=true" 15 "$n" || true
  n=$(count_lines " resized ok=true")
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width 1600 -Height 940 || echo "resizing failed"
  wait_line " resized ok=true" 15 "$n" || true
  n=$(count_lines " window size size=[0-9]+x[0-9]+ minimised=true")
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Minimise || echo "minimising failed"
  wait_line " window size size=[0-9]+x[0-9]+ minimised=true" 10 "$n" || true
  sleep 2
  n=$(count_lines " window restored")
  local after
  after=$(count_lines " resized ok=true .*after=restore")
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Restore || echo "restoring failed"
  wait_line " window restored" 15 "$n" || true
  # the game resizes its buffers after a restore only when it resized them at the minimise
  wait_line " resized ok=true .*after=restore" 5 "$after" || echo "no swap chain resize after the restore from minimised"
  ps_run post-click.ps1 -ProcessId "$game_pid" -RX 0.02 -RY 0.97 || echo "posting the click failed"
  wait_line " window pointer button=Left down=true" 10 0 || true
  ps_run post-char.ps1 -ProcessId "$game_pid" -Char a || echo "posting the character failed"
  wait_line " window char ch=a" 10 0 || true
  if [ -n "$outer" ]; then
    n=$(count_lines " resized ok=true")
    set -- $outer
    ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width "$1" -Height "$2" || echo "resizing back failed"
    wait_line " resized ok=true" 15 "$n" || true
  fi
}
while :; do
  now=$(date +%s)
  if since "$mod_log" "$mod_start" | grep -q "apitest ok"; then outcome="ok"; break; fi
  if [ -z "$game_pid" ]; then find_game_pid "$instance"; elif ! pid_alive; then outcome="game exited"; game_pid=""; break; fi
  if [ -z "$game_pid" ] && [ $(( now - started )) -ge 30 ]; then outcome="game did not start"; break; fi
  if [ -n "$game_pid" ] && [ -z "$key_posted" ] && [ $(( now - started )) -ge 10 ]; then
    ps_run post-key.ps1 -ProcessId "$game_pid" -Key 135 || echo "posting F24 failed"
    ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed"
    key_posted="yes"
  fi
  if [ -z "$joined_at" ]; then
    if since "$mod_log" "$mod_start" | grep -q "world joined"; then
      joined_at=$now
      [ -n "$game_pid" ] && { ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed"; }
    elif [ -n "$manual" ] && [ -z "$prompted" ]; then
      echo "enter a world in the game window by hand (waiting up to $world_wait s)"
      prompted="yes"
    elif [ $(( now - started )) -ge "$world_wait" ]; then
      outcome="no world joined"
      [ -z "$manual" ] && echo "no 'world joined' within $world_wait s; rerun with --manual-world and enter the world by hand"
      break
    fi
  elif [ $(( now - joined_at )) -ge "$ok_wait" ]; then
    break
  elif [ -z "$window_done" ] && [ -n "$game_pid" ] && [ "$(count_lines "overlay checks done")" -gt 0 ] \
    && [ "$(count_lines "pause opened|pause not seen|pause not opened|lease refused")" -gt 0 ]; then
    window_steps
    window_done="yes"
  fi
  sleep 2
done
echo "waited $(( $(date +%s) - started )) s: $outcome"
if [ "$outcome" = "ok" ] && [ -n "$focus_loss" ] && [ -n "$game_pid" ]; then
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
  waited=0
  while [ "$waited" -lt 15 ]; do
    if since "$mod_log" "$mod_start" | grep -q "injected released reason=FocusLost"; then break; fi
    sleep 1
    waited=$(( waited + 1 ))
  done
fi
end_game
echo "--- loader.log (this run)"
since "$logs/loader.log" "$loader_start"
echo "--- @amml/apitest.log (this run)"
since "$mod_log" "$mod_start"
echo "--- summary"
offered=$(since "$logs/loader.log" "$loader_start" | grep -c "event offered" || true)
missing=$(since "$logs/loader.log" "$loader_start" | grep -c -E "event not available|area not available" || true)
mod_missing=$(since "$mod_log" "$mod_start" | grep -c -E "event not available|area not available" || true)
echo "event offered lines: $offered, event or area not available lines: loader $missing, apitest $mod_missing"
since "$logs/loader.log" "$loader_start" | grep -E "area adapter|area withdrawn|key feed thread|key injection not available|test world|overlay|swap chain|resize|window|shutdown|capture|screenshot|surface|device|back buffer" || true
since "$mod_log" "$mod_start" | grep -E "listening|mods loaded|provider ready|frame gate open|client reads|world joined|hud|mouse|key|player move|move intent|turn seen|wheel seen|click seen|gamepad|pause|apitest|world left|injected released|disabled|failed|refused|not seen|overlay|square|frames presented|frame cost|resized|resizes seen|screenshot|capture|copy back buffer|surface|render device|window|request size|shutdown" || true
# The square's centre must be magenta in the WithOverlays screenshot and not in the GameOnly one
run_mod=$(since "$mod_log" "$mod_start")
centre=$(printf '%s\n' "$run_mod" | sed -n 's/.* square x=\([0-9]*\) y=\([0-9]*\) side=.*/\1 \2/p' | head -n 1)
pixels=""
magenta_game_only=""
magenta_with_overlays=""
if [ -n "$centre" ]; then
  set -- $centre
  for layers in GameOnly WithOverlays; do
    path=$(printf '%s\n' "$run_mod" | sed -n "s/.* screenshot asked layers=$layers id=[0-9]* path=\(.*\) {\"module.*/\1/p" | head -n 1)
    if [ -z "$path" ] || [ ! -f "$(cygpath -u "$path")" ]; then echo "screenshot pixel layers=$layers: no file"; continue; fi
    rgb=$(ps_run png-pixel.ps1 -Path "$path" -X "$1" -Y "$2" | tr -d '\r')
    magenta="false"
    case "$rgb" in 25[0-5]\ [0-5]\ 25[0-5]) magenta="true" ;; esac
    echo "screenshot pixel layers=$layers x=$1 y=$2 rgb=$rgb magenta=$magenta file=$path"
    [ "$layers" = "GameOnly" ] && magenta_game_only="$magenta"
    [ "$layers" = "WithOverlays" ] && magenta_with_overlays="$magenta"
  done
fi
[ "$magenta_with_overlays" = "true" ] && [ "$magenta_game_only" = "false" ] && pixels="ok"
echo "screenshot check: ${pixels:-failed} (WithOverlays magenta=${magenta_with_overlays:-none}, GameOnly magenta=${magenta_game_only:-none})"
# The Shutdown listener runs at WM_CLOSE; on a build whose window thread is not the game thread it may be queued
# and lost when the game exits first, so a missing line is a warning, not a failure
if [ "$(count_lines "shutdown game_alive")" -eq 0 ]; then echo "warning: no 'shutdown game_alive' line in the apitest log of this run"; fi
[ "$outcome" = "ok" ] && [ "$missing" -eq 0 ] && [ "$mod_missing" -eq 0 ] && [ "$pixels" = "ok" ]
