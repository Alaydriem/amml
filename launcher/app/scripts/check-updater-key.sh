#!/usr/bin/env bash
# Usage: check-updater-key.sh <tauri.conf.json> <pub file> <sig>...
# Exits 1 unless the minisign key ID is the same in plugins.updater.pubkey, the .pub file and every .sig.
set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "usage: check-updater-key.sh <tauri.conf.json> <pub file> <sig>..." >&2
  exit 2
fi

conf=$1
pub=$2
shift 2

# Key ID in hex: line 2 of the decoded minisign text is base64 of algorithm (2 bytes), key ID (8 bytes), data.
key_id() {
  local text=$1
  printf '%s' "$text" | tr -d '\r\n ' | base64 -d | sed -n '2p' | tr -d '\r' | base64 -d 2>/dev/null \
    | head -c 10 | tail -c 8 | od -An -tx1 | tr -d ' \n'
}

conf_text=$(jq -j '.plugins.updater.pubkey // empty' "$conf")
if [ -z "$conf_text" ]; then
  echo "no plugins.updater.pubkey in $conf" >&2
  exit 1
fi

expected=$(key_id "$conf_text")
if [ "${#expected}" -ne 16 ]; then
  echo "cannot read a key ID from plugins.updater.pubkey in $conf" >&2
  exit 1
fi
echo "$expected  $conf (plugins.updater.pubkey)"

status=0
for file in "$pub" "$@"; do
  id=$(key_id "$(cat "$file")" || true)
  if [ "${#id}" -ne 16 ]; then
    echo "unreadable  $file" >&2
    status=1
    continue
  fi
  if [ "$id" = "$expected" ]; then
    echo "$id  $file"
  else
    echo "$id  $file  DIFFERS from $expected" >&2
    status=1
  fi
done

if [ "$status" -ne 0 ]; then
  echo "updater key ID mismatch" >&2
fi
exit "$status"
