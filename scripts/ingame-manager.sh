# The body of the ingame-manager mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-manager`, which sets usage_instance, usage_modmanager, usage_world and usage_manual_world and runs
# it from the repo root. Builds modmanager and its probe mod in the modmanager checkout, installs only those two into a
# standalone test instance (the instance's other mods are parked for the run and put back after) and checks the
# manager and probe lines in the running game.
set -e
. scripts/ingame-instance.sh
manual=""
[ "${usage_manual_world:-false}" = "true" ] && manual="yes"
world="${usage_world:-}"
if [ -n "$manual" ] && [ -n "$world" ]; then echo "--world and --manual-world exclude each other"; exit 1; fi
case "$world" in
  */* | *\\* | .* | *:*) echo "--world must be one world folder name, got '$world'"; exit 1 ;;
esac
resolve_test_instance "${usage_instance:-}"
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

# Every other mod of the instance is parked for the run; the EXIT trap moves the two test mods to the parked folder
# and puts the parked mods back, and sets the pause option back. It is set before the option is changed, so a failed
# build cannot leave the option off
launched=""
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @amml/managerprobe
  pause_on_focus_lost_restore
  exit "$rc"
}
trap on_exit EXIT
# the posted focus loss must leave the game in play for the panel steps
pause_on_focus_lost_off

cargo build -p amml-loader --release
cp target/release/amml.dll "$instance/amml.dll"
rm -rf "$instance/amml/recipes" "$instance/amml/data"
mkdir -p "$instance/amml/data" "$instance/amml/cache"
cp -r recipes "$instance/amml/recipes"
cp data/*.hcl "$instance/amml/data/"
mise --cd "$modmanager" run package
mise --cd "$modmanager" run package-probe
dist="$modmanager/dist"
for mod in modmanager managerprobe; do
  [ -d "$dist/@amml/$mod" ] || { echo "the modmanager build wrote no $dist/@amml/$mod"; exit 1; }
done

park_mods
mkdir -p "$mods_root/@amml"
cp -r "$dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$dist/@amml/managerprobe" "$mods_root/@amml/managerprobe"
echo "installed @amml/modmanager and @amml/managerprobe from $dist"

compat_rc=0
mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
if [ "$compat_rc" -ne 0 ]; then echo "compat failed (exit $compat_rc); the game is not started"; exit 1; fi

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
probe_log="$logs/@amml/managerprobe.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
probe_start=$(size_of "$probe_log")
out_dir="target/ingame-manager/$(basename "$instance")"
mkdir -p "$out_dir"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or probe
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    probe) since "$probe_log" "$probe_start" ;;
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

before=" $(echo $(game_pids)) "
game_pid=""
# the variable is set for the game process only
if [ -z "$manual" ]; then
  (cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
else
  (cd "$instance" && AMML_TEST_BACKGROUND=1 ./Minecraft.Windows.exe > /dev/null 2>&1) &
fi
launched="yes"
started=$(date +%s)
while [ -z "$game_pid" ]; do
  find_game_pid "$instance"
  [ -n "$game_pid" ] && break
  if [ $(( $(date +%s) - started )) -ge 30 ]; then echo "the game did not start within 30 s"; game_gone="yes"; break; fi
  sleep 2
done
# The key and mouse feeds of some builds are not ready until a real key or mouse message has shown their thread. F24
# and mouse moves go to the window as soon as the loader has subclassed it, so the feeds can be ready before the world
# join; F24 is posted again while the loader has not decided the key feed thread
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
wait_line manager "overlay bound" 180 || true
wait_line probe "probe linked" 180 || true

# F8 for the probe's key bind, posted again while no `probe keybind fired` came
if alive; then
  for try in 1 2 3; do
    ps_run post-key.ps1 -ProcessId "$game_pid" -Key 119 || echo "posting F8 failed (try $try)"
    wait_line probe "probe keybind fired" 5 && break
  done
fi

# The probe's key hold starts at the world join; its line is the sign of play
world_wait=120
[ -n "$manual" ] && { world_wait=300; echo "enter a world in the game window by hand (waiting up to $world_wait s)"; }
joined_at=""
if wait_line probe "probe key hold done" "$world_wait"; then
  joined_at=$(date +%s)
fi

# Up to 480 s after the world join for `probe ok`; once the mouse step is done (the FULL lease starts), mouse moves
# for the look block check
if [ -n "$joined_at" ] && alive; then
  echo "--- probe"
  moved=""
  while alive; do
    [ "$(count_lines probe "probe ok")" -gt 0 ] && break
    if [ $(( $(date +%s) - joined_at )) -ge 480 ]; then echo "no 'probe ok' within 480 s of the world join"; break; fi
    if [ -z "$moved" ] && [ "$(count_lines probe "probe click seen|probe mouse waived")" -gt 0 ]; then
      ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed"
      moved="yes"
    fi
    sleep 2
  done
fi

# The focus loss releases the W the probe holds after `probe ok`; the probe holds it again after the release. The
# options of data_dir have pause_menu_on_focus_lost:0 for this run, so the game stays in play for the panel steps
if [ -n "$joined_at" ] && alive; then
  echo "--- focus loss"
  sleep 2
  ps_run post-focus-loss.ps1 -ProcessId "$game_pid" || echo "posting the focus loss failed"
  wait_line probe "probe injected released reason=FocusLost" 15 || true
  sleep 2
fi

# The panel steps: open with Ctrl+Shift+F10, screenshot, three characters, the cursor and mouse lines, then a click on
# the panel's close button
close_rect=""
if [ -n "$joined_at" ] && alive; then
  echo "--- panel steps"
  ps_run post-chord.ps1 -ProcessId "$game_pid" -Keys "17,16,121" || echo "posting the chord failed"
  if wait_line manager "panel opened by=keys" 15; then
    wait_line manager "panel layout" 10 || true
    close_rect=$(run_lines manager | sed -n 's/.*panel layout close=\([-0-9.]*\),\([-0-9.]*\),\([-0-9.]*\),\([-0-9.]*\) .*/\1 \2 \3 \4/p' | head -n 1)
    sleep 1
    shot=$(ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/panel")" -ShotOnly | tr -d '\r') || echo "capturing the panel failed"
    echo "$shot"
    client=$(printf '%s\n' "$shot" | sed -n 's/^screenshot: .*\b\([0-9][0-9]*\)x\([0-9][0-9]*\) .*/\1 \2/p' | head -n 1)
    for ch in a b c; do
      ps_run post-char.ps1 -ProcessId "$game_pid" -Char "$ch" || echo "posting the character failed"
    done
    wait_line manager "panel char received" 10 2 || true
    # posted mouse messages cause no WM_SETCURSOR, so the cursor message is posted too
    ps_run post-mouse-move.ps1 -ProcessId "$game_pid" || echo "posting mouse moves failed"
    ps_run post-set-cursor.ps1 -ProcessId "$game_pid" || echo "posting WM_SETCURSOR failed"
    wait_line manager "cursor shape" 10 || true
    wait_line manager "mouse captured value=false" 10 || true
    case "$client" in 0\ * | *\ 0) client="" ;; esac
    if [ -n "$close_rect" ] && [ -n "$client" ]; then
      fractions=$(echo "$close_rect $client" | awk '{ printf "%.5f %.5f", ($1 + $3 / 2) / $5, ($2 + $4 / 2) / $6 }') || true
      set -- $fractions
      ps_run post-click.ps1 -ProcessId "$game_pid" -RX "$1" -RY "$2" || echo "posting the close click failed"
    else
      echo "no close button rect (layout '$close_rect') or client size ('$client'); cannot click close"
    fi
    wait_line manager "panel action action=Close" 10 || true
    wait_line manager "mouse captured value=true" 10 || true
  fi
