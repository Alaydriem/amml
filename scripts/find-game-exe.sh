#!/usr/bin/env bash
# Prints the game executable that the game tests use.
# Usage: find-game-exe.sh [instances_dir]   (default: $LOCALAPPDATA/amml/instances)
# Pick: the highest version by the leading dotted number of the folder name; on a tie, the folder with
# no suffix first, then name order. In that folder: Minecraft.Windows.exe.bak, .orig, then the exe.
# Reads only. Exits 1 with a message that names the folder when nothing fits.
set -u

root="${1:-${LOCALAPPDATA:-}/amml/instances}"
if [ ! -d "$root" ]; then
  echo "find-game-exe: no instances folder: $root" >&2
  exit 1
fi

lines=""
for dir in "$root"/*/; do
  [ -d "$dir" ] || continue
  name="$(basename "$dir")"
  version="$(printf '%s' "$name" | grep -oE '^[0-9]+(\.[0-9]+)*')" || continue
  suffix="${name#"$version"}"
  key=""
  IFS=. read -r -a parts <<< "$version"
  for part in "${parts[@]}"; do
    key="$key$(printf '%08d' "$((10#$part))")."
  done
  if [ -z "$suffix" ]; then rank=0; else rank=1; fi
  lines="$lines$key	$rank	$name
"
done

if [ -z "$lines" ]; then
  echo "find-game-exe: no instance folder in $root" >&2
  exit 1
fi

picked="$(printf '%s' "$lines" | LC_ALL=C sort -t "$(printf '\t')" -k1,1r -k2,2n -k3,3 | head -n 1 | cut -f3)"
folder="$root/$picked"
for file in Minecraft.Windows.exe.bak Minecraft.Windows.exe.orig Minecraft.Windows.exe; do
  if [ -f "$folder/$file" ]; then
    echo "$folder/$file"
    exit 0
  fi
done
echo "find-game-exe: no Minecraft.Windows.exe(.bak/.orig) in $folder" >&2
exit 1
