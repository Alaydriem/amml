#!/usr/bin/env bash
# Usage: update-loop.sh <from version> <to version> <port>
# Run from launcher/app. Builds two signed launcher bundles that talk to a local
# latest.json, then serves the newer one from 127.0.0.1.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: update-loop.sh <from version> <to version> <port>" >&2
  exit 2
fi
from=$1
to=$2
port=$3

if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  echo "TAURI_SIGNING_PRIVATE_KEY is empty: set it to the updater private key (and TAURI_SIGNING_PRIVATE_KEY_PASSWORD) before launcher-app:update-loop" >&2
  exit 1
fi

root=../..
loop=$root/update-loop
bundle=$root/target/release/bundle
base_url="http://127.0.0.1:$port"

rm -rf "$loop"
mkdir -p "$loop/install" "$loop/serve"

for v in "$from" "$to"; do
  config="{\"version\":\"$v\",\"plugins\":{\"updater\":{\"endpoints\":[\"$base_url/latest.json\"],\"dangerousInsecureTransportProtocol\":true}}}"
  mise run launcher-app:bundle -- --config "$config"
  nsis="$bundle/nsis/AMML Launcher_${v}_x64-setup.exe"
  msi="$bundle/msi/AMML Launcher_${v}_x64_en-US.msi"
  for file in "$nsis" "$nsis.sig" "$msi" "$msi.sig"; do
    if [ ! -f "$file" ]; then
      echo "missing $file after the bundle build for $v" >&2
      exit 1
    fi
  done
  prefix="amml-launcher-$v-windows-x64"
  if [ "$v" = "$from" ]; then
    cp "$nsis" "$loop/install/$prefix-setup.exe"
  fi
  if [ "$v" = "$to" ]; then
    cp "$nsis" "$loop/serve/$prefix-setup.exe"
    cp "$nsis.sig" "$loop/serve/$prefix-setup.exe.sig"
    cp "$msi" "$loop/serve/$prefix.msi"
    cp "$msi.sig" "$loop/serve/$prefix.msi.sig"
  fi
done

bash scripts/updater-json.sh "$to" "$loop/serve" "$base_url"

installer=$(cd "$loop/install" && pwd -W 2>/dev/null || pwd)
cat <<TEXT

Update loop ready.
  1. Install: $installer\amml-launcher-$from-windows-x64-setup.exe
  2. Open the launcher, then Settings > About.
  3. Expect a notice for $to. Click Update. The app restarts and About shows $to.
  4. This window logs each request: latest.json, then the setup exe.
Server: $base_url/latest.json (127.0.0.1 only). Press Ctrl+C to stop.

TEXT

exec node scripts/serve-dir.mjs "$loop/serve" "$port"
