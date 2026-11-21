# The body of the ingame-headless-input mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-headless-input`, which sets usage_instance, usage_headless and usage_modmanager and runs it from the
# repo root. Builds modmanager and an ingame-proof build of headless-input (pairing allowed without the switch),
# installs only those two into a standalone test instance (the instance's other mods are parked for the run and put
# back after), starts the per-mod test world amml-test-headless-input from its pristine copy, drives the mod through
# its MCP hub and checks the lines that prove each feature.
set -e
. scripts/ingame-instance.sh
resolve_test_instance "${usage_instance:-}"
headless="${usage_headless:-${AMML_HEADLESS_DIR:-../headless-input}}"
[ -d "$headless" ] || { echo "no headless-input folder '$headless' (--headless, else AMML_HEADLESS_DIR, else ../headless-input)"; exit 1; }
headless="$(cd "$headless" && pwd -P)"
[ -f "$headless/mise.toml" ] || { echo "$headless has no mise.toml: not a headless-input checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
. scripts/ingame-bds.sh
before=""
game_pid=""
require_no_game "$instance"

# The per-mod test world: a copy of the instance's amml-test world (from its pristine copy when there is one), with
# its own pristine copy; each run starts from that pristine copy. The shared amml-test world is never changed.
world="amml-test-headless-input"
seeded=""
for root in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftWorlds; do
  [ -d "$root" ] || continue
  case "$root" in */Users/Shared/*) continue ;; esac
  pristine_root="$(dirname "$root")/amml-pristine-worlds"
  source_world="$pristine_root/amml-test"
  [ -d "$source_world" ] || source_world="$root/amml-test"
  [ -d "$source_world" ] || continue
  if [ ! -d "$pristine_root/$world" ]; then
    mkdir -p "$pristine_root"
    cp -r "$source_world" "$pristine_root/$world"
    printf '%s' "$world" > "$pristine_root/$world/levelname.txt"
    echo "made the pristine copy $pristine_root/$world from $source_world"
  fi
  rm -rf "${root:?}/${world:?}"
  cp -r "$pristine_root/$world" "$root/$world"
  echo "restored world $world from $pristine_root/$world"
  seeded="yes"
done
[ -n "$seeded" ] || { echo "no amml-test world under $data_dir to copy; seed the instance first"; exit 1; }
pause_on_focus_lost_off
# the multiplayer safety notices stop the scripted server join until a person accepts them
run_options=()
run_option_set do_not_show_multiplayer_online_safety_warning 0 1
run_option_set do_not_show_multiplayer_ip_safety_warning 0 1
launched=""
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  bds_stop
  restore_mods @amml/modmanager @alaydriem/headless-input
  [ -z "${mods_root:-}" ] || rmdir "$mods_root/@alaydriem" 2> /dev/null || true
  pause_on_focus_lost_restore
  run_options_restore
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
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
# the ingame-proof build, staged as its own dist folder
(cd "$headless" && cargo build --release -p headless-input-amml --features ingame-proof)
# the dedicated server the join answer row joins (downloaded once per instance, as DejaVu's fixture runs one)
bds_ensure
proof_dist="$PWD/target/ingame-headless-input/dist"
rm -rf "$proof_dist"
mkdir -p "$proof_dist/@alaydriem/headless-input"
cp "$headless/target/release/alaydriem_headless_input.dll" "$proof_dist/@alaydriem/headless-input/"
cp "$headless/src/amml/amml.hcl" "$headless/src/amml/icon.png" "$proof_dist/@alaydriem/headless-input/"

for dist in "$modmanager/dist" "$proof_dist"; do
  compat_rc=0
  mise run compat -- --exe "$(cygpath -w "$exe")" --mods "$(cygpath -w "$dist")" || compat_rc=$?
  if [ "$compat_rc" -ne 0 ]; then echo "compat failed on $dist (exit $compat_rc); the game is not started"; exit 1; fi
done

park_mods
mkdir -p "$mods_root/@amml" "$mods_root/@alaydriem"
cp -r "$modmanager/dist/@amml/modmanager" "$mods_root/@amml/modmanager"
cp -r "$proof_dist/@alaydriem/headless-input" "$mods_root/@alaydriem/headless-input"
# MCP on, on a port of its own per instance, so the two test instances never share a hub
case "$(basename "$instance")" in
  1.26.63.01-test) mcp_port=28361; bds_port=28210 ;;
  *) mcp_port=28362; bds_port=28211 ;;
esac
mkdir -p "$mods_root/@alaydriem/headless-input/config"
printf 'mcp {\n  enabled = true\n  port    = %s\n}\n' "$mcp_port" > "$mods_root/@alaydriem/headless-input/config/config.hcl"
echo "installed @amml/modmanager and the ingame-proof @alaydriem/headless-input (MCP port $mcp_port)"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
mod_log="$logs/@alaydriem/headless-input.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
mod_start=$(size_of "$mod_log")
out_dir="target/ingame-headless-input/$(basename "$instance")"
mkdir -p "$out_dir"
mod_mark="$out_dir/.mod-log-mark"
touch "$mod_mark"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
run_lines() {
  case "$1" in
    loader) since "$logs/loader.log" "$loader_start" ;;
    manager) since "$manager_log" "$manager_start" ;;
    mod) mod_run_lines ;;
    bds) [ -f "$out_dir/bds.log" ] && cat "$out_dir/bds.log" ;;
  esac
}
# the mod log of this run: the mod log rotates at about 1 MB, so the lines of this run can start in a rotated
# headless-input_<time>.log (from the start offset) and go on in later rotated files and the live file
mod_run_lines() {
  local first="yes" f
  while IFS= read -r f; do
    [ -n "$f" ] && [ "$f" -nt "$mod_mark" ] || continue
    if [ -n "$first" ]; then tail -c +$(( mod_start + 1 )) "$f"; first=""; else cat "$f"; fi
  done < <(ls "$(dirname "$mod_log")"/headless-input_*.log 2> /dev/null | sort)
  if [ -n "$first" ]; then since "$mod_log" "$mod_start"; elif [ -f "$mod_log" ]; then cat "$mod_log"; fi
}
count_lines() { run_lines "$1" | grep -c -E "$2" || true; }
# the loader line of the first real key down: the key-processor adapter (70.x) logs `first real key down`; the 26.6x
# adapter decides the key feed thread at that key and logs the bare `key feed thread is the game thread` (the 70.x line
# with `from: window` comes before any real key and does not count)
real_key='first real key down|key feed thread is the game thread[[:space:]]*$'
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
token=""
# one MCP tool call; prints the result text (or `error: ...`)
mcp() { ps_run mcp-call.ps1 -Port "$mcp_port" -Token "$token" -Tool "$1" -ArgsJson "$2" | tr -d '\r'; }
# a field of the status reply: `status_field yaw`
status_json=""
read_status() { status_json=$(mcp status '{}'); }
status_field() { printf '%s' "$status_json" | grep -o -E "\"$1\":(\"[^\"]*\"|[^,}]*)" | head -n 1 | cut -d ':' -f 2- | tr -d '"'; }
# waits up to $1 s for the run state idle; then the reason of the last run in last_reason
last_reason=""
wait_idle() {
  local waited=0
  while [ "$waited" -lt "$1" ]; do
    read_status
    if [ "$(status_field state)" = "idle" ]; then
      last_reason=$(printf '%s' "$status_json" | grep -o -E '"reason":"[^"]*"' | head -n 1 | cut -d '"' -f 4)
      return 0
    fi
    alive || return 1
    sleep 1
    waited=$(( waited + 1 ))
  done
  echo "the run did not end within $1 s"
  return 1
}
run_file() {
  local text="$1" json
  json=$(printf '%s' "$text" | sed 's/\\/\\\\/g; s/"/\\"/g' | awk 'BEGIN { ORS = "\\n" } { print }')
  mcp run_control_file "{\"text\":\"$json\"}"
}
results=()
note() { results+=("$1"); echo "$1"; }

before=" $(echo $(game_pids)) "
game_pid=""
(cd "$instance" && AMML_TEST_BACKGROUND=1 AMML_TEST_WORLD="$world" ./Minecraft.Windows.exe > /dev/null 2>&1) &
launched="yes"
started=$(date +%s)
while [ -z "$game_pid" ]; do
  find_game_pid "$instance"
  [ -n "$game_pid" ] && break
  if [ $(( $(date +%s) - started )) -ge 30 ]; then echo "the game did not start within 30 s"; game_gone="yes"; break; fi
  sleep 2
done
bds_configure "$bds_port"
if ! bds_install_trust; then
  echo "no saved server trust: the server join of this run waits for a person to accept the game's trust prompt"
fi

echo "--- start"
wait_line manager "manager enabled" 180 || true
wait_line mod "attached to the mod manager" 180 || true
wait_line mod "the game tick reached the local player in this world" 180 || true
# the hub binds the port after the settings reached the mod
for try in 1 2 3 4 5 6 7 8 9 10; do
  token=$(ps_run mcp-call.ps1 -Port "$mcp_port" -Pair | tr -d '\r')
  case "$token" in error:* | "") token=""; sleep 3 ;; *) break ;; esac
done
[ -n "$token" ] && echo "paired with the hub on port $mcp_port" || echo "pairing failed"
sleep 3

if [ -n "$token" ] && alive; then
  # HEADLESS_ONLY=join runs only the server join (the other rows then fail); for checking the join on its own
  if [ "${HEADLESS_ONLY:-}" != "join" ]; then
  echo "--- press before any real key"
  real_before=$(count_lines loader "$real_key")
  note "press: $(mcp press '{"key":"F1"}')"
  wait_line mod "key tap sent" 10 || true
  sleep 1
  mcp press '{"key":"F1"}' > /dev/null
  echo "real key downs before the press: $real_before"

  echo "--- hotbar"
  note "hotbar: $(mcp hotbar '{"slot":3}')"
  wait_line mod 'hotbar selected.*"slot":3|hotbar selected.*slot=3' 10 || true
  mcp release '{}' > /dev/null || true

  echo "--- square"
  note "square: $(run_file "$(cat "$headless/examples/square.hcl")")"
  wait_idle 60 || true
  square_reason="$last_reason"

  echo "--- walk_to"
  read_status
  pos=$(printf '%s' "$status_json" | grep -o -E '"position":\[[^]]*\]' | head -n 1 | sed 's/.*\[//; s/\]//')
  px=$(printf '%s' "$pos" | cut -d ',' -f 1)
  pz=$(printf '%s' "$pos" | cut -d ',' -f 3)
  tx=$(awk -v x="$px" 'BEGIN { printf "%.1f", x + 3 }')
  note "walk_to from $px,$pz to $tx: $(run_file "$(printf 'name = "walk-to"\n\nstep "walk_to" {\n  x      = %s\n  z      = %s\n  within = 1.0\n}\n' "$tx" "$pz")")"
  wait_idle 40 || true
  walk_reason="$last_reason"

  echo "--- turn and look_at"
  note "look: $(run_file "$(printf 'name = "look"\n\nstep "turn" {\n  yaw  = 90\n  over = "10t"\n}\n\nstep "look_at" {\n  yaw   = 0\n  pitch = 30\n  over  = "10t"\n}\n')")"
  wait_idle 30 || true
  look_reason="$last_reason"
  read_status
  look_yaw=$(status_field yaw)
  look_pitch=$(status_field pitch)
  echo "look read yaw=$look_yaw pitch=$look_pitch"

  echo "--- command"
  note "command: $(run_file "$(printf 'name = "say"\n\nstep "command" {\n  text = "/say headless"\n}\n\nstep "wait" {\n  for = "1s"\n}\n')")"
  wait_idle 20 || true
  wait_line mod "a command was sent to the server" 5 || true

  echo "--- screenshot"
  shot=$(mcp screenshot '{"layers":"game","max_width":640}')
  echo "screenshot: ${shot:0:160}"
  wait_line mod "screenshot written" 15 || true
  shot_file=$(ls -t "$mods_root/@alaydriem/headless-input/data/screenshots/"*.png 2> /dev/null | head -n 1 || true)
  [ -n "$shot_file" ] && cp "$shot_file" "$out_dir/screenshot.png"

  echo "--- dimension"
  note "dimension: $(run_file "$(printf 'name = "nether"\n\nstep "command" {\n  text = "/execute in nether run tp @s 0 100 0"\n}\n\nstep "walk_until_dimension" {\n  dimension = "nether"\n  timeout   = "40s"\n}\n')")"
  wait_idle 60 || true
  dimension_reason="$last_reason"

  echo "--- leave"
  note "leave: $(mcp leave_world '{"timeout":"60s"}')"
  wait_line mod "asking the game to leave the world" 30 || true
  wait_idle 90 || true
  sleep 5

  echo "--- join world"
  note "join_world: $(mcp join_world "{\"world\":\"$world\",\"timeout\":\"120s\"}")"
  wait_line mod "the game took the world join" 60 || true
  wait_idle 150 || true
  wait_line mod "the game tick reached the local player in this world" 120 1 || true
  sleep 5

  fi

  echo "--- servers tab and join answer"
  bds_start "$out_dir/bds.log" || true
  note "join_server: $(mcp join_server "{\"address\":\"127.0.0.1\",\"port\":$bds_port,\"timeout\":\"240s\"}")"
  wait_line mod "screen after the servers tab was asked" 120 || true
  # what the game shows once the join started (the first join of an instance asks a person to trust the server key)
  wait_line mod "the game took the join" 30 || true
  sleep 3
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/join-screen")" -ShotOnly > /dev/null || echo "the join screenshot failed"
  if [ -f "$bds_trust_dir/storage_object" ]; then
    wait_line mod "the game answered the join" 90 || true
  else
    # the input lease blocks the person's click: end the live session (the game's join goes on) and wait for the click
    mcp release '{}' > /dev/null || true
    echo "accept 'Trust and Join' in the game window now (waiting up to 240 s)"
    wait_line mod "the game answered the join" 240 || true
  fi
  # the join answer comes at login; the player is in the server's world only at its spawn line
  wait_line bds "Player Spawned" 60 || true
  sleep 3
  ps_run capture-game-window.ps1 -ProcessId "$game_pid" -Out "$(cygpath -w "$PWD/$out_dir/server-world")" -ShotOnly > /dev/null || echo "the server world screenshot failed"
  # the game writes its trust of the server key into storage_object once a person accepted; keep it for later runs
  bds_save_trust
  bds_stop
  wait_idle 120 || true

  echo "--- first real key"
  ps_run post-key.ps1 -ProcessId "$game_pid" -Key 135 || echo "posting F24 failed"
  wait_line loader "$real_key" 10 || true
fi

echo "--- close"
end_game
sleep 2
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/headless-input.log (this run)"
run_lines mod

echo "--- summary"
failures=()
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
check_value() {
  if [ "$2" = "yes" ]; then echo "ok   $1"; else echo "FAIL $1 ($3)"; failures+=("$1"); fi
}
echo "ok   compat on $modmanager/dist and $proof_dist"
check mod "attached to the mod manager" "link: attached to the mod manager"
check mod "the game tick reached the local player in this world" "tick: the game tick reached the local player" 2
check loader 'event offered .*"client\.game_tick"' "loader: client.game_tick offered"
check mod "key tap sent" "press: key tap sent"
first_key=$(run_lines loader | grep -n -m 1 -E "$real_key" | cut -d ':' -f 1 || true)
first_tap=$(run_lines mod | grep -c "key tap sent" || true)
if [ "${real_before:-1}" = "0" ]; then echo "ok   press came before any real key down (first real key down line at loader line ${first_key:-none})"; else echo "FAIL press before any real key (${real_before:-unknown} real key downs before)"; failures+=("press before a real key"); fi
check mod 'hotbar selected' "hotbar: hotbar selected slot=3"
check_value "square: the run ended done" "$([ "$square_reason" = "done" ] && echo yes)" "last reason ${square_reason:-none}"
check_value "position: walk_to ended done" "$([ "$walk_reason" = "done" ] && echo yes)" "last reason ${walk_reason:-none}"
look_ok=$(awk -v y="${look_yaw:-999}" -v p="${look_pitch:-999}" 'BEGIN { d = y; if (d < 0) d = -d; e = p - 30; if (e < 0) e = -e; print (d <= 1 && e <= 1) ? "yes" : "no" }')
check_value "rotation: look read yaw=0 pitch=30 within 1 degree" "$([ "$look_reason" = "done" ] && [ "$look_ok" = "yes" ] && echo yes)" "reason ${look_reason:-none}, yaw=${look_yaw:-none} pitch=${look_pitch:-none}"
check mod "a command was sent to the server" "command: a command was sent to the server"
check mod "screenshot written" "screenshot: screenshot written"
shot_bytes=$(size_of "$out_dir/screenshot.png")
check_value "screenshot: the PNG exists" "$([ "$shot_bytes" -gt 2000 ] && echo yes)" "$shot_bytes bytes"
check_value "dimension: walk_until_dimension nether ended done" "$([ "$dimension_reason" = "done" ] && echo yes)" "last reason ${dimension_reason:-none}"
check mod "asking the game to leave the world" "leave: asking the game to leave the world"
check loader "leave world asked" "leave: the loader asked the game to leave"
check mod "the game took the world join" "join world: the game took the world join"
check mod "screen after the servers tab was asked" "servers tab: screen after the servers tab was asked"
check mod "the game answered the join" "join result: the game answered the join"
check loader "actor position measured" "position: actor position measured"
check_value "join result: the player spawned in the server world" "$(grep -q 'Player Spawned' "$out_dir/bds.log" 2> /dev/null && echo yes)" "$(grep -m 1 -i 'player\|error' "$out_dir/bds.log" 2> /dev/null || echo 'no player line')"
check loader "$real_key" "key feed: first real key down"
for log in loader manager mod; do
  bad=$(count_lines "$log" "event not available|area not available|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no event, area or panic line in the $log log"; else echo "FAIL $bad event not available, area not available or panic lines in the $log log"; failures+=("$log log lines"); fi
done
echo "position measured: $(run_lines loader | grep -m 1 -o -E 'actor position measured.*' || echo none)"
echo "key feed: $(run_lines loader | grep -o -E 'key feed thread is the game thread.*|first real key down.*' | tr '\n' ' ')"
echo "join answer: $(run_lines mod | grep -m 1 -o -E 'the game answered the join.*' || echo none)"
echo "screen names: $(run_lines loader | grep -o -E 'top screen name .*' | tr '\n' ' ')"
for line in "${results[@]}"; do echo "tool: $line"; done
[ -f "$out_dir/screenshot.png" ] && echo "screenshot: $out_dir/screenshot.png"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-headless-input: pass"
else
  echo "ingame-headless-input: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
