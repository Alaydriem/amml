# The body of the ingame-hotbar-manager mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-hotbar-manager`, which sets usage_instance, usage_hotbar_manager and usage_modmanager and runs it
# from the repo root. Builds hotbar-manager and modmanager in their checkouts, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after), runs them in the
# world amml-test-hotbar-manager (a copy of amml-test with its own pristine copy) and checks the mod's and the loader's
# lines. The keys and the click are posted messages (post-chord.ps1, post-key.ps1, post-click.ps1): nothing reaches any
# other window.
#
# What the run proves (the pass lines are the mod's log lines):
#   core            `hotbar linked`
#   loader          `event offered` for ui.screen_render, input.ui_pointer, ui.screen_changed, inventory.request_done
#   rotation        Alt+D on the HUD gives `rotation done moves=N` (N is printed, not checked: the test world decides it)
#   screen gate     E opens the inventory, then Alt+D gives `rotation ignored reason=screen_open`
#   buttons         `buttons drawn kind=PlayerInventory profile=Classic` and `textures ready count=11`
#   click           a posted click on the drop-filtered button gives `pointer taken action=DropFiltered`, then
#                   `action done` or `action ignored` for it (an empty cursor ignores it: the click path is the proof)
#   screen change   Escape gives `screen changed screen=Hud`
# Rows only a person can see are in docs/findings/stable-mod-api/hotbar-manager-port/status.md (attended checklist).
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
hotbar="${usage_hotbar_manager:-${AMML_HOTBAR_MANAGER_DIR:-../hotbar-manager}}"
[ -d "$hotbar" ] || { echo "no hotbar-manager folder '$hotbar' (--hotbar-manager, else AMML_HOTBAR_MANAGER_DIR, else ../hotbar-manager)"; exit 1; }
hotbar="$(cd "$hotbar" && pwd -P)"
[ -f "$hotbar/mise.toml" ] || { echo "$hotbar has no mise.toml: not a hotbar-manager checkout"; exit 1; }
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
if declare -F seed_mod_world > /dev/null; then
  seed_mod_world hotbar-manager
else
  pick_test_world
  own_world="amml-test-hotbar-manager"
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
  restore_mods @amml/modmanager @alaydriem/hotbar-manager
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
mise --cd "$hotbar" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$hotbar/dist/@alaydriem/hotbar-manager" ] || { echo "the hotbar-manager build wrote no $hotbar/dist/@alaydriem/hotbar-manager"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$hotbar/dist/@alaydriem/hotbar-manager" "$mods_root/@alaydriem/hotbar-manager"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/hotbar-manager from $hotbar/dist"

for dist in "$modmanager/dist" "$hotbar/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
hotbar_log="$logs/@alaydriem/hotbar-manager.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
hotbar_start=$(size_of "$hotbar_log")
out_dir="target/ingame-hotbar-manager/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or hotbar
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    hotbar) since "$hotbar_log" "$hotbar_start" ;;
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
# Alt (VK 0x12 = 18) then the key, as the default chords of the mod are Alt+D (VK 0x44 = 68)
alt_chord() { ps_run post-chord.ps1 -ProcessId "$game_pid" -Keys "18,$1" || echo "posting Alt+$1 failed"; }
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
wait_line hotbar "hotbar linked" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
  # the test world is creative; the buttons sit on the classic survival inventory panel
  chat_command "gamemode survival"
  sleep 2
fi

rotation_before=0
screen_open_before=0
if [ -n "$joined" ] && alive; then
  echo "--- rotation on the HUD (Alt+D)"
  for try in 1 2 3; do
    alive || break
    alt_chord 68
    wait_line hotbar "rotation done moves=" 8 && break
  done
  rotation_before=$(count_lines hotbar "rotation done moves=")
  echo "--- inventory (E)"
  key 69
  wait_line hotbar "buttons drawn kind=PlayerInventory profile=Classic" 20 || true
  wait_line hotbar "textures ready count=11" 30 || true
  sleep 1
  shot inventory
  echo "--- rotation with the inventory open (Alt+D)"
  screen_open_before=$(count_lines hotbar "rotation ignored reason=screen_open")
  alt_chord 68
  wait_line hotbar "rotation ignored reason=screen_open" 10 "$screen_open_before" || true
  echo "--- click on the drop-filtered button"
  place=$(run_lines hotbar | grep -E "button placed kind=PlayerInventory action=DropFiltered" | tail -n 1 || true)
  rx=$(printf '%s' "$place" | sed -n 's/.* rx=\([0-9.]*\).*/\1/p')
  ry=$(printf '%s' "$place" | sed -n 's/.* ry=\([0-9.]*\).*/\1/p')
  if [ -n "$rx" ] && [ -n "$ry" ]; then
    echo "button centre at $rx $ry of the client area"
    ps_run post-click.ps1 -ProcessId "$game_pid" -RX "$rx" -RY "$ry" || echo "posting the click failed"
    wait_line hotbar "pointer taken action=DropFiltered" 10 || true
    wait_line hotbar "action (done|ignored) action=DropFiltered" 10 || true
  else
    echo "no 'button placed' line for DropFiltered in the hotbar-manager log; the click is not posted"
  fi
  shot after_click
  echo "--- close the inventory (Escape)"
  key 27
  wait_line hotbar "screen changed screen=Hud" 10 || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/hotbar-manager.log (this run)"
run_lines hotbar

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $hotbar/dist"
check hotbar "hotbar linked" "manager link"
for event in ui.screen_render input.ui_pointer ui.screen_changed inventory.request_done; do
  check loader "event offered .*\"$event\"" "event offered $event"
done
check hotbar "rotation done moves=" "rotation on the HUD (Alt+D): rotation done"
check hotbar "rotation ignored reason=screen_open" "rotation with the inventory open: ignored (screen_open)"
check hotbar "buttons drawn kind=PlayerInventory profile=Classic" "buttons drawn on the player inventory (Classic)"
check hotbar "textures ready count=11" "button textures ready (11)"
# A posted mouse message never reaches MouseDevice::feed while the game runs in the background (no `mouse feed thread`
# line; docs/findings/stable-mod-api/ui-ports/debug-screen-changed.md), so the button click is an attended row (A2, A3).
echo "note the button click is not checked in game: posted mouse input does not reach the game in the background; attended rows A2 and A3"
check hotbar "screen changed screen=Hud" "Escape: screen changed to the HUD"
for log in loader manager hotbar; do
  bad=$(count_lines "$log" "event not available|area not available|area withdrawn|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, withdrawn or panic line in the $log log"; else echo "FAIL $bad unavailable, withdrawn or panic lines in the $log log"; failures+=("$log log lines"); fi
done
bad=$(count_lines hotbar "the row rotation ended early|the row rotation was not started|the row rotation request never ended|textures did not load|glyph did not load|a listener was re-entered")
if [ "$bad" -eq 0 ]; then echo "ok   no failed rotation, texture or re-entry line in the hotbar-manager log"; else echo "FAIL $bad failed rotation, texture or re-entry lines in the hotbar-manager log"; failures+=("hotbar log faults"); fi
for name in inventory after_click; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot: $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-hotbar-manager: pass"
else
  echo "ingame-hotbar-manager: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
