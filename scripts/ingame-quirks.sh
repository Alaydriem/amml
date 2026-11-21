# The body of the ingame-quirks mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-quirks`, which sets usage_instance, usage_quirks, usage_modmanager, usage_idle_seconds and
# usage_skip_idle and runs it from the repo root. Builds quirks (twice: the shipped build and the test-no-keep-alive
# build) and modmanager in their checkouts, installs only those two into a standalone test instance (the instance's
# other mods are parked for the run and put back after) and checks the quirks lines in two game runs:
#   run A, the shipped build: the link, the setting, the first keep-alive fire and the loader's note_activity result;
#   run B, the test-no-keep-alive build: no input for --idle-seconds, the idle-screen ask refused.
set -e
. scripts/ingame-instance.sh
resolve_test_instance "${usage_instance:-}"
quirks="${usage_quirks:-${AMML_QUIRKS_DIR:-../quirks}}"
[ -d "$quirks" ] || { echo "no quirks folder '$quirks' (--quirks, else AMML_QUIRKS_DIR, else ../quirks)"; exit 1; }
quirks="$(cd "$quirks" && pwd -P)"
[ -f "$quirks/mise.toml" ] || { echo "$quirks has no mise.toml: not a quirks checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
idle_seconds="${usage_idle_seconds:-1080}"
case "$idle_seconds" in ''|*[!0-9]*) echo "--idle-seconds must be a whole number, got '$idle_seconds'"; exit 1 ;; esac
. scripts/game-pid.sh
. scripts/ingame-common.sh
game_pid=""
require_no_game "$instance"
# after the running-game check: the world amml-test-quirks is seeded from the pristine amml-test world and restored
seed_mod_world quirks
# the game window is not focused during the run; the world must keep running
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/quirks
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
mise --cd "$quirks" run package
mise --cd "$quirks" run package-test-no-keep-alive
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$quirks/dist/@alaydriem/quirks" ] || { echo "the quirks build wrote no $quirks/dist/@alaydriem/quirks"; exit 1; }
[ -d "$quirks/dist-test/@alaydriem/quirks" ] || { echo "the quirks test build wrote no $quirks/dist-test/@alaydriem/quirks"; exit 1; }
for dist in "$modmanager/dist" "$quirks/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
# installs the quirks build of folder $1 (dist or dist-test)
install_quirks() {
  rm -rf "$mods_root/@alaydriem/quirks"
  cp -r "$quirks/$1/@alaydriem/quirks" "$mods_root/@alaydriem/quirks"
  echo "installed @alaydriem/quirks from $quirks/$1"
}

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
quirks_log="$logs/@alaydriem/quirks.log"
mkdir -p "$logs"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of the current game run of log loader, manager or quirks
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    quirks) since "$quirks_log" "$quirks_start" ;;
  esac
}
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
# starts the game with the quirks world; the log marks of this run are taken first
start_game() {
  loader_start=$(size_of "$logs/loader.log")
  manager_start=$(size_of "$manager_log")
  quirks_start=$(size_of "$quirks_log")
  game_gone=""
  game_pid=""
  (cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
  local started
  started=$(date +%s)
  while [ -z "$game_pid" ]; do
    find_game_pid "$instance"
    [ -n "$game_pid" ] && break
    if [ $(( $(date +%s) - started )) -ge 30 ]; then echo "the game did not start within 30 s"; game_gone="yes"; return 1; fi
    sleep 2
  done
  wait_line manager "manager enabled" 180 || true
  wait_line quirks "attached to the mod manager" 180 || true
  joined=""
  if wait_line loader 'top screen name .*hud_screen' 180; then joined="yes"; fi
}
# keeps the lines of the run that just ended, for the summary
keep_lines() {
  run_lines loader > "$out_dir/$1-loader.log"
  run_lines manager > "$out_dir/$1-manager.log"
  run_lines quirks > "$out_dir/$1-quirks.log"
}
out_dir="target/ingame-quirks/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.log

echo "--- run A: the shipped build"
install_quirks dist
start_game || true
if [ -n "$joined" ] && alive; then
  # the first fire is due 60 s after enable
  wait_line quirks "idle keep-alive: fired" 120 || true
  wait_line loader "note_activity" 10 || true
fi
end_game
game_pid=""
sleep 2
keep_lines a

skip_idle="${usage_skip_idle:-}"
if [ -z "$skip_idle" ]; then
  echo "--- run B: the test-no-keep-alive build, no input for up to $idle_seconds s"
  restore_test_world
  install_quirks dist-test
  start_game || true
  if [ -n "$joined" ] && alive; then
    wait_line quirks "idle guard: refused the idle screen" "$idle_seconds" || true
    # later asks are refused too; the game must stay in the world
    sleep 20
  fi
  end_game
  game_pid=""
  sleep 2
  keep_lines b
fi

for run in a b; do
  for log in loader manager quirks; do
    [ -f "$out_dir/$run-$log.log" ] || continue
    echo "--- run $run: $log (this run)"
    cat "$out_dir/$run-$log.log"
  done
done

echo "--- summary"
failures=()
# counts the lines of saved run log $1 (a-loader, b-quirks, ...) that match $2
# (grep -c prints 0 and exits 1 on no match, so the fallback runs only for a missing file)
saved_count() { if [ -f "$out_dir/$1.log" ]; then grep -c -E "$2" "$out_dir/$1.log" || true; else echo 0; fi; }
check() {
  local log="$1" pattern="$2" label="$3" got
  got=$(saved_count "$log" "$pattern")
  if [ "$got" -ge 1 ]; then echo "ok   $label"; else echo "FAIL $label ($log: no line '$pattern')"; failures+=("$label"); fi
}
none() {
  local log="$1" pattern="$2" label="$3" got
  got=$(saved_count "$log" "$pattern")
  if [ "$got" -eq 0 ]; then echo "ok   $label"; else echo "FAIL $label ($log: $got lines '$pattern')"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $quirks/dist"
check a-quirks "attached to the mod manager" "A: quirks attached to the mod manager"
check a-quirks "idle keep-alive: on .*every_seconds=60" "A: idle keep-alive: on every_seconds=60"
check a-quirks "idle keep-alive: fired .*count=1" "A: idle keep-alive: fired count=1"
check a-loader 'note_activity .*"result":"ok"' "A: loader note_activity result=ok"
check a-loader 'area adapter .*client\.activity' "A: loader area adapter client.activity"
check a-loader 'event offered .*client\.idle_screen_open' "A: loader event offered client.idle_screen_open"
check a-quirks "idle keep-alive: off .*why=.?disabled" "A: idle keep-alive: off why=disabled at close"
if [ -z "$skip_idle" ]; then
  check b-quirks "this test build never fires" "B: the test build is installed"
  check b-quirks "idle guard: refused the idle screen" "B: idle guard: refused the idle screen"
  check b-loader "idle screen thread" "B: loader idle screen thread"
  none b-loader 'top screen name .*idle' "B: no idle screen shown"
  none b-quirks "idle keep-alive: fired" "B: no keep-alive fire in the test build"
else
  echo "skip run B (--skip-idle): the idle-screen refusal is not proven by this run"
  failures+=("run B skipped")
fi
for run in a b; do
  for log in loader manager quirks; do
    [ -f "$out_dir/$run-$log.log" ] || continue
    none "$run-$log" "event not available|area not available|panic" "$run: no event, area or panic line in the $log log"
  done
done
echo "idle screen thread: $(grep -m 1 -o -E 'idle screen thread.*' "$out_dir/b-loader.log" 2> /dev/null || echo none)"
echo "note_activity: $(grep -m 1 -o -E 'note_activity.*' "$out_dir/a-loader.log" 2> /dev/null || echo none)"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-quirks: pass"
else
  echo "ingame-quirks: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
