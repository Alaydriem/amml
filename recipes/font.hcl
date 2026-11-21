entry "Font.getLineLength" {
  kind     = "vslot"
  notes    = "Slot of int Font::getLineLength(std::string_view, float fontSize, bool showColorSymbol) in the Font vtable: 6 in 26.51 (header order: ~Font 0, switchFontsource 1, a path pair getter 2, fetchPage 3, drawCached 4, supportsChar 5, getLineLength 6; Font has no virtual base). The call site shows the registers: rcx = the Font, rdx = a pointer to the 16-byte string_view (a copy on the stack: `lea rdx,[rbp-8]` with the data pointer at [rbp-8] and the length at [rbp]), xmm2 = the float font size (`movss xmm2,[rip+..]`), r9b = the bool (`xor r9d,r9d`); the int is returned in eax. The call is `mov rcx,[rax]; mov r8,[rcx+30h]; ...; mov rcx,rax; xor r9d,r9d; mov rax,r8; call [guard]`. Recipe 0 is the call in the text-width function (0x15632c0, after FontHandle::getFont and the std::string to string_view setup). Recipe 1 is the same call in a second function (0x435c770, after the same setup). Each starts at its own preceding code (the setup differs in registers and offsets) and reads the displacement as a slot. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60): the slot is still 6. The call sites changed around the call: FontHandle::getFont is inlined, the Font pointer is held in rax and rcx differently (`mov rax,[rcx]; mov rax,[rax+30h]` instead of `mov rcx,[rax]; mov r8,[rcx+30h]`), and the string_view is stored from other registers. The two old patterns started at code that no longer exists, so both are now a pattern_any with one alternative per build; the 26.60 alternatives start with wildcard padding so that the offset of the displacement is the same as in the 26.51 alternative. Class: real code change (inlined callee, other registers). Cold run result: failed (both recipes no match). What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): in the first function the call before the text-cache loads is followed by a nop (so recipe 0 gains alternative 2, the 26.51 shape with the nop; alternatives 0 and 1 got one leading wildcard so that all three use offset 41); the second function (the line-length caller with the clamp `cmp eax,29h`) keeps its shape but holds the string in rbx instead of r15, so recipe 1 gains alternative 2. The slot is still 6 on every build. Both functions are the functions of the old recipes: same prologue and the same body (the first caches the width of a text and stores it as a float, the second clamps the width at 28h). Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? E8 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 83 BE ?? ?? ?? ?? ?? 72 ?? 48 8B BE ?? ?? ?? ?? 48 89 7D ?? 48 89 4D ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 C1 45 31 C9 4C 89 C0 FF 15",
        "?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 89 7D ?? 48 89 45 ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 45 31 C9 FF 15",
        "E8 ?? ?? ?? ?? 90 48 8B 8E ?? ?? ?? ?? 48 83 BE ?? ?? ?? ?? ?? 72 ?? 48 8B BE ?? ?? ?? ?? 48 89 7D ?? 48 89 4D ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 C1 45 31 C9 4C 89 C0 FF 15",
      ]
      offset = 41
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "49 8B 4F ?? 48 85 C9 74 ?? 49 83 7F ?? ?? 72 ?? 4D 8B 3F 4C 89 7D ?? 48 89 4D ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 C1 45 31 C9 4C 89 C0 FF 15",
        "?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 4C 89 4D ?? 48 89 55 ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 45 31 C9 FF 15",
        "48 8B 4B ?? 48 85 C9 74 ?? 48 83 7B ?? ?? 72 ?? 48 8B 1B 48 89 5D ?? 48 89 4D ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 C1 45 31 C9 4C 89 C0 FF 15 ?? ?? ?? ??",
      ]
      offset = 30
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 20
  }

}

