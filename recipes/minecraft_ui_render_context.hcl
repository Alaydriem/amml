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

entry "MinecraftUIRenderContext.drawImage" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::drawImage(BedrockTextureData const& texture, glm::vec2 const& position, glm::vec2 const& size, glm::vec2 const& uv, glm::vec2 const& uvSize, bool) in the MinecraftUIRenderContext vtable: 8 on 26.60 and 26.62. rcx the context, rdx the BedrockTextureData (the first pointer of the mce::TexturePtr that getTexture returns; its first 10h bytes are a shared_ptr that flushImages copies; the item keeps the pointer, not a copy, so the texture must stay alive until flushImages; passing the TexturePtr itself crashes in the flush, proven in game on 26.62), r8 the position and r9 the size (two f32 each, GUI units), then on the stack from [rsp+28h]: the uv pointer, the uv size pointer (normalized texture coordinates) and the bool (stored in the item at +28h; drawNineslice passes 0). It snaps the rectangle to the pixel grid (position times the GUI scale truncated, size times the GUI scale rounded up, both then times the inverse GUI scale), queues a 30h-byte image item in the vector at +98h and returns; nothing draws until flushImages. Recipe 0 is the prologue and the argument moves (`push rbp; push r15..r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; four movaps xmm saves; mov qword ptr [rbp+..],-2; mov r14,r9; mov r12,r8; mov [rbp+..],rdx; mov rdi,rcx; mov r13,[rbp+..]; mov rsi,[rbp+..]; movzx eax,byte ptr [rbp+..]; mov [rbp+..],al`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the fill of the queued item (`mov rax,[rbp+..]; mov [rbx],rax; movss [rbx+8],xmm6; movss [rbx+0Ch],xmm7; movss [rbx+10h],xmm8; movss [rbx+14h],xmm9; movss xmm0,[rsi]; movss [rbx+18h],xmm0; ...; movzx eax,byte ptr [rbp+..]; mov [rbx+28h],al`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes; drawNineslice (slot 9) fills the same item with other registers and does not match. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 layout: it refuses as not for this build on 26.51. Value 8 on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 44 0F 29 4D ?? 44 0F 29 45 ?? 0F 29 7D ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 4D 89 CE 4D 89 C4 48 89 55 ?? 48 89 CF 4C 8B AD ?? ?? ?? ?? 48 8B B5 ?? ?? ?? ?? 0F B6 85 ?? ?? ?? ?? 88 45 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 8B 45 ?? 48 89 03 F3 0F 11 73 ?? F3 0F 11 7B ?? F3 44 0F 11 43 ?? F3 44 0F 11 4B ?? F3 0F 10 06 F3 0F 11 43 ?? F3 0F 10 46 ?? F3 0F 11 43 ?? F3 41 0F 10 45 ?? F3 0F 11 43 ?? F3 41 0F 10 45 ?? F3 0F 11 43 ?? 0F B6 45 ?? 88 43 ??" }
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

entry "MinecraftUIRenderContext.flushImages" {
  kind     = "vslot"
  notes    = "Slot of void MinecraftUIRenderContext::flushImages(mce::Color const& color, float alpha, HashedString const& material) in the MinecraftUIRenderContext vtable: 10 on 26.60 and 26.62. rcx the context, rdx the Color (r, g and b are read, a is not), xmm2 the alpha, r9 the material name. It draws every image item queued by drawImage (the vector at +98h, 30h bytes each) as quads of one mesh with the texture of the last queued item, so one flush must cover the items of one texture only; then it clears the queue. The color and the alpha go to the ScreenContext shader color (+30h). When the material string is empty (`cmp qword ptr [r9+18h],0`, the std::string size) it uses the game's static HashedString ui_textured_and_glcolor. Recipe 0 is the prologue and the first moves (`push rbp; push r15..r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; mov r14,[rcx+98h]; mov rbx,[rcx+0A0h]; cmp r14,rbx; je; movaps xmm6,xmm2; mov rax,[rcx+10h]; mov rdi,[rax+0B0h]; cmp byte ptr [rdi+..],0; mov [rbp+..],rcx; mov [rbp+..],r9; mov [rbp+..],rdx`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the shader color store and the material choice (`movsd [rax],xmm1; movss [rax+8],xmm0; movss [rax+0Ch],xmm6; mov byte ptr [rax+10h],1; mov rax,[rbp+..]; cmp qword ptr [rax+18h],0; lea r8,[rip+..]; cmovne r8,rax; lea rdx,[rip+..]; lea rcx,[rbp+..]; call`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 layout: it refuses as not for this build on 26.51. Value 10 on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4C 8B B1 ?? ?? ?? ?? 48 8B 99 ?? ?? ?? ?? 49 39 DE 0F 84 ?? ?? ?? ?? 0F 28 F2 48 8B 41 ?? 48 8B B8 ?? ?? ?? ?? 80 BF ?? ?? ?? ?? ?? 48 89 4D ?? 4C 89 4D ?? 48 89 55 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "F2 0F 11 08 F3 0F 11 40 ?? F3 0F 11 70 ?? C6 40 ?? ?? 48 8B 45 ?? 48 83 78 ?? ?? 4C 8D 05 ?? ?? ?? ?? 4C 0F 45 C0 48 8D 15 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ??" }
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

entry "MinecraftUIRenderContext.getTexture" {
  kind     = "vslot"
  notes    = "Slot of mce::TexturePtr MinecraftUIRenderContext::getTexture(ResourceLocation const& location, bool forceReload) in the MinecraftUIRenderContext vtable: 32 (20h) on 26.60 and 26.62. rcx the context, rdx the hidden return slot (a 20h-byte TexturePtr: a shared_ptr to the BedrockTextureData, then a shared_ptr to a ResourceLocation; returned in rax), r8 the ResourceLocation (38h bytes: the u32 file system at +0, the std::string path at +8, the FNV-1a 64 hash of the path at +28h, the full hash at +30h), r9 the bool. An empty path gives an empty TexturePtr; file system 10 (StoreCache) goes to the texture group at +78h; any other file system goes to the UI texture group at +58h (TextureGroup::getTexture with the load flags 100000001h), and when that group is gone it returns the game's default TexturePtr. Recipe 0 is the head after the prologue (`mov rbx,rdx; cmp qword ptr [r8+18h],0; je; cmp dword ptr [r8],0Ah; jne; mov rax,[rcx+68h]; test rax,rax; jne; mov dword ptr [0],0DEADC0DEh; cmp byte ptr [rax],0; jne; mov dword ptr [0],0DEADC0DEh; mov rcx,[rcx+78h]; add rcx,18h; mov rax,100000001h; mov [rsp+20h],rax; mov byte ptr [rsp+28h],0; mov rdx,rbx; call`), then function_start_of and slot_of MinecraftUIRenderContext.vtable. Recipe 1 is the fallback at the end, which copies the default TexturePtr from four globals (`test rdi,rdi; jne; xorps xmm0,xmm0; movups [rbx],xmm0; mov rax,[rip+..]; test rax,rax; je; lock inc dword ptr [rax+8]; ...; mov [rbx+10h],rcx; mov [rbx+18h],rax`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard (the file system id 0Ah too). The check is a range over the table. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 layout: it refuses as not for this build on 26.51. Value 32 (20h) on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" { bytes = "48 89 D3 49 83 78 ?? ?? 74 ?? 41 83 38 ?? 0F 85 ?? ?? ?? ?? 48 8B 41 ?? 48 85 C0 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 80 38 ?? 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 49 ?? 48 83 C1 ?? 48 B8 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 44 24 ?? C6 44 24 ?? ?? 48 89 DA E8 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftUIRenderContext.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 85 FF 75 ?? 0F 57 C0 0F 11 03 48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? F0 FF 40 ?? 48 8B 05 ?? ?? ?? ?? EB ?? 31 C0 48 8B 0D ?? ?? ?? ?? 48 89 0B 48 89 43 ?? 0F 11 43 ?? 48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? F0 FF 40 ?? 48 8B 05 ?? ?? ?? ?? EB ?? 31 C0 48 8B 0D ?? ?? ?? ?? 48 89 4B ?? 48 89 43 ??" }
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

entry "MinecraftUIRenderContext.mQueuedImages" {
  kind     = "field"
  notes    = "Offset in MinecraftUIRenderContext of mQueuedImages, the std::vector of 30h-byte image items that drawImage (slot 8) appends to and flushImages (slot 10) draws and clears: 98h on every build in the data (begin at +98h, end at +0A0h, capacity at +0A8h). A mod reads begin and end to see whether the queue holds items. Found from the two virtuals: flushImages reads [rcx+98h] and [rcx+0A0h] in its prologue and leaves the loop when they are equal, and after the draw sets end to begin; drawImage loads end from +0A0h, compares it with the capacity at +0A8h, stores the item and adds 30h to end, and on the slow path subtracts begin (+98h) from end to get the size before it grows the vector. The constructor (see MinecraftUIRenderContext.vtable) also sets up the image item vector at +98h. Recipe 0 is the clear at the end of flushImages (`mov rax,[r14+..]; cmp rax,[r14+..]; je; mov [r14+..],rax; xorps xmm0,xmm0; mov rsi,[rbp+..]; movaps/movups [rbp+..],xmm0; test rsi,rsi; je; lock dec dword ptr [rsi+8]; jne; mov rax,[rsi]; mov rax,[rax]; mov rcx,rsi; call [rip+..]`), read as the displacement of the load of begin. The store opcode byte (0F 29 movaps on 26.60 and later, 0F 11 movups on 26.51) is a wildcard. Recipe 1 is the push_back in drawImage: a first pattern on the fast path (`mov rbx,[rdi+..]; cmp rbx,[rdi+..]; je; mov rax,[rbp+..]; mov [rbx],rax; movss [rbx+..],xmm6; movss [rbx+..],xmm7; movss [rbx+..],xmm8; movss [rbx+..],xmm9`), then, inside the same function, the head of the slow path (`mov r15,rbx; sub r15,[rdi+..]; mov rax,r15; sar rax,..`), read as the displacement of the sub (begin). The second pattern is not unique in the exe on 26.51 (a second match in another function), so it runs only inside the function of the first. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window after the text item vector (+80h to +97h) up to 100h. No entry_is gate: the vector is at 98h and both recipes run on 26.51 as on the later builds. Value 98h on 6aa482fd-2658118eba0dbac5 (26.51), 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" { bytes = "49 8B 86 ?? ?? ?? ?? 49 3B 86 ?? ?? ?? ?? 74 ?? 49 89 86 ?? ?? ?? ?? 0F 57 C0 48 8B 75 ?? 0F ?? 45 ?? 48 85 F6 74 ?? F0 FF 4E ?? 75 ?? 48 8B 06 48 8B 00 48 89 F1 FF 15" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" { bytes = "48 8B 9F ?? ?? ?? ?? 48 3B 9F ?? ?? ?? ?? 74 ?? 48 8B 45 ?? 48 89 03 F3 0F 11 73 ?? F3 0F 11 7B ?? F3 44 0F 11 43 ?? F3 44 0F 11 4B ??" }
    step "pattern" {
      bytes  = "49 89 DF 4C 2B BF ?? ?? ?? ?? 4C 89 F8 48 C1 F8 ??"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 136
    max = 256
  }
}
