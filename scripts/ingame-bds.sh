# A Bedrock Dedicated Server for in-game runs that join a server, as DejaVu's fixture runs one. Source it after
# ingame-instance.sh (it needs $instance and $data_dir). The server of the instance's game version is downloaded once
# into <data_dir>/amml-bds/<version> and reused. Its identity key and the game's trust of that key are kept in
# <data_dir>/amml-join-trust: a person accepts the game's "Trust this server?" prompt once per instance, the run saves
# the game's storage_object, and later runs put the key and the trust back, so the join needs no person.

bds_trust_dir="$data_dir/amml-join-trust"
bds_pid=""

# The four-part game version and channel of the instance: the folder name gives the version, the package identity in
# appxmanifest.xml the channel (MinecraftWindowsBeta is Preview). Sets bds_version (leading zeros dropped) and bds_url.
bds_resolve() {
  local name parts part out=""
  name="$(basename "$instance")"
  name="${name%%-*}"
  IFS='.' read -r -a parts <<< "$name"
  for part in "${parts[@]}"; do
    part="${part#"${part%%[!0]*}"}"
    out="$out${out:+.}${part:-0}"
  done
  bds_version="$out"
  if grep -q 'Name="Microsoft.MinecraftWindowsBeta"' "$instance/appxmanifest.xml" 2> /dev/null; then
    bds_url="https://www.minecraft.net/bedrockdedicatedserver/bin-win-preview/bedrock-server-$bds_version.zip"
  else
    bds_url="https://www.minecraft.net/bedrockdedicatedserver/bin-win/bedrock-server-$bds_version.zip"
  fi
  bds_dir="$data_dir/amml-bds/$bds_version"
}

# Downloads and extracts the server when its executable is missing. The download site refuses a request with no
# browser agent.
bds_ensure() {
  bds_resolve
  [ -f "$bds_dir/bedrock_server.exe" ] && return 0
  local partial="$bds_dir.partial" zip="$data_dir/amml-bds/bedrock-server-$bds_version.zip"
  mkdir -p "$data_dir/amml-bds"
  if [ ! -f "$zip" ]; then
    echo "downloading $bds_url"
    curl --silent --show-error --location --fail --connect-timeout 30 --max-time 1200 --user-agent "Mozilla/5.0" \
      -o "$zip.partial" "$bds_url" || { echo "the server download failed: $bds_url"; rm -f "$zip.partial"; return 1; }
    mv "$zip.partial" "$zip"
  fi
  rm -rf "$partial"
  mkdir -p "$partial"
  # the Windows tar (bsdtar) reads zip archives; the Git Bash tar does not
  /c/Windows/System32/tar.exe -xf "$(cygpath -w "$zip")" -C "$(cygpath -w "$partial")" ||
    { echo "extracting $zip failed"; rm -rf "$partial"; return 1; }
  [ -f "$partial/bedrock_server.exe" ] || { echo "the server zip has no bedrock_server.exe"; rm -rf "$partial"; return 1; }
  rm -rf "$bds_dir"
  mv "$partial" "$bds_dir"
  rm -f "$zip"
}

# Sets the keys the run owns in server.properties ($1 the NetherNet signaling port, also the one UDP client port),
# keeps every other line.
bds_configure() {
  local port="$1" file="$bds_dir/server.properties" key value
  for pair in "transport=nethernet" "online-mode=true" "gamemode=creative" "difficulty=peaceful" "allow-cheats=true" \
    "default-player-permission-level=operator" "server-port=$port" "server-udp-ports=$port" \
    "level-name=amml-join-test" "allow-list=false"; do
    key="${pair%%=*}"
    value="${pair#*=}"
    if grep -q "^$key=" "$file"; then
      sed -i "s|^$key=.*|$key=$value|" "$file"
    else
      printf '%s=%s\r\n' "$key" "$value" >> "$file"
    fi
  done
}