entry "BitmapFont.vtable" {
  kind     = "global"
  notes    = "The primary vtable of BitmapFont (the Font subclass that draws the glyph_XX pages; the exe has no RTTI name for it, the class name is from the font code). 0xF0EA8A0 on 26.60. Slot 0 is the destructor (0x15aece0), slot 29 is _getCharWidth (BitmapFont._getCharWidth). Three functions load it (`inspect xrefs`): two constructors (0x158aa80 and 0x158b4f0 on 26.60, called from the font repository builders 0xec8800 and 0xec61d0) and the destructor. Recipe 0 is the first constructor: the base constructor call, then the vtable store and the start of the path copy (`lea rdx,[rbp+..]; mov rcx,r14; call; nop; lea rax,[rip+..]; mov [r14],rax; mov r12,[rdi]; mov rbx,[rdi+..]; mov r13,[rdi+..]; xorps xmm0,xmm0; movups [r14+..],xmm0; movups [r14+..]`). Recipe 1 is the second constructor (the copy constructor shape: `mov rdi,rdx; mov rsi,rcx; call; mov rcx,rdi; lea rax,[rip+..]; mov [rsi],rax; xorps xmm0,xmm0; movups [rsi+..],xmm0; mov [rbp+..],rsi; movups [rsi+..]`). Both read the RIP target of the lea. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The section check keeps the value in .rdata. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8D 55 ?? 4C 89 F1 E8 ?? ?? ?? ?? 90 48 8D 05 ?? ?? ?? ?? 49 89 06 4C 8B 27 48 8B 5F ?? 4C 8B 6F ?? 0F 57 C0 41 0F 11 86 ?? ?? ?? ?? 41 0F 11 86"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 D7 48 89 CE E8 ?? ?? ?? ?? 48 89 F9 48 8D 05 ?? ?? ?? ?? 48 89 06 0F 57 C0 0F 11 86 ?? ?? ?? ?? 48 89 75 ?? 0F 11 86"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "BitmapFont._getCharWidth" {
  kind     = "function"
  hooked   = true
  notes    = "float BitmapFont::_getCharWidth(int character, bool forceUnicode): rcx = the font, edx = the character (a code point), r8b = forceUnicode; the width is returned in xmm0. Slot 29 of BitmapFont.vtable (0x15902c0 on 26.60); nothing calls it directly (`inspect callers` gives 0, the only reference is the vtable slot). The body saves r8d in ebx and rcx in rsi, calls the glyph lookup (0xe6c330) with r9d = forceUnicode, tests bl, and calls its own vtable slots 33 ([rax+108h]) and 34 ([rax+110h]). Recipe 0 is the body after the lookup (`mov edi,eax; test bl,bl; je; mov bpl,imm; mov edx,[rsp+..]; jmp; mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; call [guard]; movzx eax,al; mov edx,[rsp+..]; mov bpl,imm; cmp edi,eax; jge; mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; mov r12d,edx; call [guard]`), then function_start_of. Recipe 1 is the head after the pushes (`mov ebx,r8d; mov rsi,rcx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov r15,imm64; mov r14,imm64; lea r8,[rsp+..]; mov r9d,ebx; call`; the two imm64 are the FNV-1a offset basis and prime, wildcarded), then function_start_of. Both recipes are in the same function and share no bytes, so a rewrite of the function breaks both. The in_vtable check is the independent fact: the value must be in slots 25..33 of BitmapFont.vtable (29 on 26.60). Every displacement, immediate and branch byte is a wildcard. The recipes were written on 26.60 only; 26.51 was not checked. What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): register allocation (the glyph id is held in ebp, the flag in r12b, the font in rdi) and one added `mov esi,edx` before the font move in the head; the body is otherwise the same function (same FNV constants, same two vtable calls, same glyph map member offsets). Recipe 0 now wildcards the register bytes (ModRM and REX of the moves and compares), and recipe 1 starts at the stack cookie load instead of the first two moves. Class: would have survived with a better recipe. Argument check on 6aed1906-b127418e74d2f2f9 and 6afa4e47-dadc375797a3d31a (the `signature` probe reports `+ rdx` on 70.26): the decompile of both functions has the same three parameters (this, character, forceUnicode), the same call of the glyph lookup with (this, character, a pointer to the out value, forceUnicode), the same FNV-1a constants and the same glyph map member offsets; the extra read of edx is the saved `mov esi,edx` before that call, not a new argument, so the C++ signature is unchanged and no selector entry is needed."

  recipe {
    step "pattern" {
      bytes = "89 ?? 84 DB 74 ?? ?? ?? ?? 8B 54 24 ?? E9 ?? ?? ?? ?? 48 8B ?? 48 8B 80 ?? ?? ?? ?? 48 89 ?? FF 15 ?? ?? ?? ?? 0F B6 C0 8B 54 24 ?? ?? ?? ?? 39 ?? 0F 8D ?? ?? ?? ?? 48 8B ?? 48 8B 80 ?? ?? ?? ?? 48 89 ?? 41 89 ?? FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 49 BF ?? ?? ?? ?? ?? ?? ?? ?? 49 BE ?? ?? ?? ?? ?? ?? ?? ?? 4C 8D 44 24 ?? 41 89 D9 E8"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "BitmapFont.vtable"
    min_slot = 25
    max_slot = 33
  }
}
