#!/usr/bin/env bash
# Usage: updater-json.sh <version> <dir> [base_url]
# Writes <dir>/latest.json from the two .sig files in <dir>.
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: updater-json.sh <version> <dir> [base_url]" >&2
  exit 2
fi

version=$1
dir=$2
base_url=${3:-https://github.com/alaydriem/amml/releases/download/$version}

setup="amml-launcher-$version-windows-x64-setup.exe"
msi="amml-launcher-$version-windows-x64.msi"

for file in "$setup" "$setup.sig" "$msi" "$msi.sig"; do
  if [ ! -f "$dir/$file" ]; then
    echo "missing $dir/$file" >&2
    exit 1
  fi
done

setup_sig=$(tr -d '\r\n' < "$dir/$setup.sig")
msi_sig=$(tr -d '\r\n' < "$dir/$msi.sig")
pub_date=$(date -u +%Y-%m-%dT%H:%M:%SZ)

jq -n \
  --arg version "$version" \
  --arg notes "AMML $version" \
  --arg pub_date "$pub_date" \
  --arg setup_sig "$setup_sig" \
  --arg setup_url "$base_url/$setup" \
  --arg msi_sig "$msi_sig" \
  --arg msi_url "$base_url/$msi" \
  '{
    version: $version,
    notes: $notes,
    pub_date: $pub_date,
    platforms: {
      "windows-x86_64-nsis": {signature: $setup_sig, url: $setup_url},
      "windows-x86_64": {signature: $setup_sig, url: $setup_url},
      "windows-x86_64-msi": {signature: $msi_sig, url: $msi_url}
    }
  }' > "$dir/latest.json"
echo "wrote $dir/latest.json"