# Puts the saved identity key into the server before it starts (the game trusts that key).
bds_install_key() {
  [ -f "$bds_trust_dir/server_identity_key.pem" ] || return 0
  mkdir -p "$bds_dir/keys"
  cp "$bds_trust_dir/server_identity_key.pem" "$bds_dir/keys/server_identity_key.pem"
}

# Saves the key the server made on its first start.
bds_save_key() {
  [ -f "$bds_dir/keys/server_identity_key.pem" ] || return 0
  mkdir -p "$bds_trust_dir"
  cmp -s "$bds_dir/keys/server_identity_key.pem" "$bds_trust_dir/server_identity_key.pem" 2> /dev/null ||
    cp "$bds_dir/keys/server_identity_key.pem" "$bds_trust_dir/server_identity_key.pem"
}

# The minecraftpe folders of the instance's players (not the Shared one).
bds_trust_folders() { ls -d "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftpe 2> /dev/null | grep -v '/Shared/' || true; }

# Copies the saved trust into the game before it starts. True when a saved trust exists.
bds_install_trust() {
  [ -f "$bds_trust_dir/storage_object" ] || return 1
  local pe
  while IFS= read -r pe; do
    [ -n "$pe" ] && cp "$bds_trust_dir/storage_object" "$pe/storage_object" && echo "installed the server trust into $pe"
  done < <(bds_trust_folders)
  return 0
}

# Saves the trust the game wrote after a person accepted the prompt.
bds_save_trust() {
  local pe
  mkdir -p "$bds_trust_dir"
  while IFS= read -r pe; do
    [ -n "$pe" ] && [ -f "$pe/storage_object" ] || continue
    cmp -s "$pe/storage_object" "$bds_trust_dir/storage_object" 2> /dev/null ||
      { cp "$pe/storage_object" "$bds_trust_dir/storage_object" && echo "saved the server trust from $pe"; }
  done < <(bds_trust_folders)
}

# Starts the server ($1 the log file) with its console on a pipe and waits up to 120 s for its ready line. With no
# saved identity key, `serveridentity save` writes the server's key to keys/server_identity_key.pem and the run keeps
# it, so the next start presents the same key and the game's saved trust matches.
bds_start() {
  local log waited=0 had_key=""
  # absolute: the server runs in its own folder
  mkdir -p "$(dirname "$1")"
  log="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
  rm -f "$log"
  [ -f "$bds_trust_dir/server_identity_key.pem" ] && had_key="yes"
  bds_install_key
  coproc BDS_PROC { cd "$bds_dir" && exec ./bedrock_server.exe > "$log" 2>&1; }
  bds_pid=$BDS_PROC_PID
  exec {bds_in}>&"${BDS_PROC[1]}"
  until grep -q "Server started" "$log" 2> /dev/null; do
    if [ "$waited" -ge 120 ] || ! kill -0 "$bds_pid" 2> /dev/null; then
      echo "the server did not start (see $log)"
      return 1
    fi
    sleep 1
    waited=$(( waited + 1 ))
  done
  if [ -z "$had_key" ]; then
    echo "serveridentity save" >&"$bds_in"
    waited=0
    until [ -f "$bds_dir/keys/server_identity_key.pem" ] || [ "$waited" -ge 15 ]; do sleep 1; waited=$(( waited + 1 )); done
    bds_save_key
    [ -f "$bds_trust_dir/server_identity_key.pem" ] && echo "saved the server identity key" || echo "serveridentity save wrote no key file"
  fi
  echo "server $bds_version started on port $(grep '^server-port=' "$bds_dir/server.properties" | tr -d '
' | cut -d '=' -f 2)"
}

# Stops the server this run started with its `stop` command, then ends it if it is still running after 15 s.
bds_stop() {
  [ -n "$bds_pid" ] || return 0
  local waited=0
  { echo "stop" >&"$bds_in"; } 2> /dev/null || true
  while kill -0 "$bds_pid" 2> /dev/null && [ "$waited" -lt 15 ]; do sleep 1; waited=$(( waited + 1 )); done
  kill "$bds_pid" 2> /dev/null || true
  bds_pid=""
}
