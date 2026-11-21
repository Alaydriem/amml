entry "Biome.mId" {
  kind  = "field"
  notes = "draft"

  recipe {
    step "pattern" {
      bytes = "66 89 91 ?? ?? ?? ?? 48 8D 81 ?? ?? ?? ?? 48 89 45 ?? 0F 11 B1 ?? ?? ?? ?? 48 C7 81 ?? ?? ?? ?? ?? ?? ?? ?? C6 81 ?? ?? ?? ?? ??"
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "BiomeRegistry.lookupById" }
    step "pattern" { bytes = "0F B7 ?? ?? ?? ?? ?? 66 39 ?? 72 ?? 77 ??" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "Biome.mHash" {
  kind  = "field"
  notes = "draft"

  recipe {
    step "pattern" {
      bytes  = "48 89 4D ?? 48 81 C1 ?? ?? ?? ?? 41 0F 10 00 0F 29 45 ?? 48 8D 55 ?? 48 89 4D ?? E8"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 87 ?? ?? ?? ?? 48 89 45 ?? 48 8D 8E ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 45 ?? 49 89 E9 E8",
        "49 8B 9E ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 48 89 DA E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}
