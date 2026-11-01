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
