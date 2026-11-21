# The body of the ingame-heads mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-heads`, which sets usage_instance, usage_heads and usage_modmanager and runs it from the repo root.
# Builds heads and modmanager in their checkouts, installs only those two into a standalone test instance (the
# instance's other mods are parked for the run and put back after), runs them in the world amml-test-heads (a copy of
# amml-test with its own pristine copy) and checks the mod's and the loader's lines. The local player is the one player
# of the list: the run proves the player list, the face cut, the texture upload, the inline icon and the chat edit.
# The locator bar face of a second player and the other rows of the status file are attended checks.
set -e
. scripts/ingame-instance.sh
world="amml-test"
resolve_test_instance "${usage_instance:-}"
heads="${usage_heads:-${AMML_HEADS_DIR:-../heads}}"
[ -d "$heads" ] || { echo "no heads folder '$heads' (--heads, else AMML_HEADS_DIR, else ../heads)"; exit 1; }
heads="$(cd "$heads" && pwd -P)"
[ -f "$heads/mise.toml" ] || { echo "$heads has no mise.toml: not a heads checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
# the constructor shape of the chat line the loader must pick on this build
case "$(lower "$(basename "$instance")")" in
  1.26.70.*) chat_shape="ctor_flag_word" ;;
  *) chat_shape="ctor" ;;
esac
. scripts/game-pid.sh
. scripts/ingame-common.sh
before=""
game_pid=""
require_no_game "$instance"
# the shared amml-test world is only read here (restored from its pristine copy); the run plays in its own copy
if declare -F seed_mod_world > /dev/null; then
  seed_mod_world heads
else
  pick_test_world
  own_world="amml-test-heads"
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
  restore_mods @amml/modmanager @alaydriem/heads
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
mise --cd "$heads" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$heads/dist/@alaydriem/heads" ] || { echo "the heads build wrote no $heads/dist/@alaydriem/heads"; exit 1; }

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$heads/dist/@alaydriem/heads" "$mods_root/@alaydriem/heads"
# the settings of the run: the defaults, written out
mkdir -p "$mods_root/@alaydriem/heads/config"
printf 'heads {\n  enabled = true\n  scale = 1.0\n  chat = true\n}\n' > "$mods_root/@alaydriem/heads/config/config.hcl"
echo "installed @amml/modmanager from $modmanager/dist and @alaydriem/heads from $heads/dist"

for dist in "$modmanager/dist" "$heads/dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
heads_log="$logs/@alaydriem/heads.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
heads_start=$(size_of "$heads_log")
out_dir="target/ingame-heads/$(basename "$instance")"
mkdir -p "$out_dir"
rm -f "$out_dir"/*.png
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or heads
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    heads) since "$heads_log" "$heads_start" ;;
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
# types one chat line: T opens the chat, then the text and Enter
chat_line() {
  local text="$1" i
  key 84
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
wait_line heads "heads enabled" 60 || true
wait_line heads "heads settings applied enabled=true scale=1 chat=true" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

if [ -n "$joined" ] && alive; then
  echo "--- world join: player list, face, texture, icon"
  wait_line heads "heads player list action=add entries=[1-9]" 30 || true
  wait_line heads "heads face ready source=(skin|persona)" 30 || true
  wait_line heads "heads face uploaded" 30 || true
  wait_line heads "heads chat icon registered" 30 || true
  echo "--- chat line"
  for try in 1 2 3; do
    alive || break
    chat_line "heads proof $try"
    wait_line heads "heads chat line edited" 8 && break
  done
  sleep 2
  shot chat
fi

echo "--- close"
end_game
wait_line heads "heads world left faces=0" 20 || true
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/heads.log (this run)"
run_lines heads

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $heads/dist"
check heads "heads enabled features_off=( |\{|$)" "enable with no feature off"
check heads "heads settings applied enabled=true scale=1 chat=true" "settings applied"
check heads "heads player list action=add entries=[1-9]" "player list add, at least one entry"
check heads "heads face ready source=(skin|persona)" "face cut from the skin of the local player"
check heads "heads face uploaded" "face texture uploaded"
check heads "heads chat icon registered" "chat icon registered"
check heads "heads chat line edited" "chat line edited with the face glyph"
check heads "heads world left faces=0" "world left, faces cleared"
check loader 'event offered .*"network\.player_list"' "event offered network.player_list"
check loader 'event offered .*"network\.player_skin"' "event offered network.player_skin"
check loader 'event offered .*"ui\.locator_bar_icon"' "event offered ui.locator_bar_icon"
check loader 'event offered .*"ui\.chat_message"' "event offered ui.chat_message"
check loader 'area adapter .*"ui\.textures"|area offered .*"ui\.textures"' "area ui.textures offered"
check loader "player list thread" "player list thread"
check loader "chat message thread" "chat message thread"
check loader "textures upload done" "textures upload done"
check loader "inline icon page ready" "inline icon page ready"
check loader "event offered .*\"ui\.chat_message\".*\"shape\":\"$chat_shape\"" "chat line shape $chat_shape"
check loader 'textures freed .*world_left' "textures freed at the world leave"
echo "note the locator bar face of a second player, scale, skin change, persona, resource pack reload and Alt+Tab are attended checks (status file)"
for log in loader manager heads; do
  bad=$(count_lines "$log" "event not available|area not available|area withdrawn|upload refused|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, withdrawn, refused or panic line in the $log log"; else echo "FAIL $bad unavailable, withdrawn, refused or panic lines in the $log log"; failures+=("$log log lines"); fi
done
[ -f "$out_dir/chat.png" ] && echo "screenshot: $out_dir/chat.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-heads: pass"
else
  echo "ingame-heads: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
