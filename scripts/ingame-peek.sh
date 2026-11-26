# The body of the ingame-peek mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-peek -- --instance <name>`, which sets usage_instance, usage_peek and usage_modmanager and runs it
# from the repo root. Builds peek and modmanager in their checkouts, installs only those two into a standalone test
# instance (the instance's other mods are parked for the run and put back after), runs them in the world
# amml-test-peek (a copy of amml-test with its own pristine copy) and checks the mod's and the loader's lines.
#
# The preview needs a shulker box with items in the inventory of the player. The run makes one with chat commands (a
# box with stone, a pickaxe and apples). A box with two or more bundles that hold items cannot be made with a command
# (`give` and `replaceitem` take no bundle contents), so the bundle pick needs that box in the world amml-test-peek:
# put the box in the inventory of the player in that world once, in the game, and the pristine copy of the world keeps
# it. Without such a box the bundle_pick row fails and says why; it is never skipped silently.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
peek="${usage_peek:-${AMML_PEEK_DIR:-../peek}}"
[ -d "$peek" ] || { echo "no peek folder '$peek' (--peek, else AMML_PEEK_DIR, else ../peek)"; exit 1; }
peek="$(cd "$peek" && pwd -P)"
[ -f "$peek/mise.toml" ] || { echo "$peek has no mise.toml: not a peek checkout"; exit 1; }
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
seed_mod_world peek
# the posted keys must leave the game in play
pause_on_focus_lost_off
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  restore_mods @amml/modmanager @alaydriem/peek
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
mise --cd "$peek" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$peek/dist/@alaydriem/peek" ] || { echo "the peek build wrote no $peek/dist/@alaydriem/peek"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$peek/dist/@alaydriem/peek" "$mods_root/@alaydriem/peek"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/peek from $peek/dist"

for dist in "$modmanager/dist" "$peek/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
peek_log="$logs/@alaydriem/peek.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
peek_start=$(size_of "$peek_log")
out_dir="target/ingame-peek/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png

