# The body of the ingame-durability-ping mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-durability-ping`, which sets usage_instance, usage_durability_ping and usage_modmanager and runs it
# from the repo root. Builds durability-ping and modmanager in their checkouts, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after), runs them in the
# world amml-test-durability-ping (a copy of amml-test with its own pristine copy) and checks the mod's lines.
set -e
. scripts/ingame-instance.sh
manual=""
world=""
resolve_test_instance "${usage_instance:-}"
ping="${usage_durability_ping:-${AMML_DURABILITY_PING_DIR:-../durability-ping}}"
[ -d "$ping" ] || { echo "no durability-ping folder '$ping' (--durability-ping, else AMML_DURABILITY_PING_DIR, else ../durability-ping)"; exit 1; }
ping="$(cd "$ping" && pwd -P)"
[ -f "$ping/mise.toml" ] || { echo "$ping has no mise.toml: not a durability-ping checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# the run plays in its own world amml-test-durability-ping, seeded once from amml-test and restored from its own
# pristine copy; the shared amml-test world is not changed
seed_mod_world durability-ping
# the posted keys and clicks must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/durability-ping
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
mise --cd "$ping" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$ping/dist/@alaydriem/durability-ping" ] || { echo "the durability-ping build wrote no $ping/dist/@alaydriem/durability-ping"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$ping/dist/@alaydriem/durability-ping" "$mods_root/@alaydriem/durability-ping"
# the settings of the run: every tool is low, so the alert plays for the given pickaxe and the guard refuses its hit
mkdir -p "$mods_root/@alaydriem/durability-ping/config"
printf 'alert {\n  enabled = true\n  percent = 0\n  uses = 2000\n  volume = 0\n}\nblock {\n  enabled = true\n  percent = 0\n  uses = 2000\n}\n' \
  > "$mods_root/@alaydriem/durability-ping/config/config.hcl"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/durability-ping from $ping/dist"

for dist in "$modmanager/dist" "$ping/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
ping_log="$logs/@alaydriem/durability-ping.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
ping_start=$(size_of "$ping_log")
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or ping
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    ping) since "$ping_log" "$ping_start" ;;
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
wait_line ping "durability-ping enabled" 60 || true
wait_line ping "attached to the mod manager" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

# The steps: a damaged pickaxe into the hand (the alert), the view straight down, then left clicks on the block under
# the player (the guard refuses each hit)
if [ -n "$joined" ] && alive; then
  echo "--- pickaxe"
  wait_line ping "held read slot=" 20 || true
  chat_command "replaceitem entity @s slot.weapon.mainhand 0 iron_pickaxe 1 245"
  wait_line ping "alert played" 20 || true
  # A hit on a block needs the game's mouse feed: a posted mouse message does not reach it in play, and the loader's
  # mouse injection waits for a real mouse feed (as in ingame-apitest), so the BlockBreakStart raise and the guard
  # refusal are attended rows (P2-18); the clicks are posted anyway and their lines are printed, not checked
  echo "--- hit (attended rows; printed, not checked)"
  chat_command "tp @s ~ ~ ~ 0 90"
  for try in 1 2 3; do
    alive || break
    ps_run post-click.ps1 -ProcessId "$game_pid" -RX 0.5 -RY 0.5 > /dev/null || echo "posting a click failed (try $try)"
    sleep 2
  done
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/durability-ping.log (this run)"
run_lines ping

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $ping/dist"
check ping "@alaydriem/durability-ping enabled" "load"
check ping "@alaydriem/durability-ping attached to the mod manager" "manager link"
for event in block_break_start block_break_continue item_use attack item_use_on_block; do
  check loader "event offered .*\"player\.$event\"" "event offered player.$event"
done
check loader 'area adapter .*"inventory\.read"' "area adapter inventory.read"
check ping "held read slot=[0-8] " "held read (alert)"
check ping "alert played slot=[0-8] item=minecraft:iron_pickaxe remaining=5" "alert played for the given pickaxe (name and damage read)"
echo "attended: BlockBreakStart raise and guard refusal (P2-18): $(count_lines ping 'action seen kind=player\.block_break_start') action seen, $(count_lines ping 'guard refused') guard refused"
for log in loader manager ping; do
  bad=$(count_lines "$log" "event not available|area not available|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no event, area or panic line in the $log log"; else echo "FAIL $bad event not available, area not available or panic lines in the $log log"; failures+=("$log log lines"); fi
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-durability-ping: pass"
else
  echo "ingame-durability-ping: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