fi

# The window steps: two resizes, a minimise and a restore, then the original size
if [ -n "$joined_at" ] && alive; then
  echo "--- window steps"
  n=$(count_lines manager "resized ok=true")
  out=$(ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width 1280 -Height 760) || echo "resizing failed"
  echo "$out"
  outer=$(printf '%s\n' "$out" | sed -n 's/.*client before=[^ ]* outer=\([0-9]*\)x\([0-9]*\) after=.*/\1 \2/p' | head -n 1)
  wait_line manager "resized ok=true" 15 "$n" || true
  n=$(count_lines manager "resized ok=true")
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width 1600 -Height 940 || echo "resizing failed"
  wait_line manager "resized ok=true" 15 "$n" || true
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Minimise || echo "minimising failed"
  sleep 3
  ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Restore || echo "restoring failed"
  sleep 3
  if [ -n "$outer" ]; then
    n=$(count_lines manager "resized ok=true")
    set -- $outer
    ps_run resize-game-window.ps1 -ProcessId "$game_pid" -Width "$1" -Height "$2" || echo "resizing back failed"
    wait_line manager "resized ok=true" 15 "$n" || true
  fi
fi

# The close disables the mods: the probe sees the manager's provider go
echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @amml/managerprobe.log (this run)"
run_lines probe

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
for line in "probe linked" "probe layer drawn" "probe keybind fired" "probe settings changed" "probe hud lease granted" \
  "probe key hold done" "probe move seen" "probe turn seen"; do
  check probe "$line" "$line"
