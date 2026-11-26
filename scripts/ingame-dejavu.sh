# The body of the ingame-dejavu mise task (mise.toml has the description and the flags). Run it through
# `mise run ingame-dejavu -- --instance <name>`, which sets usage_instance, usage_dejavu and usage_modmanager and runs it
# from the repo root. Builds the release loader, modmanager and dejavu, installs only modmanager and dejavu into a
# standalone test instance (the instance's other mods are parked for the run and put back after), and runs one game
# through three runs, all driven by dejavu's own MCP server (pre-paired by a token file the script writes: pairing
# needs a person, and no person is there):
#   run A  local    the world amml-test-dejavu records for 30 s, the MCP leave tool leaves it, the capture is saved
#   run B  remote   the instance's BDS starts, the MCP join tool joins it, the game plays 30 s and leaves
#   run C  playback the local recording opens through MCP and plays; the free camera at roll 30; one render of 60 frames
#                   (the clock is held and stepped); one camera-key marker set; the replay closes
# Posted mouse messages never reach the game in the background, so no step needs the mouse. A tool call that fails
# prints a line and the run goes on: the pass lines below decide the result.
#
# The pass lines (the dejavu log unless noted; the text is the proof table of the dejavu port plan):
#   core          `dejavu enabled`, `manager linked`; loader `event offered` for every contract event and
#                 `area adapter` for every contract area
#   capture       `capture started kind=local`, `capture packets out=<n> in=<n>` both > 0, `capture saved`; run B
#                 `capture started kind=remote`
#   protocol      `protocol version=<n>` (2225 on 26.63, 2259 on 70.26)
#   world_folder  `world folder opened`, `world folder closed`
#   join          `replay join started host=127.0.0.1`, `viewer connected`
#   camera        `camera free set`, `camera pose applied frames=<n>` (> 0), `camera free released`
#   fov           `fov override result=<deg>`
#   body          `body hidden on=true`
#   viewer        `viewer look applied`
#   clock         `clock held`, `clock step applied stepped_us=<n>`, `clock barrier count=<n>`, `clock released`
#   graphics      `graphics options set vsync=0 max_framerate=0`, `graphics options restored`
#   markers       `shapes applied count=<n>`, `look ray hit`
#   guards        `idle guard on`
#   all           none of `event not available`, `area not available`, `loader too old`, `panic` in the three logs
# Printed, not checked: the first `join started names`, `option key`, `packet wire time`, `packet wire out thread`,
# `packet wire in thread`, `app focus thread` and `time hold released` lines, and the `ConnectionStarted` lines of the
# loader log. Pass/fail: exactly one `connection started protocol=<n>` line per `world join started local=<bool>` line
# in the dejavu log (one ConnectionStarted per join).
set -e
. scripts/ingame-instance.sh
resolve_test_instance "${usage_instance:-}"
dejavu="${usage_dejavu:-${AMML_DEJAVU_DIR:-../dejavu}}"
[ -d "$dejavu" ] || { echo "no dejavu folder '$dejavu' (--dejavu, else AMML_DEJAVU_DIR, else ../dejavu)"; exit 1; }
dejavu="$(cd "$dejavu" && pwd -P)"
[ -f "$dejavu/mise.toml" ] || { echo "$dejavu has no mise.toml: not a dejavu checkout"; exit 1; }
modmanager="${usage_modmanager:-${AMML_MODMANAGER_DIR:-../modmanager}}"
[ -d "$modmanager" ] || { echo "no modmanager folder '$modmanager' (--modmanager, else AMML_MODMANAGER_DIR, else ../modmanager)"; exit 1; }
modmanager="$(cd "$modmanager" && pwd -P)"
[ -f "$modmanager/mise.toml" ] || { echo "$modmanager has no mise.toml: not a modmanager checkout"; exit 1; }
. scripts/game-pid.sh
. scripts/ingame-common.sh
. scripts/ingame-bds.sh
before=""
game_pid=""
launched=""
require_no_game "$instance"
# the world of this run: a copy of amml-test with its own pristine copy
seed_mod_world dejavu
# the posted focus messages must leave the game in play; the multiplayer safety notices stop the scripted server
# join until a person accepts them
run_options=()
changed_options=()
on_exit() {
  local rc=$?
  end_game || echo "ending the game failed"
  bds_stop
  restore_mods @amml/modmanager @alaydriem/dejavu
  [ -z "${shared_tokens:-}" ] || rm -f "$shared_tokens"
  [ -z "${dejavu_out:-}" ] || rm -rf "$dejavu_out"
  # the run makes the @alaydriem folder; an empty one is not left behind
  [ -z "${mods_root:-}" ] || rmdir "$mods_root/@alaydriem" 2> /dev/null || true
  pause_on_focus_lost_restore
  run_options_restore
  exit "$rc"
}
trap on_exit EXIT
pause_on_focus_lost_off
run_option_set do_not_show_multiplayer_online_safety_warning 0 1
run_option_set do_not_show_multiplayer_ip_safety_warning 0 1

