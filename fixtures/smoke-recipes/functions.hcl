entry "Smoke.HudMobEffectBackground" {
  kind  = "function"
  notes = "The one function loading the string textures/ui/hud_mob_effect_background"

  recipe {
    step "string_xref" {
      text = "textures/ui/hud_mob_effect_background"
    }
  }

  check "function_start" {}
}

entry "Smoke.ButtonBorderlessLight" {
  kind  = "function"
  notes = "The one function loading the string textures/ui/button_borderless_light"

  recipe {
    step "string_xref" {
      text = "textures/ui/button_borderless_light"
    }
  }

  check "function_start" {}
}

entry "Smoke.PersonaPlayerNameTitle" {
  kind  = "function"
  notes = "The one function loading the string persona.player_name.title"

  recipe {
    step "string_xref" {
      text = "persona.player_name.title"
    }
  }

  check "function_start" {}
}