done
if [ "$(count_lines probe "probe mouse waived")" -gt 0 ]; then
  echo "ok   probe mouse waived (counts for wheel and click)"
else
  check probe "probe wheel seen" "probe wheel seen"
  check probe "probe click seen" "probe click seen"
fi
check probe 'probe full lease gamepad_skipped=' "probe full lease"
check probe "probe full lease released" "probe full lease released"
check probe 'probe screenshot written layers=GameOnly' "probe screenshot written layers=GameOnly"
check probe 'probe screenshot written layers=WithOverlays' "probe screenshot written layers=WithOverlays"
check probe "probe ok" "probe ok"
check probe 'probe injected released reason=FocusLost' "probe injected released reason=FocusLost"
check manager 'panel opened by=keys' "panel opened by=keys"
check manager "panel layout" "panel layout"
check manager "panel char received" "panel char received (three characters)" 3
check manager "cursor shape" "cursor shape"
check manager 'mouse captured value=false' "mouse captured value=false"
check manager 'panel action action=Close' "panel action action=Close"
check manager 'mouse captured value=true' "mouse captured value=true"
check manager 'resized ok=true' "two resized ok=true" 2
check probe "probe provider gone" "probe provider gone"
check probe 'probe call after gone result=Gone' "probe call after gone result=Gone"
for log in loader manager probe; do
  bad=$(count_lines "$log" "event not available|area not available|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no event, area or panic line in the $log log"; else echo "FAIL $bad event not available, area not available or panic lines in the $log log"; failures+=("$log log lines"); fi
done
# the adapter areas the two mods use
for area in client.read client.top_screen client.hud client.mouse input.key_inject input.mouse_inject input.turn input.move; do
  check loader "area adapter \{\"area\":\"$area\"" "area adapter $area"
done
if [ "$(count_lines loader "key feed thread|key injection not available")" -gt 0 ]; then
  echo "ok   $(run_lines loader | grep -m 1 -o -E "key feed thread[^{]*|key injection not available.*")"
else
  echo "FAIL no key feed thread or key injection not available line"
  failures+=("key feed thread")
fi
echo "key hold: $(run_lines probe | grep -m 1 -o -E "probe key hold done [^{]*")"
echo "world leave and rejoin (checked by hand): $(count_lines probe "probe world left") world left, $(count_lines probe "probe world joined again") joined again"

# The probe square's centre must be magenta in the WithOverlays screenshot and not in the GameOnly one
square=$(run_lines probe | sed -n 's/.*probe layer drawn .*square=\([0-9]*\),\([0-9]*\),\([0-9]*\).*/\1 \2 \3/p' | head -n 1)
magenta_game_only=""
magenta_with_overlays=""
if [ -n "$square" ]; then
  set -- $square
  cx=$(( $1 + $3 / 2 ))
  cy=$(( $2 + $3 / 2 ))
  for layers in GameOnly WithOverlays; do
    path=$(run_lines probe | sed -n "s/.*probe screenshot written layers=$layers ok=[a-z]* path=\(.*\) {\"module.*/\1/p" | head -n 1)
    if [ -z "$path" ] || [ ! -f "$(cygpath -u "$path")" ]; then echo "screenshot pixel layers=$layers: no file"; continue; fi
    rgb=$(ps_run png-pixel.ps1 -Path "$path" -X "$cx" -Y "$cy" | tr -d '\r')
    magenta="false"
    case "$rgb" in 25[0-5]\ [0-5]\ 25[0-5]) magenta="true" ;; esac
    echo "screenshot pixel layers=$layers x=$cx y=$cy rgb=$rgb magenta=$magenta file=$path"
    [ "$layers" = "GameOnly" ] && magenta_game_only="$magenta"
    [ "$layers" = "WithOverlays" ] && magenta_with_overlays="$magenta"
  done
fi
if [ "$magenta_with_overlays" = "true" ] && [ "$magenta_game_only" = "false" ]; then
  echo "ok   screenshot check (WithOverlays magenta, GameOnly not)"
else
  echo "FAIL screenshot check (WithOverlays magenta=${magenta_with_overlays:-none}, GameOnly magenta=${magenta_game_only:-none})"
  failures+=("screenshot check")
fi
echo "panel screenshot: $out_dir/panel.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-manager: pass"
else
  echo "ingame-manager: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