cargo build -p amml-loader --release
cp target/release/amml.dll "$instance/amml.dll"
rm -rf "${instance:?}/amml/recipes" "${instance:?}/amml/data"
mkdir -p "$instance/amml/data" "$instance/amml/cache"
cp -r recipes "$instance/amml/recipes"
cp data/*.hcl "$instance/amml/data/"
mise --cd "$modmanager" run package
mise --cd "$dejavu" run package
[ -d "$modmanager/dist/@amml/modmanager" ] || { echo "the modmanager build wrote no $modmanager/dist/@amml/modmanager"; exit 1; }
[ -d "$dejavu/dist/@alaydriem/dejavu" ] || { echo "the dejavu build wrote no $dejavu/dist/@alaydriem/dejavu"; exit 1; }
# the dedicated server run B joins (downloaded once per instance)
bds_ensure

# a compat failure or a forbidden Win32 import warning stops the run before the game starts
mkdir -p target
for dist in "$modmanager/dist" "$dejavu/dist"; do
  compat_out="target/ingame-dejavu-compat-$(basename "$(dirname "$dist")").txt"
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
cp -r "$dejavu/dist/@alaydriem/dejavu" "$mods_root/@alaydriem/dejavu"
# MCP on, on a port of its own per instance, and one token written before the game starts (the server reads its
# token file at start; a pairing request would wait for a person)
case "$(basename "$instance")" in
  1.26.63.01-test) mcp_port=28371; bds_port=28212 ;;
  *) mcp_port=28372; bds_port=28213 ;;
esac
dejavu_data="$mods_root/@alaydriem/dejavu/data"
mkdir -p "$dejavu_data"
# renders go to a test folder under the instance's data dir (the default Videos folder may not exist); removed on exit
dejavu_out="$data_dir/dejavu-renders"
mkdir -p "$dejavu_out"
dejavu_out_win=$(cygpath -m "$dejavu_out")
ffmpeg_exe=$(command -v ffmpeg || true); case "$ffmpeg_exe" in ""|*.exe) ;; *) [ -f "$ffmpeg_exe.exe" ] && ffmpeg_exe="$ffmpeg_exe.exe" ;; esac
if [ -n "$ffmpeg_exe" ]; then
  printf '{"developer":{"enabled":true},"mcp":{"enabled":true,"port":%s},"ffmpeg_path":"%s","export_folder":"%s"}\n' "$mcp_port" "$(cygpath -m "$ffmpeg_exe")" "$dejavu_out_win" > "$dejavu_data/settings.json"
else
  echo "no ffmpeg: the render and clock rows cannot pass"
  printf '{"developer":{"enabled":true},"mcp":{"enabled":true,"port":%s},"export_folder":"%s"}\n' "$mcp_port" "$dejavu_out_win" > "$dejavu_data/settings.json"
fi
token=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
token_json=$(printf '{"tokens":[{"token":"%s","client":"ingame-dejavu","issued_ms":%s}]}' "$token" "$(( $(date +%s) * 1000 ))")
printf '%s\n' "$token_json" > "$dejavu_data/mcp.tokens.json"
# the mod reads the tokens from the loader's shared mod data folder, which for this instance is <data_dir>/mod-data
# (never the operator's %LOCALAPPDATA%/amml/mod-data); the file is removed on exit
shared_tokens_dir="$data_dir/mod-data/@alaydriem/dejavu"
shared_tokens="$shared_tokens_dir/mcp.tokens.json"
mkdir -p "$shared_tokens_dir"
printf '%s\n' "$token_json" > "$shared_tokens"
echo "installed @amml/modmanager and @alaydriem/dejavu from $dejavu/dist (MCP port $mcp_port)"

logs="$instance/amml/logs"
manager_log="$logs/@amml/modmanager.log"
dejavu_log="$logs/@alaydriem/dejavu.log"
mkdir -p "$logs"
loader_start=$(size_of "$logs/loader.log")
manager_start=$(size_of "$manager_log")
dejavu_start=$(size_of "$dejavu_log")
out_dir="target/ingame-dejavu/$(basename "$instance")"
mkdir -p "$out_dir"
# logs rotate at about 1 MB: the lines of a run are the current log and every archive written since this marker
marker="$out_dir/.start"
touch "$marker"
ps_run() { powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$PWD/scripts/$1")" "${@:2}"; }
# the lines of this run of log loader, manager or dejavu
run_lines() {
  case "$1" in
    loader) since_rotated "$logs/loader.log" "$loader_start" "$marker" ;;
    manager) since_rotated "$manager_log" "$manager_start" "$marker" ;;
    dejavu) since_rotated "$dejavu_log" "$dejavu_start" "$marker" ;;
    bds) [ -f "$out_dir/bds.log" ] && cat "$out_dir/bds.log" ;;
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
# one MCP tool call: prints the result text (or `error: ...`) and keeps it in mcp_last
mcp_last=""
mcp() {
  mcp_last=$(ps_run mcp-call.ps1 -Port "$mcp_port" -Token "$token" -Tool "$1" -ArgsJson "$2" | tr -d '\r') || true
  echo "mcp $1: ${mcp_last:0:200}"
}
# one engine command (a JSON object with a "cmd" field) through the engine_command tool
engine() {
  local json
  json=$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g')
  mcp engine_command "{\"command\":\"$json\"}"
}
# a string field of the last reply: `reply_field last_capture`
reply_field() { printf '%s' "$mcp_last" | grep -o -E "\"$1\":\"[^\"]*\"" | head -n 1 | cut -d '"' -f 4; }
# the leave tool; a failed call sets leave_missing: the waits and runs that need a leave are then skipped, and the
# summary names the port dependency
leave_missing=""
leave() {
  mcp leave_world '{}'
  case "$mcp_last" in error:*) leave_missing="yes"; echo "port dependency: leave_world missing (the leave steps of this run did not happen)" ;; esac
}
# the protocol of the instance's build
case "$(basename "$instance")" in
  *1.26.63*) protocol='2225' ;;
  *1.26.70*) protocol='2259' ;;
  *) protocol='[1-9][0-9]*' ;;
esac
# every play-time line a check needs, with the check's own pattern, waited for before the close
final_waits=(
  "capture started kind=local"
  "capture packets out=[1-9][0-9]* in=[1-9][0-9]*"
  "protocol version=$protocol([^0-9]|$)"
  "world folder opened"
  "world folder closed"
  "capture started kind=remote"
  "replay join started host=127\.0\.0\.1"
  "viewer connected"
  "camera free set"
  "camera pose applied frames=[1-9][0-9]*"
  "camera free released"
  "fov override result="
  "body hidden on=true"
  "viewer look applied"
  "clock held"
  "clock step applied stepped_us=[0-9]+"
  "clock barrier count=[0-9]+"
  "clock released"
  "graphics options set vsync=0 max_framerate=0"
  "graphics options restored"
  "shapes applied count=[0-9]+"
  "look ray hit"
  "idle guard on"
)

bds_configure "$bds_port"
if ! bds_install_trust; then
  echo "no saved server trust: the server join of run B waits for a person to accept the game's trust prompt"
fi
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

echo "--- start"
wait_line manager "manager enabled" 180 || true
wait_line dejavu "dejavu enabled" 120 || true
wait_line dejavu "manager linked" 120 || true
joined=""
if wait_line loader 'top screen name .*hud_screen' 120; then
  joined="yes"
  sleep 5
fi

local_recording=""
if [ -n "$joined" ] && alive; then
  echo "--- run A (local capture)"
  # a capture may start by itself at the join; start_capture then says it is refused, and the line is the proof
  mcp start_capture '{}'
  wait_line dejavu "capture started kind=local" 30 || true
  sleep 30
  mcp capture_status '{}'
  leave
  recorder_state=""
  if [ -z "$leave_missing" ]; then
    wait_line dejavu "capture saved" 60 || true
    wait_line dejavu "world folder closed" 30 || true
    sleep 3
    mcp capture_status '{}'
    local_recording=$(reply_field last_capture)
    recorder_state=$(reply_field state)
    echo "local recording: ${local_recording:-none}; recorder state: ${recorder_state:-unknown}"
  fi

  echo "--- run B (remote capture)"
  case "$recorder_state" in
    idle | off) ;;
    *) run_b_skipped="yes"; echo "run B skipped: the recorder state is '${recorder_state:-unknown}', not idle or off after the leave" ;;
  esac
  if [ -z "${run_b_skipped:-}" ]; then
  bds_start "$out_dir/bds.log" || true
  mcp join_server "{\"host\":\"127.0.0.1\",\"port\":$bds_port}"
  if [ ! -f "$bds_trust_dir/storage_object" ]; then
    echo "accept 'Trust and Join' in the game window now (waiting up to 240 s)"
    wait_line bds "Player Spawned" 240 || true
  fi
  wait_line bds "Player Spawned" 90 || true
  wait_line dejavu "capture started kind=remote" 60 || true
  # the recorder keeps a file only after a record action, as in run A
  mcp start_capture '{}'
  sleep 30
  # the game writes its trust of the server key into storage_object once a person accepted; keep it for later runs
  bds_save_trust
  leave
  wait_line dejavu "capture saved" 60 1 || true
  bds_stop
  sleep 3
  fi

  echo "--- run C (playback)"
  echo "if the game asks to trust the replay server, accept 'Trust and Join' in the game window (saved for later runs)"
  if [ -n "$local_recording" ]; then
    mcp open_replay "{\"path\":\"$local_recording\"}"
    wait_line dejavu "replay join started host=127.0.0.1" 120 || true
    wait_line dejavu "viewer connected" 120 || true
    mcp wait_for '{"condition":{"until":"viewer_joined"},"timeout_ms":60000}'
    engine '{"cmd":"play"}'
    sleep 5
    # the free camera at roll 30, high over the world origin
    engine '{"cmd":"move_viewer","take_view":true,"pose":{"coordinates":[0.0,100.0,0.0],"orientation":{"pitch":30.0,"yaw":0.0,"roll":30.0}}}'
    sleep 5
    # one scene with camera keys (the in-game marker set) and two timelapse keys one second apart (60 frames at 60 fps)
    engine '{"cmd":"add_scene","name":"ingame"}'
    scene_id=$(printf '%s' "$mcp_last" | grep -o -E '"id":"[^"]*"' | head -n 1 | cut -d '"' -f 4)
    scene_id="${scene_id:-scene-1}"
    scene_num=$(printf '%s' "$scene_id" | tr -dc '0-9')
    engine "{\"cmd\":\"add_track\",\"scene_id\":\"$scene_id\",\"track\":\"camera\"}"
    engine "{\"cmd\":\"add_key\",\"scene_id\":\"$scene_id\",\"track\":\"camera\",\"time_ms\":2000,\"value\":{\"coordinates\":[0.0,100.0,0.0],\"orientation\":{\"pitch\":30.0,\"yaw\":0.0}}}"
    engine "{\"cmd\":\"add_key\",\"scene_id\":\"$scene_id\",\"track\":\"camera\",\"time_ms\":3000,\"value\":{\"coordinates\":[8.0,100.0,8.0],\"orientation\":{\"pitch\":30.0,\"yaw\":90.0}}}"
    engine "{\"cmd\":\"add_track\",\"scene_id\":\"$scene_id\",\"track\":\"timelapse\"}"
    engine "{\"cmd\":\"add_key\",\"scene_id\":\"$scene_id\",\"track\":\"timelapse\",\"time_ms\":2000}"
    engine "{\"cmd\":\"add_key\",\"scene_id\":\"$scene_id\",\"track\":\"timelapse\",\"time_ms\":3000}"
    # the scene camera and the markers follow the active scene
    engine "{\"cmd\":\"set_active_scene\",\"id\":\"$scene_id\"}"
    # one camera key through the studio (the look ray runs only for a studio key); its view is 4 blocks above the
    # recorded player, looking almost straight down, with the replay paused (a ray at pitch 30 leaves the loaded chunks)
    engine '{"cmd":"pause"}'
    mcp state '{}'
    pos=$(printf '%s' "$mcp_last" | grep -o -E '"players_now":\[\{[^]]*"position":\[[-0-9.eE]+,[-0-9.eE]+,[-0-9.eE]+\]' | grep -o -E '\[[-0-9.eE]+,[-0-9.eE]+,[-0-9.eE]+\]$' | tr -d '[]' || true)
    px=$(printf '%s' "$pos" | cut -d, -f1); py=$(printf '%s' "$pos" | cut -d, -f2); pz=$(printf '%s' "$pos" | cut -d, -f3)
    vy=$(awk -v y="${py:-100}" 'BEGIN { printf "%.2f", y + 4 }')
    mcp set_view "{\"x\":${px:-0},\"y\":$vy,\"z\":${pz:-0},\"yaw\":0.0,\"pitch\":80.0}"
    sleep 2
    mcp ui_command "{\"command\":{\"AddKeyHere\":{\"scene\":${scene_num:-1},\"kind\":\"Camera\",\"time_ms\":2500}}}"
    # the seek releases the taken view and puts the playhead before the camera keys
    engine '{"cmd":"seek","time_ms":1500}'
    sleep 3
    engine '{"cmd":"play"}'
    wait_line dejavu "camera free set" 20 || true
    mcp markers '{"hidden":false}'
    wait_line dejavu "shapes applied count=" 30 || true
    wait_line dejavu "look ray hit" 30 || true
    mcp render_scene "{\"scene_id\":${scene_num:-1},\"preset\":{\"format\":\"H264\",\"resolution\":\"P1080\",\"fps\":60,\"quality\":0.5,\"sidecar\":false}}"
    mcp wait_for '{"condition":{"until":"render_done"},"timeout_ms":180000}'
    wait_line dejavu "clock released" 60 || true
    mcp close_replay '{}'
    wait_line dejavu "camera free released" 30 || true
    wait_line dejavu "graphics options restored" 30 || true
  else
    echo "no local recording: run C did not happen"
  fi
  # every play-time line a check needs, before the close (skipped when the leave tool is missing: they cannot come)
  if [ -z "$leave_missing" ]; then
    echo "--- final waits"
    for pattern in "${final_waits[@]}"; do
      wait_line dejavu "$pattern" 20 || true
    done
  fi
fi

echo "--- close"
end_game
sleep 2
# storage_object holds the trust of the BDS and of the replay server; save it again after run C so a replay trust a
# person accepted is installed before the next run
bds_save_trust
echo "--- loader.log (this run)"
run_lines loader
echo "--- @amml/modmanager.log (this run)"
run_lines manager
echo "--- @alaydriem/dejavu.log (this run)"
run_lines dejavu

echo "--- summary"
failures=()
if [ -n "$leave_missing" ]; then
  echo "FAIL port dependency: leave_world missing (the DejaVu MCP server has no leave tool; runs A, B and C could not complete)"
  failures+=("port dependency: leave_world missing")
fi
if [ -n "${run_b_skipped:-}" ]; then echo "note run B was skipped (see the run B line above)"; fi
check() {
  local log="$1" pattern="$2" label="$3" min="${4:-1}" got
  got=$(count_lines "$log" "$pattern")
  if [ "$got" -ge "$min" ]; then echo "ok   $label"; else echo "FAIL $label ($log log: $got of $min)"; failures+=("$label"); fi
}
echo "ok   compat on $modmanager/dist and $dejavu/dist"
for event in frame.update client.tick world.joined world.left mods.loaded registry.provider_ready registry.provider_gone \
  input.lease_suspended input.lease_resumed input.lease_lost network.packet_wire network.connection_started \
  world.join_started world.server_join_starting client.join_server_result camera.setup camera.fov player.moved \
  time.step_applied time.frame_end time.latency_barrier client.app_focus_lost client.app_focus_gained \
  client.idle_screen_open world.storage_opened world.storage_closed; do
  check loader "event offered .*\"${event//./\.}\"" "event offered $event"
done
for area in client.read client.top_screen network.chunk_cache client.world_info client.join_server \
  client.leave_world client.in_gameplay client.can_join camera.pose camera.free camera.hide_actor actor.read \
  actor.look_write time.control client.focus_replay client.activity options.int world.shapes world.raycast \
  world.actors_in network.protocol; do
  check loader "area adapter .*\"${area//./\.}\"" "area adapter $area"
done
check dejavu "dejavu enabled" "core: dejavu enabled"
check dejavu "manager linked" "core: manager linked"
check dejavu "capture started kind=local" "capture started kind=local"
check dejavu "capture packets out=[1-9][0-9]* in=[1-9][0-9]*" "capture packets (out and in > 0)"
check dejavu "capture saved" "capture saved (run A)"
check dejavu "capture started kind=remote" "capture started kind=remote (run B)"
check dejavu "protocol version=$protocol([^0-9]|$)" "protocol version=$protocol"
check dejavu "world folder opened" "world folder opened"
check dejavu "world folder closed" "world folder closed"
check dejavu "replay join started host=127\.0\.0\.1" "replay join started host=127.0.0.1"
check dejavu "viewer connected" "viewer connected"
check dejavu "camera free set" "camera free set"
check dejavu "camera pose applied frames=[1-9][0-9]*" "camera pose applied (frames > 0)"
check dejavu "camera free released" "camera free released"
check dejavu "fov override result=" "fov override result"
check dejavu "body hidden on=true" "body hidden on=true"
check dejavu "viewer look applied" "viewer look applied"
check dejavu "clock held" "clock held"
check dejavu "clock step applied stepped_us=[0-9]+" "clock step applied stepped_us"
check dejavu "clock barrier count=[0-9]+" "clock barrier count"
check dejavu "clock released" "clock released"
check dejavu "graphics options set vsync=0 max_framerate=0" "graphics options set vsync=0 max_framerate=0"
check dejavu "graphics options restored" "graphics options restored"
check dejavu "shapes applied count=[0-9]+" "shapes applied count"
check dejavu "look ray hit" "look ray hit"
check dejavu "idle guard on" "idle guard on"
for log in loader manager dejavu; do
  bad=$(count_lines "$log" "event not available|area not available|loader too old|panic")
  if [ "$bad" -eq 0 ]; then echo "ok   no unavailable, too-old or panic line in the $log log"; else echo "FAIL $bad unavailable, too-old or panic lines in the $log log"; failures+=("$log log lines"); fi
done
# the first value of each lead-check line, loader log first then dejavu log (info only)
for label in "join started names" "option key" "packet wire time" "packet wire out thread" "packet wire in thread" \
  "app focus thread" "time hold released"; do
  first=$( { run_lines loader; run_lines dejavu; } | grep -F -m 1 "$label" || true)
  if [ -n "$first" ]; then echo "info $label: $first"; else echo "info $label: no line in this run"; fi
done
echo "info ConnectionStarted lines in the loader log: $(count_lines loader 'ConnectionStarted|connection started|connection_started')"
# exactly one ConnectionStarted per join: the dejavu log has one `connection started protocol=<n>` line per start and
# one `world join started local=<bool>` line per JoinStarted; the counts must be equal and not 0
starts=$(count_lines dejavu 'connection started protocol=[0-9]+')
joins=$(count_lines dejavu 'world join started local=(true|false)')
if [ "$joins" -ge 1 ] && [ "$starts" -eq "$joins" ]; then
  echo "ok   one connection start per join ($starts starts, $joins joins)"
else
  echo "FAIL one connection start per join (dejavu log: $starts 'connection started' lines, $joins 'world join started' lines)"
  failures+=("one connection start per join")
fi
echo "info replay joins in the dejavu log: $(count_lines dejavu 'join started|replay join started')"
if [ "${#failures[@]}" -eq 0 ]; then
  echo "ingame-dejavu: pass"
else
  echo "ingame-dejavu: fail (${#failures[@]}): ${failures[*]}"
  exit 1
fi
