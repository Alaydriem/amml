# Shared by the ingame-* mise tasks that run a standalone test instance (source it from the repo root).
# resolve_test_instance checks the instance and sets instance, exe and data_dir; pick_test_world picks the world and
# restores it from its pristine copy. Both exit the script with a line on a refusal. park_mods and restore_mods move
# the instance's mods aside for a run and back; pause_on_focus_lost_off and pause_on_focus_lost_restore change the
# pause on focus loss option for a run and back.

lower() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]'; }

# Sets instance, exe and data_dir for the instance folder named $1 under %LOCALAPPDATA%/amml/instances. Refuses a name
# that is not one folder name, a link out of the instances folder, the operator play instances, a missing or bad
# data_dir, a data_dir in %LOCALAPPDATA%/amml/data and an exe that does not import amml.dll.
resolve_test_instance() {
  local name="$1"
  [ -n "$LOCALAPPDATA" ] || { echo "LOCALAPPDATA is not set; the instances live under %LOCALAPPDATA%/amml/instances"; exit 1; }
  # one plain folder name: no separators, no `.` or `..`, no drive letter
  case "$name" in
    "" | */* | *\\* | .* | *:*) echo "--instance must be one folder name under %LOCALAPPDATA%/amml/instances, got '$name'"; exit 1 ;;
  esac
  local instances
  instances="$(cd "$LOCALAPPDATA/amml/instances" && pwd -P)"
  [ -d "$instances/$name" ] || { echo "no instance folder $instances/$name"; exit 1; }
  instance="$(cd "$instances/$name" && pwd -P)"
  # a junction or link that leads out of the instances folder is refused
  [ "$(dirname "$instance")" = "$instances" ] || { echo "$instances/$name resolves to $instance, outside $instances"; exit 1; }
  # the operator plays in these instances; tests use 1.26.63.01-test and 1.26.70.26-standalone
  case "$(lower "$(basename "$instance")")" in
    1.26.63.01 | 1.26.63.01-p)
      echo "$instance is an operator play instance and is not for tests; use a test instance such as 1.26.63.01-test (its data_dir is under %LOCALAPPDATA%/amml/test-data)"
      exit 1 ;;
  esac
  exe="$instance/Minecraft.Windows.exe"
  [ -f "$exe" ] || { echo "no Minecraft.Windows.exe in $instance"; exit 1; }
  # without a data_dir the game writes its worlds and options to the default folder of the operator's own game
  local data_line data_win shared_data
  data_line=$(grep -E '^[[:space:]]*data_dir[[:space:]]*=' "$instance/amml/instance.hcl" 2> /dev/null | head -n 1 || true)
  if [ -z "$data_line" ]; then
    echo "$instance/amml/instance.hcl with a data_dir line is missing; copy it from another AMML instance (data_dir is the folder that holds the test game data, under %LOCALAPPDATA%/amml) so the run does not use the default game data folder"
    exit 1
  fi
  data_win=$(printf '%s' "$data_line" | cut -d '"' -f 2 | tr -s '\\\\')
  data_dir=$(cygpath -u "$data_win" 2> /dev/null || true)
  if [ -z "$data_dir" ] || [ ! -d "$data_dir" ]; then echo "data_dir '$data_win' of $instance/amml/instance.hcl is not a folder"; exit 1; fi
  data_dir="$(cd "$data_dir" && pwd -P)"
  # %LOCALAPPDATA%/amml/data holds the operator's own worlds and options; a test run must never change it
  # both the folder as named and, when it is a junction or link, the folder it leads to
  local shared_named shared_real
  shared_named="$(cd "$LOCALAPPDATA/amml" && pwd -P)/data"
  shared_real="$(cd "$LOCALAPPDATA/amml/data" 2> /dev/null && pwd -P || echo "$shared_named")"
  for shared_data in "$shared_named" "$shared_real"; do
    case "$(lower "$data_dir")/" in
      "$(lower "$shared_data")"/*)
        echo "data_dir '$data_win' of $instance/amml/instance.hcl is the operator's game data $shared_data (or a folder in it); set data_dir to a folder under %LOCALAPPDATA%/amml/test-data/<instance>"
        exit 1 ;;
    esac
  done
  if ! grep -q -a -F "amml.dll" "$exe"; then
    echo "$exe does not import amml.dll: not a standalone instance; make one with mise run make-standalone-instance"
    exit 1
  fi
}

# Picks the world of the run under data_dir and sets world. Reads manual ("yes" for --manual-world: nothing to pick)
# and world (the --world folder name, or empty for the test world). The world folders of data_dir are
# <data_dir>/Minecraft Bedrock*/Users/<user>/games/com.mojang/minecraftWorlds (not the Shared user).
pick_test_world() {
  world_roots=()
  local root
  for root in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftWorlds; do
    [ -d "$root" ] || continue
    case "$root" in */Users/Shared/*) continue ;; esac
    world_roots+=("$root")
  done
  [ -z "$manual" ] || return 0
  [ -n "${world_roots[0]:-}" ] || { echo "no minecraftWorlds folder under $data_dir; start the instance once and create a world"; exit 1; }
  local folder base found
  if [ -n "$world" ]; then
    found=""
    for root in "${world_roots[@]}"; do [ -d "$root/$world" ] && found="$root/$world"; done
    [ -n "$found" ] || { echo "no world folder '$world' under the minecraftWorlds folders of $data_dir"; exit 1; }
    echo "world: $world ($found)"
    # only the task's own test worlds are restored; another world keeps its state
    case "$world" in
      amml-test | amml-test-[0-9]*) restore_test_world ;;
      *) echo "world $world is not a task test world (amml-test, amml-test-<n>): it is not restored from a pristine copy" ;;
    esac
    return 0
  fi
  # The test world is the folder amml-test the instance was seeded with, a folder amml-test-<n> made by this task (the
  # game rewrites levelname.txt from level.dat on save, so the folder name is the mark that lasts), or one whose
  # levelname.txt is amml-test
  for root in "${world_roots[@]}"; do
    for folder in "$root"/*/; do
      folder="${folder%/}"
      base="$(basename "$folder")"
      case "$base" in amml-test | amml-test-[0-9]*) world="$base"; break 2 ;; esac
      [ -f "$folder/levelname.txt" ] || continue
      if [ "$(tr -d '\r\n' < "$folder/levelname.txt")" = "amml-test" ]; then world="$base"; break 2; fi
    done
  done
  # The copy source is the most recently changed world of the release product folder (a preview build upgrades a
  # world past what a release build opens, so a preview world is used only when no release folder has one); the
  # task's own copies are never a source
  newest() {
    local r f
    for r in "$@"; do
      for f in "$r"/*/; do
        f="${f%/}"
        case "$(basename "$f")" in amml-test-[0-9]*) continue ;; esac
        [ -f "$f/level.dat" ] && stat -c '%Y %n' "$f"
      done
    done | sort -n -r | head -n 1 | cut -d ' ' -f 2-
  }
  local release_roots=() source_world="" n taken
  for root in "${world_roots[@]}"; do
    case "$root" in */Minecraft\ Bedrock\ Preview/*) ;; *) release_roots+=("$root") ;; esac
  done
  [ -n "${release_roots[0]:-}" ] && source_world=$(newest "${release_roots[@]}")
  [ -n "$source_world" ] || source_world=$(newest "${world_roots[@]}")
  if [ -z "$world" ]; then
    n=1
    while :; do
      taken=""
      for root in "${world_roots[@]}"; do [ -e "$root/amml-test-$n" ] && taken="yes"; done
      [ -z "$taken" ] && break
      n=$(( n + 1 ))
    done
    world="amml-test-$n"
  fi
  # the same folder name in every product folder, so whichever one the game reads holds the world
  for root in "${world_roots[@]}"; do
    if [ ! -d "$root/$world" ]; then
      [ -n "$source_world" ] || { echo "no world under $data_dir to copy; create one in the instance first"; exit 1; }
      cp -r "$source_world" "$root/$world"
      printf 'amml-test' > "$root/$world/levelname.txt"
      echo "copied world $source_world to $root/$world"
    fi
  done
  echo "world: $world"
  restore_test_world
}

# Gives each product folder's copy of world a fresh state from its pristine copy, because the game saves the player
# position in the world and a run that ends with the player blocked breaks the next run's movement checks. The
# pristine copy is <minecraftWorlds>/../amml-pristine-worlds/<world>, beside the worlds folder inside data_dir; when
# there is none yet, the world as it is now becomes the pristine copy. Changes nothing outside data_dir.
restore_test_world() {
  local root pristine
  for root in "${world_roots[@]}"; do
    [ -d "$root/$world" ] || continue
    pristine="$(dirname "$root")/amml-pristine-worlds/$world"
    case "$pristine" in "$data_dir"/*) ;; *) echo "pristine world $pristine is outside $data_dir"; exit 1 ;; esac
    if [ ! -d "$pristine" ]; then
      mkdir -p "$(dirname "$pristine")"
      cp -r "$root/$world" "$pristine"
      echo "made the pristine copy $pristine of world $world (once; later runs restore the world from it)"
      continue
    fi
    rm -rf "${root:?}/${world:?}"
    cp -r "$pristine" "$root/$world"
    echo "restored world $world from $pristine"
  done
}

# The posted focus loss of a run must leave the game in play, so the game does not open the pause screen on focus loss
# for the run: sets pause_menu_on_focus_lost:0 in the options.txt files under data_dir and records them in
# changed_options. Changes nothing outside data_dir; pause_on_focus_lost_restore sets them back.
pause_on_focus_lost_off() {
  changed_options=()
  local options
  for options in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftpe/options.txt; do
    [ -f "$options" ] || continue
    if grep -q '^pause_menu_on_focus_lost:1' "$options"; then
      sed -i 's/^pause_menu_on_focus_lost:1/pause_menu_on_focus_lost:0/' "$options"
      changed_options+=("$options")
      echo "set pause_menu_on_focus_lost:0 in $options"
    fi
  done
}

pause_on_focus_lost_restore() {
  local options
  for options in "${changed_options[@]}"; do
    [ -f "$options" ] || continue
    sed -i 's/^pause_menu_on_focus_lost:0/pause_menu_on_focus_lost:1/' "$options"
    echo "set pause_menu_on_focus_lost:1 back in $options"
  done
}

# Sets the options.txt key $1 from $2 to $3 in every options file of the instance's data_dir for the run, and
# records each change; run_options_restore puts every recorded value back.
run_option_set() {
  local key="$1" from="$2" to="$3" options
  for options in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftpe/options.txt; do
    [ -f "$options" ] || continue
    if grep -q "^$key:$from" "$options"; then
      sed -i "s/^$key:$from/$key:$to/" "$options"
      run_options+=("$options|$key|$from|$to")
      echo "set $key:$to in $options"
    fi
  done
}

run_options_restore() {
  local entry options key from to
  for entry in "${run_options[@]}"; do
    IFS='|' read -r options key from to <<< "$entry"
    [ -f "$options" ] || continue
    sed -i "s/^$key:$to/$key:$from/" "$options"
    echo "set $key:$from back in $options"
  done
  run_options=()
}

# Moves every mod folder <namespace>/<name> of the loader's mod root <instance>/mods/amml to the parked folder
# <instance>/mods-parked/<stamp>/before for the run, the run's test mods included when they were there before. Sets
# mods_root and parked; restore_mods puts the mods back.
park_mods() {
  mods_root="$instance/mods/amml"
  parked="$instance/mods-parked/$(date +%Y%m%d-%H%M%S)"
  mkdir -p "$mods_root" "$parked/before"
  local ns mod
  for ns in "$mods_root"/*/; do
    [ -d "$ns" ] || continue
    ns="${ns%/}"
    for mod in "$ns"/*/; do
      [ -d "$mod" ] || continue
      mod="${mod%/}"
      mkdir -p "$parked/before/$(basename "$ns")"
      mv "$mod" "$parked/before/$(basename "$ns")/$(basename "$mod")"
      echo "parked $(basename "$ns")/$(basename "$mod")"
    done
  done
}

# Moves the run's test mods, each argument <namespace>/<name>, to <parked>/run and puts the parked mods back, so the
# instance's mod set is as before the run. Does nothing when park_mods did not run. A move never goes to a target that
# exists: `mv` would put the folder inside it. On any failure the parked folder stays and its path is printed; when
# every mod is back, the parked folder (empty, or holding only the run's test mod copies) is deleted.
restore_mods() {
  [ -n "${parked:-}" ] || return 0
  local ns mod test target failed=""
  for test in "$@"; do
    if [ -d "$mods_root/$test" ]; then
      mkdir -p "$parked/run/$(dirname "$test")"
      target="$parked/run/$test"
      if [ -e "$target" ] || ! mv "$mods_root/$test" "$target"; then
        echo "moving the test mod $test out failed"
        failed="yes"
      fi
    fi
  done
  for ns in "$parked/before"/*/; do
    [ -d "$ns" ] || continue
    ns="${ns%/}"
    for mod in "$ns"/*/; do
      [ -d "$mod" ] || continue
      mod="${mod%/}"
      mkdir -p "$mods_root/$(basename "$ns")"
      target="$mods_root/$(basename "$ns")/$(basename "$mod")"
      if [ -e "$target" ] || ! mv "$mod" "$target"; then
        echo "putting back $(basename "$ns")/$(basename "$mod") failed ($target is in the way); it stays in $parked/before"
        failed="yes"
      fi
    done
  done
  if [ -n "$failed" ]; then
    echo "the parked mods stay in $parked"
    return 0
  fi
  rm -rf "${parked:?}"
  rmdir "$instance/mods-parked" 2> /dev/null || true
  echo "the instance's mods are back"
}

# The world of one mod's run (P2-19): sets world to amml-test-$1 and world_roots as pick_test_world does. A product
# folder without that world gets it once, copied from the pristine copy of the shared amml-test world (else from the
# amml-test folder itself), with levelname.txt amml-test; then each copy is restored from its own pristine copy
# (restore_test_world). The shared amml-test world and its pristine copy are only read. Exits the script with a line
# when the mod name is not one plain name or no source world is found.
seed_mod_world() {
  local mod="$1" root source pristine
  case "$mod" in
    "" | */* | *\* | .* | *:* | *" "*) echo "seed_mod_world needs one plain mod name, got '$mod'"; exit 1 ;;
  esac
  world_roots=()
  for root in "$data_dir"/Minecraft\ Bedrock*/Users/*/games/com.mojang/minecraftWorlds; do
    [ -d "$root" ] || continue
    case "$root" in */Users/Shared/*) continue ;; esac
    world_roots+=("$root")
  done
  [ -n "${world_roots[0]:-}" ] || { echo "no minecraftWorlds folder under $data_dir; start the instance once and create a world"; exit 1; }
  world="amml-test-$mod"
  for root in "${world_roots[@]}"; do
    [ -d "$root/$world" ] && continue
    pristine="$(dirname "$root")/amml-pristine-worlds/amml-test"
    if [ -d "$pristine" ]; then source="$pristine"; elif [ -d "$root/amml-test" ]; then source="$root/amml-test"; else source=""; fi
    [ -n "$source" ] || { echo "no amml-test world (or its pristine copy) in $root to copy for $world"; exit 1; }
    cp -r "$source" "$root/$world"
    printf 'amml-test' > "$root/$world/levelname.txt"
    echo "copied world $source to $root/$world"
  done
  echo "world: $world"
  restore_test_world
}
