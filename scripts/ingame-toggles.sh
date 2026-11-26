# The body of the ingame-toggles mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-toggles`, which sets usage_instance, usage_toggles and usage_modmanager and runs it from the repo
# root. Builds toggles and modmanager in their checkouts, installs only those two into a standalone test instance (the
# instance's other mods are parked for the run and put back after), starts the toggles world amml-test-toggles and
# checks the toggles lines in the running game.
set -e
. scripts/ingame-instance.sh
resolve_test_instance "${usage_instance:-}"
toggles="${usage_toggles:-${AMML_TOGGLES_DIR:-../toggles}}"
[ -d "$toggles" ] || { echo "no toggles folder '$toggles' (--toggles, else AMML_TOGGLES_DIR, else ../toggles)"; exit 1; }
toggles="$(cd "$toggles" && pwd -P)"
[ -f "$toggles/mise.toml" ] || { echo "$toggles has no mise.toml: not a toggles checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# after the running-game check: the toggles world is seeded once and restored from its pristine copy
seed_mod_world toggles
# the posted focus loss must leave the game in play, so the next steps still run in the world
pause_on_focus_lost_off
launched=""
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/toggles
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
mise --cd "$toggles" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$toggles/dist/@alaydriem/toggles" ] || { echo "the toggles build wrote no $toggles/dist/@alaydriem/toggles"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$toggles/dist/@alaydriem/toggles" "$mods_root/@alaydriem/toggles"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/toggles from $toggles/dist"

for dist in "$modmanager/dist" "$toggles/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
toggles_log="$logs/@alaydriem/toggles.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
toggles_start=$(size_of "$toggles_log")
out_dir="target/ingame-toggles/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or toggles
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    toggles) since "$toggles_log" "$toggles_start" ;;
  esac
}
# the number of lines of this run of log $1 that match the extended regex $2
count_lines() { run_lines "$1" | grep -c -E "$2" || true; }
game_gone=""
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
# posts chord $1 (virtual-key codes in press order, comma separated) and waits up to 10 s for one more toggles line
# matching $2 than before; up to three tries, as a chord posted before the manager is ready is not seen
chord_for() {
  local keys="$1" pattern="$2" before_count try
  before_count=$(count_lines toggles "$pattern")
  for try in 1 2 3; do
    alive || return 1
    ps_run post-chord.ps1 -ProcessId "$game_pid" -Keys "$keys" || echo "posting chord $keys failed (try $try)"
    wait_line toggles "$pattern" 10 "$before_count" && return 0
  done
  return 1
}
key() { ps_run post-key.ps1 -ProcessId "$game_pid" -Key "$1" -Phase "${2:-press}" || echo "posting key $1 ${2:-press} failed"; }
shot() {
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/$1")" -ShotOnly | tr -d '\r' || echo "capturing $1 failed"
}
F3=114
W=87
S=83
SHIFT=16
ESCAPE=27
walk_on='toggle on kind=walk key=0x57'
sneak_on='toggle on kind=sneak key=0x10'

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
wait_line toggles "toggles linked" 180 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi
wait_line toggles "death check ready alive=true" 20 || true

if [ -n "$joined" ] && alive; then
  echo "--- auto-walk, then the S cancel key"
  if chord_for "$F3,$W" "$walk_on"; then
    sleep 1
    shot walk-1
    sleep 2
    shot walk-2
    key "$S"
    wait_line toggles 'toggle off kind=walk reason=cancel_key' 10 || true
  fi
  echo "--- sneak on and off by chord"
  if chord_for "$F3,$SHIFT" "$sneak_on"; then
    sleep 1
    chord_for "$F3,$SHIFT" 'toggle off kind=sneak reason=chord' || true
  fi
  echo "--- auto-walk, then the pause screen"
  if chord_for "$F3,$W" "$walk_on"; then
    sleep 1
    key "$ESCAPE"
    wait_line toggles 'toggle off kind=walk reason=menu' 10 || true
    sleep 1
    key "$ESCAPE"
    wait_line loader 'top screen name .*hud_screen' 10 || true
    sleep 2
  fi
  echo "--- auto-walk, then a focus loss"
  if chord_for "$F3,$W" "$walk_on"; then
    sleep 1
    ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
    wait_line toggles 'toggle off kind=walk reason=focus_lost' 10 || true
    sleep 2
  fi
  echo "--- sneak, then close the game"
  chord_for "$F3,$SHIFT" "$sneak_on" || true
  sleep 1
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/toggles.log (this run)"
run_lines toggles

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $toggles/dist"
check toggles "toggles linked" "toggles linked"
check toggles "death check ready alive=true" "death check ready alive=true (actor.read is_alive)"
check toggles "$walk_on" "auto-walk on (three chords)" 3
check toggles 'toggle off kind=walk reason=cancel_key' "auto-walk off reason=cancel_key"
check toggles "$sneak_on" "sneak on (two chords)" 2
check toggles 'toggle off kind=sneak reason=chord' "sneak off reason=chord"
check toggles 'toggle off kind=walk reason=menu' "auto-walk off reason=menu"
check toggles 'toggle off kind=walk reason=focus_lost' "auto-walk off reason=focus_lost"
check toggles 'toggle off kind=sneak reason=(world_left|disabled)' "sneak off reason=world_left or disabled at close"
check loader 'event offered .*"input\.key"' "event offered input.key"
check loader 'event offered .*"client\.tick"' "event offered client.tick"
check loader 'area adapter .*"input\.key_inject"' "area adapter input.key_inject"
check loader 'area adapter .*"actor\.read"' "area adapter actor.read"
for log in loader manager toggles; do
  bad=$(count_lines "$log" "event not available|area not available|key injection not available|loader too old|key refused|panic")
  if [ "$bad" -eq 0 ]; then
    echo "ok   no event, area, key injection, loader too old, key refused or panic line in the $log log"
  else
    echo "FAIL $bad event not available, area not available, key injection not available, loader too old, key refused or panic lines in the $log log"
    failures+=("$log log lines")
  fi
done
for name in walk-1 walk-2; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot (the view moves between the two, checked by hand): $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-toggles: pass"
else
  echo "ingame-toggles: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
