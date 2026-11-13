#!/usr/bin/env bash
# Usage: release-files.sh <version> <msix name> <msix publisher> <msix publisher display>
# Run from launcher/app. Writes the launcher release files into ../../launcher-dist.
set -euo pipefail

if [ "$#" -lt 4 ] || [ -z "$1" ]; then
  echo "usage: release-files.sh <version> <msix name> <msix publisher> <msix publisher display>" >&2
  exit 2
fi

version=$1
msix_name=$2
msix_publisher=$3
msix_publisher_display=$4

root=../..
bundle=$root/target/release/bundle
dist=$root/launcher-dist

nsis_src="$bundle/nsis/AMML Launcher_${version}_x64-setup.exe"
msi_src="$bundle/msi/AMML Launcher_${version}_x64_en-US.msi"
exe_src=$root/target/release/amml-launcher-app.exe

for file in "$nsis_src" "$nsis_src.sig" "$msi_src" "$msi_src.sig" "$exe_src"; do
  if [ ! -f "$file" ]; then
    echo "missing $file (run launcher-app:bundle for $version first)" >&2
    exit 1
  fi
done

bash scripts/check-updater-key.sh src-tauri/tauri.conf.json src-tauri/amml-launcher.key.pub "$nsis_src.sig" "$msi_src.sig"

rm -rf "$dist"
mkdir -p "$dist"
prefix="amml-launcher-$version-windows-x64"
cp "$exe_src" "$dist/$prefix.exe"
cp "$nsis_src" "$dist/$prefix-setup.exe"
cp "$nsis_src.sig" "$dist/$prefix-setup.exe.sig"
cp "$msi_src" "$dist/$prefix.msi"
cp "$msi_src.sig" "$dist/$prefix.msi.sig"

powershell -NoProfile -ExecutionPolicy Bypass -File scripts/pack-msix.ps1 -Version "$version" \
  -Name "$msix_name" -Publisher "$msix_publisher" -PublisherDisplay "$msix_publisher_display" \
  -Out "$dist/$prefix.msix"

bash scripts/updater-json.sh "$version" "$dist"

(
  cd "$dist"
  : > launcher-SHA256SUMS.txt
  for file in "$prefix.exe" "$prefix-setup.exe" "$prefix-setup.exe.sig" "$prefix.msi" "$prefix.msi.sig" \
    "$prefix.msix" latest.json; do
    sha256sum "$file" >> launcher-SHA256SUMS.txt
  done
)
ls "$dist"
