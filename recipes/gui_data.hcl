entry "GuiData.getGuiScale" {
  kind     = "function"
  notes    = "float GuiData::getGuiScale() const: rcx the GuiData, the scale in xmm0 (screen pixels per GUI unit). When the editor GUI scale flag of the GuiData is set it asks a service for the editor scale (and falls back to the member when the service is gone); otherwise it returns mGuiScale (+5Ch on 26.60 and 26.62; mInvGuiScale, its inverse, is the next f32). MinecraftUIRenderContext::drawImage calls it for each coordinate to snap positions and sizes to the pixel grid. The GuiData comes from the ScreenContext (ScreenContext.guiData). Recipe 0 is the head of the body after the prologue (`mov rsi,rcx; cmp byte ptr [rcx+..],..; jne; mov rcx,[rsi+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rbp+..]; call [guard]`), then function_start_of. Recipe 1 is the first call site in MinecraftUIRenderContext::drawImage (slot 8 of MinecraftUIRenderContext.vtable): `movss xmm6,[r12]; mov rcx,r15; call <this>`, read as the call target. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end with function_start_of, so it counts). The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value E7E290h on 6ad79ee4-17d51891a1119914 (26.60) and E7E150h on 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" { bytes = "48 89 CE 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F3 41 0F 10 34 24 4C 89 F9 E8 ?? ?? ?? ??"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
