entry "GuiData.getGuiScale" {
  kind     = "function"
  notes    = "float GuiData::getGuiScale() const: rcx the GuiData, the scale in xmm0 (screen pixels per GUI unit). When the editor GUI scale flag of the GuiData is set it asks a service for the editor scale (and falls back to the member when the service is gone); otherwise it returns mGuiScale (+5Ch on 26.60 and 26.62; mInvGuiScale, its inverse, is the next f32). MinecraftUIRenderContext::drawImage calls it for each coordinate to snap positions and sizes to the pixel grid. The GuiData comes from the ScreenContext (ScreenContext.guiData). Recipe 0 is the head of the body after the prologue (`mov rsi,rcx; cmp byte ptr [rcx+..],..; jne; mov rcx,[rsi+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rbp+..]; call [guard]`), then function_start_of. Recipe 1 is the first call site in MinecraftUIRenderContext::drawImage (slot 8 of MinecraftUIRenderContext.vtable): `movss xmm6,[r12]; mov rcx,r15; call <this>`, read as the call target. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end with function_start_of, so it counts). Value E7E290h on 6ad79ee4-17d51891a1119914 (26.60) and E7E150h on 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (70.26): the body is the same code (the editor flag test, the virtual call on the service holder, the lookup of the editor scale at +18h, the fallback to mGuiScale at +5Ch), compiled with a smaller push list (rbp, rsi), a short `jne` and a one-byte displacement for the service virtual, so the 26.62 pattern did not match; recipe 1 changed with the drawImage body (the call sits after `mov [rbp-..],rdx` instead of a `mov r12`-based sequence, and the two bytes before the movss differ). Each recipe is now a pattern_any, one alternative per shape. Evidence on 70.26: the head of the 26.63 body (flag test, then the service virtual call, then the lookup with the +18h read and the +5Ch fallback) is the body of the function that the new patterns find (decompile shape and instruction order the same); the first call in the drawImage body of 70.26 (the function that loads the ScreenContext at +10h, the GuiData pointer at +0A0h and calls this function after the `test; je; cmp byte ptr [..],0; je` run, as 26.63 does) is the same function, and it has 88 direct callers (83 on 26.63). Would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 CE 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15",
        "48 89 CE 80 B9 ?? ?? ?? ?? ?? 75 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? FF 15",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? F3 41 0F 10 34 24 4C 89 F9 E8 ?? ?? ?? ??",
        "89 55 ?? F3 41 0F 10 ?? 4C 89 F9 E8 ?? ?? ?? ??",
      ]
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
