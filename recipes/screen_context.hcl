entry "ScreenContext.tessellator" {
  kind     = "field"
  notes    = "Offset in ScreenContext of the `Tessellator&` member, stored as a pointer: 0xB8 in 26.51. The header sum from frameAllocator (0x108, which extractNameTags reads as the LinearAllocator of the frame) gives it: commandListQueue 0x100, isCapturingPanorama and its padding 0xF8, frameDiscardContext 0xF0, profanityContext 0xD8, minecraftGraphics 0xC0, tessellator 0xB8. extractNameTags passes it as the Tessellator& argument of ActorRenderDispatcher::extractRenderTextObjects(Font&, Tessellator&, Actor&, ..): `mov r9,[rcx+0B8h]` (this in rcx, the hidden vector in rdx, the font in r8, the tessellator in r9). Recipe 0 is the first of the two call sites (`mov r9,[rcx+..]; lea rax,[rbp+..]; mov [rsp+..],rax; movss [rsp+..],xmm0; ... mov rcx,r14; mov rdx,r15; mov r8,[rbp+..]; call`). Recipe 1 is the second call site (`mov rax,[rbp+..]; mov r9,[rax+..]; mov rax,[rdi+..]; add rdi,imm; mov rax,[rax+8]; mov r10,[rip+..]; mov [rsp+..],rsi`). Both are in extractNameTags and share no bytes. Every register, displacement and immediate byte is a wildcard; the displacement of the r9 load is read. What changed in 6ad79ee4-17d51891a1119914 (26.60): the value is 0xB0 (the frame allocator is at 0x100 there, 8 bytes lower, so the header sum from it gives 0xB0, and both call sites pass `mov r9,[reg+0B0h]` as the Tessellator&). The old recipe 0 failed because the dispatcher argument moves changed register (`mov rdx,r15` became `mov rdx,r12`), so the ModRM bytes of those two moves are wildcards now. The old recipe 1 failed because the dispatcher pointer is read differently (`mov rax,[rbx]; mov rax,[rax+40h]` instead of an offset add on the renderer), so it is a pattern_any with one alternative per build. Class: would have survived with a better recipe for recipe 0; a real code change for recipe 1. Cold run result: failed (both recipes no match)."

  recipe {
    step "pattern" {
      bytes  = "4C 8B 89 ?? ?? ?? ?? 48 8D 45 ?? 48 89 44 24 ?? F3 0F 11 44 24 ?? 48 8D 45 ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 89 5C 24 ?? ?? 89 ?? ?? 89 ?? 4C 8B 85 ?? ?? ?? ?? E8"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 85 ?? ?? ?? ?? 4C 8B 88 ?? ?? ?? ?? 48 8B 87 ?? ?? ?? ?? 48 81 C7 ?? ?? ?? ?? 48 8B 40 ?? 4C 8B 15 ?? ?? ?? ?? 48 89 74 24 ??",
        "48 8B 85 ?? ?? ?? ?? 4C 8B 88 ?? ?? ?? ?? 48 8B 03 48 8B 40 ?? 4C 8B 15 ?? ?? ?? ?? 4C 89 74 24 ??",
      ]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 512
  }

}

entry "ScreenContext.guiData" {
  kind     = "field"
  notes    = "Offset in ScreenContext of the GuiData pointer: A0h on 26.60 and 26.62. The member is a Bedrock::NotNullNonOwnerPtr<GuiData> (24 bytes from +90h: the shared control block pointer, its counter block, then the raw GuiData pointer); the entry is the offset of the raw pointer, which GuiData.getGuiScale takes as this. Recipe 0 is MinecraftUIRenderContext::drawImage (slot 8 of MinecraftUIRenderContext.vtable): `mov rbx,[rax+..]; mov [rbp+..],rbx; mov [rbp+..],rcx; mov [rbp+..],rcx; mov r15,[rax+..]; mov [rbp+..],r15; test rbx,rbx; je; cmp byte ptr [rbx],..; je; movss xmm6,[r12]; mov rcx,r15; call <getGuiScale>`, read as the displacement of the load of r15. Recipe 1 is a second UI draw function that does the same on the ScreenContext in rdi (`mov r15,[rdi+..]; mov [rbp+..],r15; mov [rbp+..],rax; mov r13,[rdi+..]; mov [rbp+..],r13; test r15,r15; je; cmp byte ptr [r15],..; je; mov rcx,r13; call <getGuiScale>`), read as the displacement of the load of r13; the same function then loads the tessellator from [rdi+0B0h] (ScreenContext.tessellator), which shows rdi is the ScreenContext. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value A0h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 98 ?? ?? ?? ?? 48 89 5D ?? 48 89 4D ?? 48 89 4D ?? 4C 8B B8 ?? ?? ?? ?? 4C 89 7D ?? 48 85 DB 0F 84 ?? ?? ?? ?? 80 3B ?? 0F 84 ?? ?? ?? ?? F3 41 0F 10 34 24 4C 89 F9 E8"
      offset = 19
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8B BF ?? ?? ?? ?? 4C 89 7D ?? 48 89 45 ?? 4C 8B AF ?? ?? ?? ?? 4C 89 AD ?? ?? ?? ?? 4D 85 FF 0F 84 ?? ?? ?? ?? 41 80 3F ?? 0F 84 ?? ?? ?? ?? 4C 89 E9 E8"
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 96
    max = 256
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
