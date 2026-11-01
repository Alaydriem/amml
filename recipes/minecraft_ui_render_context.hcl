entry "MinecraftUIRenderContext.vtable" {
  kind     = "global"
  notes    = "The vtable of MinecraftUIRenderContext (no RTTI, 41 slots), a helper for the slot entries of its virtuals. The constructor stores it first (`lea rax,[rip+..]; mov [rcx],rax`), then the client instance at +8, the ScreenContext at +10h, the MinecraftUIMeasureStrategy (its own vtable) at +18h, the text alpha 1.0 at +38h, the text item vector at +80h and the image item vector at +98h. UIScene::render builds one on its stack, hands it to ScreenView::render and flushes it; it has two callers. Recipe 0 is the head of the constructor: `mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov [rcx+8],rdx; mov [rcx+10h],r8; mov rax,[rdx]; mov rax,[rax+..]; lea rdx,[rbp+..]; mov rcx,rbx; call [guard]`, read as the RIP target of the lea. Recipe 1 reaches the constructor from its caller: the prologue of UIScene::render (found by its own prologue and a thread-local guard test), then `mov r8,rdi; mov r9,rbx; call <ctor>` read as the call target, then the store of the vtable in that function. The two recipes are in three functions and the call path shares no pattern bytes with recipe 0. Every displacement, immediate and branch byte is a wildcard. The check is section .rdata. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value F2D18C0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 89 51 ?? 4C 89 41 ?? 48 8B 02 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? 48 89 D9 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" { bytes = "55 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 D7 48 89 CB 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ??" }
    step "pattern" {
      bytes  = "49 89 F8 49 89 D9 E8 ?? ?? ?? ??"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 89 51 ?? 4C 89 41 ??" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "MinecraftUIRenderContext.mScreenContext" {
  kind     = "field"
  notes    = "Offset in MinecraftUIRenderContext of mScreenContext, the ScreenContext* the context draws into: 10h on 26.60 (the client instance pointer is at +8). The slot 13 virtual loads it and reads the tessellator from it (the existing ScreenContext.tessellator entry, +0B0h on 26.60), which is how the value was confirmed. Recipe 0 is the constructor head: `lea rax,[rip+..]; mov [rcx],rax; mov [rcx+8],rdx; mov [rcx+..],r8; mov rax,[rdx]`, read as the displacement of the store of r8. Recipe 1 is the start of flushText (a different function): `mov [rbp+..],rcx; mov rax,[rcx+..]; xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov rcx,[rax+..]; test rcx,rcx; je; lock inc dword ptr [rcx+8]`, read as the displacement of the load of rax, which reads the ScreenContext to get the font repository weak pointer. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 10h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 89 51 ?? 4C 89 41 ?? 48 8B 02"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 8D ?? ?? ?? ?? 48 8B 41 ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 85 C9 74 ?? F0 FF 41 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "MinecraftUIRenderContext.fillRectangle" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::fillRectangle(RectangleArea const& area, mce::Color const& color, float alpha) in the MinecraftUIRenderContext vtable: 16 on 26.60. rcx the context, rdx the RectangleArea (four f32: x0, x1, y0, y1), r8 the Color (four f32, r g b a; only r, g and b are used), xmm3 the alpha. It draws at once with the game's ui_fill_color material (a call to 24E5500h with x0, y0, x1, y1 and the color), so nothing is queued and no flush is needed. Recipe 0 is the prologue (`push r14; push rsi; push rdi; push rbx; sub rsp,..; movaps [rsp+..],xmm6; movaps xmm6,xmm3; mov rbx,r8; mov rdi,rdx; mov rsi,rcx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rcx,[rip+..]; test rcx,rcx; jne; mov rcx,[rip+..]`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the argument setup before the draw call (`movsd xmm0,[rbx]; movsd [rsp+..],xmm0; movss xmm0,[rbx+8]; movss [rsp+..],xmm0; movss [rsp+..],xmm6; movss xmm0,[rdi+0Ch]; movss xmm2,[rdi]; movss xmm1,[rdi+4]; movss xmm3,[rdi+8]; mov rdx,[rsi+..]; lea rax,[rsp+..]; mov [rsp+..],rax; movss [rsp+..],xmm0; movss [rsp+..],xmm1; call`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes; the call sites of this virtual are inside UI draw functions that cannot be told apart by their bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 16 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "41 56 56 57 53 48 83 EC ?? 0F 29 74 24 ?? 0F 28 F3 4C 89 C3 48 89 D7 48 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 0D ?? ?? ?? ?? 48 85 C9 75 ?? 48 8B 0D ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "F2 0F 10 03 F2 0F 11 44 24 ?? F3 0F 10 43 ?? F3 0F 11 44 24 ?? F3 0F 11 74 24 ?? F3 0F 10 47 ?? F3 0F 10 17 F3 0F 10 4F ?? F3 0F 10 5F ?? 48 8B 56 ?? 48 8D 44 24 ?? 48 89 44 24 ?? F3 0F 11 44 24 ?? F3 0F 11 4C 24 ?? E8 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  check "range" {
    min = 0
    max = 40
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "MinecraftUIRenderContext.drawRectangle" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::drawRectangle(RectangleArea const& area, mce::Color const& color, float alpha, int thickness) in the MinecraftUIRenderContext vtable: 15 on 26.60. rcx the context, rdx the RectangleArea (x0, x1, y0, y1), r8 the Color, xmm3 the alpha, the int thickness at [rsp+28h] on entry; it draws four filled bars (top, bottom, left, right) with four calls to 24E5500h, and truncates the coordinates to int first. Recipe 0 is the prologue (`push r15; push r14; push rsi; push rdi; push rbp; push rbx; sub rsp,..; movaps [rsp+..],xmm9; movaps [rsp+..],xmm8; movaps [rsp+..],xmm7; movaps [rsp+..],xmm6; movaps xmm6,xmm3; mov rbx,r8; mov rdi,rdx; mov rsi,rcx; mov ebp,[rsp+..]`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the coordinate setup before the first bar (`movsd xmm0,[rbx]; movsd [rsp+..],xmm0; movss xmm0,[rbx+8]; movss [rsp+..],xmm0; movss [rsp+..],xmm6; movss xmm9,[rdi+0Ch]; cvttss2si r15d,xmm9; movss xmm0,[rdi]; movss xmm1,[rdi+4]; cvttss2si ebx,xmm1; movss xmm2,[rdi+8]; cvttss2si eax,xmm2; cvttss2si edi,xmm0; add eax,ebp; xorps xmm3,xmm3; cvtsi2ss xmm3,eax`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 15 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "41 57 41 56 56 57 55 53 48 81 EC ?? ?? ?? ?? 44 0F 29 8C 24 ?? ?? ?? ?? 44 0F 29 44 24 ?? 0F 29 7C 24 ?? 0F 29 74 24 ?? 0F 28 F3 4C 89 C3 48 89 D7 48 89 CE 8B AC 24 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "F2 0F 10 03 F2 0F 11 44 24 ?? F3 0F 10 43 ?? F3 0F 11 44 24 ?? F3 0F 11 74 24 ?? F3 44 0F 10 4F ?? F3 45 0F 2C F9 F3 0F 10 07 F3 0F 10 4F ?? F3 0F 2C D9 F3 0F 10 57 ?? F3 0F 2C C2 F3 0F 2C F8 01 E8 0F 57 DB F3 0F 2A D8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  check "range" {
    min = 0
    max = 40
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "MinecraftUIRenderContext.drawText" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::drawText(Font const& font, RectangleArea const& area, std::string_view text, mce::Color const& color, float alpha, ui::TextAlignment alignment, TextMeasureData const& measure, CaretMeasureData const& caret) in the MinecraftUIRenderContext vtable: 5 on 26.60 (the overload that takes a std::string&& is the next slot, 6: the compiler put the overloads in reverse order). rcx the context, rdx the Font, r8 the RectangleArea, r9 a pointer to a 16-byte string_view copy, then on the stack from [rsp+28h]: the Color pointer, the f32 alpha, the alignment (u32), the TextMeasureData pointer and the CaretMeasureData pointer. It queues a 68h-byte text item (the string_view is copied as is, so the text bytes must stay alive until flushText) and returns. Recipe 0 is the prologue (`push r15; push r14; push r13; push r12; push rsi; push rdi; push rbp; push rbx; sub rsp,..; movaps [rsp+..],xmm6; mov rbx,rcx; mov rax,[rsp+..]; mov rsi,[rsp+..]; mov r10d,[rsp+..]; movss xmm6,[rsp+..]; mov r13,[rsp+..]`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the fill of the queued item (`movups xmm0,[r9]; movsd xmm1,[rsi]; mov [r15],rdx; movups xmm2,[r8]; movups [r15+8],xmm2; movups [r15+18h],xmm0; mov byte ptr [r15+38h],1; movss xmm0,[r13]; movss [r15+40h],xmm0; ...`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 5 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC ?? 0F 29 74 24 ?? 48 89 CB 48 8B 84 24 ?? ?? ?? ?? 48 8B B4 24 ?? ?? ?? ?? 44 8B 94 24 ?? ?? ?? ?? F3 0F 10 B4 24 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "41 0F 10 01 F2 0F 10 0E 49 89 17 41 0F 10 10 41 0F 11 57 ?? 41 0F 11 47 ?? 41 C6 47 ?? ?? F3 41 0F 10 45 ?? F3 41 0F 11 47 ?? F3 41 0F 10 45 ?? F3 41 0F 11 47 ?? F3 41 0F 10 45 ?? F3 41 0F 11 47 ?? F3 41 0F 11 77 ?? 41 89 47 ?? F2 41 0F 11 4F ?? 45 89 57 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  check "range" {
    min = 0
    max = 40
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "MinecraftUIRenderContext.flushText" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::flushText(float deltaTime, std::optional<float> const& maxLifetime) in the MinecraftUIRenderContext vtable: 7 on 26.60. rcx the context, xmm1 the delta time, r8 the optional<float> by value in one register (low 32 bits the value, bit 32 the engaged flag). It draws the text items queued by drawText (the vector at +80h, 68h bytes each) with the fonts and clears the queue. Recipe 0 is the prologue (eight pushes, `sub rsp,..`, `lea rbp,[rsp+..]`, nine xmm saves, `mov qword ptr [rbp+..],-2`, `mov rdi,r8`). Recipe 1 is the first read of the ScreenContext (`mov [rbp+..],rcx; mov rax,[rcx+10h]; xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov rcx,[rax+98h]; test rcx,rcx; je; lock inc dword ptr [rcx+8]`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 7 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 44 0F 29 BD ?? ?? ?? ?? 44 0F 29 B5 ?? ?? ?? ?? 44 0F 29 AD ?? ?? ?? ?? 44 0F 29 A5 ?? ?? ?? ?? 44 0F 29 9D ?? ?? ?? ?? 44 0F 29 95 ?? ?? ?? ?? 44 0F 29 8D ?? ?? ?? ?? 44 0F 29 85 ?? ?? ?? ?? 0F 29 BD ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4C 89 C7 F3 0F 11 8D ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 89 8D ?? ?? ?? ?? 48 8B 41 ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 85 C9 74 ?? F0 FF 41 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  check "range" {
    min = 0
    max = 40
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