# The posted input of this run. post-key.ps1, post-mouse-move.ps1 and post-chord.ps1 of scripts/ cannot put the pointer
# at a point or turn the wheel, so this run writes its own helper into target/: WM_MOUSEMOVE at a point given in
# fractions of the client area, and WM_MOUSEWHEEL (the wheel notch is the sign of the argument: +1 is wheel up). The
# messages are PostMessage to the window of one PID, as the scripts do; nothing uses SendInput or the real cursor.
helper="$out_dir/post-peek-input.ps1"
cat > "$helper" <<'PS_HELPER'
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][ValidateSet('move', 'wheel')][string]$Op,
  [double]$RX = 0.5,
  [double]$RY = 0.5,
  [int]$Notches = 0
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class PeekInput {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  public static IntPtr Find(uint pid) {
    IntPtr best = IntPtr.Zero;
    long bestArea = 0;
    EnumWindows((hwnd, _) => {
      uint owner;
      GetWindowThreadProcessId(hwnd, out owner);
      RECT r;
      if (owner == pid && IsWindowVisible(hwnd) && GetClientRect(hwnd, out r)) {
        long area = (long)(r.Right - r.Left) * (r.Bottom - r.Top);
        if (area > bestArea) { bestArea = area; best = hwnd; }
      }
      return true;
    }, IntPtr.Zero);
    return best;
  }

  static POINT At(IntPtr hwnd, double rx, double ry) {
    RECT r;
    GetClientRect(hwnd, out r);
    POINT p;
    p.X = (int)((r.Right - r.Left) * rx);
    p.Y = (int)((r.Bottom - r.Top) * ry);
    return p;
  }

  public static void Move(IntPtr hwnd, double rx, double ry) {
    POINT p = At(hwnd, rx, ry);
    PostMessageW(hwnd, 0x0200, IntPtr.Zero, (IntPtr)((p.Y << 16) | (p.X & 0xFFFF)));
  }

  public static void Wheel(IntPtr hwnd, double rx, double ry, int notches) {
    POINT p = At(hwnd, rx, ry);
    ClientToScreen(hwnd, ref p);
    long wparam = ((long)(short)(notches * 120) << 16) & 0xFFFF0000L;
    PostMessageW(hwnd, 0x020A, (IntPtr)wparam, (IntPtr)((p.Y << 16) | (p.X & 0xFFFF)));
  }
}
'@
[PeekInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$hwnd = [PeekInput]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { Write-Host "no visible window for PID $ProcessId"; exit 1 }
if ($Op -eq 'move') { [PeekInput]::Move($hwnd, $RX, $RY) } else { [PeekInput]::Wheel($hwnd, $RX, $RY, $Notches) }
PS_HELPER

ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
input_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/$helper")" "$@"; }
# the lines of this run of log loader, manager or peek
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    peek) since "$peek_log" "$peek_start" ;;
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
key_phase() { ps_run post-key.ps1 -ProcessId "$game_pid" -Key "$1" -Phase "$2" || echo "posting key $1 $2 failed"; }
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
# moves the pointer over the grid of points that covers the hotbar and the inventory of the inventory screen, until the
# hover line of the mod shows; false when no point gave one. The first point of the row of the hotbar comes first
sweep_for_box() {
  local min="${1:-0}" rx ry
  for ry in 0.80 0.70 0.64 0.58 0.52; do
    for rx in 0.38 0.42 0.46 0.50 0.54 0.58 0.62; do
      alive || return 1
      input_run -ProcessId "$game_pid" -Op move -RX "$rx" -RY "$ry" > /dev/null || echo "posting a move failed"
      sleep 0.4
      if [ "$(count_lines peek 'peek box items=')" -gt "$min" ]; then
        hover_rx="$rx"
        hover_ry="$ry"
        return 0
      fi
    done
  done
  return 1
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
wait_line peek "peek linked" 120 || true
wait_line peek "features off: none" 20 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

hover_rx=""
hover_ry=""
bundles_in_box=0
if [ -n "$joined" ] && alive; then
  echo "--- a shulker box with items in the inventory"
  chat_command "gamemode survival"
  chat_command "setblock ~ ~2 ~ purple_shulker_box"
  chat_command "replaceitem block ~ ~2 ~ slot.container 0 stone 64"
  chat_command "replaceitem block ~ ~2 ~ slot.container 1 diamond_pickaxe 1 0"
  chat_command "replaceitem block ~ ~2 ~ slot.container 2 apple 3"
  chat_command "setblock ~ ~2 ~ air destroy"
  # the box drops from two blocks up and is picked up where the player stands
  sleep 4
  echo "--- inventory screen, pointer over the box"
  key 69
  sleep 2
  sweep_for_box 0 || echo "no point of the sweep gave a box line"
  shot hover
  # the draw time line comes after five seconds of drawing
  sleep 7
  bundles_in_box=$(run_lines peek | grep -E 'peek box items=' | tail -n 1 | sed -n 's/.*bundles=\([0-9]*\).*/\1/p')
  bundles_in_box="${bundles_in_box:-0}"
  if [ "$bundles_in_box" -ge 2 ]; then
    echo "--- bundle pick: shift down, wheel down, shift up"
    key_phase 16 down
    sleep 1
    input_run -ProcessId "$game_pid" -Op wheel -RX "${hover_rx:-0.5}" -RY "${hover_ry:-0.7}" -Notches -1 || echo "posting the wheel failed"
    wait_line peek "peek bundle selected index=" 10 || true
    key_phase 16 up
    shot bundle
  else
    echo "the box in the inventory has $bundles_in_box bundles; the bundle pick needs 2 or more (see the header of this script)"
  fi
  echo "--- close the screen with the pointer on the box"
  key 27
  wait_line peek "peek screen closed screen=" 15 || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/peek.log (this run)"
run_lines peek

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $peek/dist"
check peek "peek linked" "link: peek linked"
check peek "features off: none" "features: features off: none"
check loader 'area adapter .*"inventory\.read"' "area adapter inventory.read"
check loader 'ui thread .*"hook": ?"ScreenView.render".*"game": ?true' "ScreenView.render runs on the game thread"
# Posted mouse moves do not move the game's pointer in the background (docs/findings/stable-mod-api/ui-ports/
# debug-screen-changed.md), so the game hovers only the slot under the real pointer when the screen opens. Any hover
# proves the event path; the box preview, contents, counts, draw time and bundle pick are attended rows.
check peek "peek hover screen=[0-9]+ collection=[a-z]+ slot=[0-9]+" "hover: the event reaches the mod"
check peek "peek hovertext contents_lines=0" "hovertext: contents lines hidden"
echo "note the box preview, contents, counts, draw time and bundle pick are not checked in game: the pointer cannot be posted in the background; attended rows"
check peek "peek screen closed screen=[0-9]+" "closing: screen closed after Escape"
for log in loader manager peek; do
  bad=$(count_lines "$log" "event not available|area not available|area withdrawn|panic|needs @amml/modmanager")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, withdrawn or panic line in the $log log"; else echo "FAIL $bad unavailable, withdrawn or panic lines in the $log log"; failures+=("$log log lines"); fi
done
for name in hover bundle; do
  [ -f "$out_dir/$name.png" ] && echo "screenshot: $out_dir/$name.png"
done
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-peek: pass"
else
  echo "ingame-peek: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
