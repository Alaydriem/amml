# Which repos may publish into this registry, and which mods are core or default.
#
# A release is listed only when its repo matches the repo named here. Private repos do not publish
# (by convention: nothing enforces it), so @alaydriem/dejavu is not listed while its repo is private.
#
# core    the launcher installs the mod into every instance and never offers to remove it
# default the launcher installs the mod into new instances; the player can remove it

amml {
  repo = "alaydriem/amml"
}

mod "amml" "modmanager" {
  repo = "alaydriem/amml-modmanager"
  core = true
}

mod "alaydriem" "zoom" {
  repo    = "alaydriem/amml-zoom"
  default = true
}

mod "alaydriem" "minimap" {
  repo    = "alaydriem/amml-minimap"
  default = true
}

mod "alaydriem" "levels" {
  repo    = "alaydriem/amml-levels"
  default = true
}

mod "alaydriem" "freecam" {
  repo    = "alaydriem/amml-freecam"
  default = true
}

mod "alaydriem" "stratum" {
  repo    = "alaydriem/amml-stratum"
  default = true
}

mod "alaydriem" "heads" {
  repo    = "alaydriem/amml-heads"
  default = true
}

mod "alaydriem" "peek" {
  repo    = "alaydriem/amml-peek"
  default = true
}

mod "alaydriem" "toggles" {
  repo    = "alaydriem/amml-toggles"
  default = true
}
