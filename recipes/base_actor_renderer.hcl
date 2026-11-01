entry "BaseActorRenderer.ctor" {
  kind     = "function"
  notes    = "BaseActorRenderer::BaseActorRenderer(), the base constructor: rcx = the object, the constructor returns it in rax. It zeroes the 48 bytes of ActorShaderManager at +8, stores the BaseActorRenderer vtable at +0, and then builds eight MaterialPtr members (16 bytes each, from +38h to +0B7h) from the names `name_tag`, `name_tag_with_backface`, `name_tag_text`, `name_tag_text_with_backface`, `name_tag_depth_tested`, `name_tag_depth_tested_with_backface`, `name_text_depth_tested` and `name_text_depth_tested_with_backface`. The names are built inline (immediate stores into a std::string) and passed with a hash to the material lookup (0x52f1d30). The derived renderers call it first (`mov rsi,rcx; call ctor; lea rax,..; mov [rsi],rax`). Recipe 0 is the string xref of `name_text_depth_tested`: the only function that loads that format string. Recipe 1 is the prologue and the first stores (`push rbp; push rsi; push rdi; sub rsp,imm; lea rbp,[rsp+80h]; mov qword ptr [rbp+..],-2; xorps xmm0,xmm0; movups [rcx+..],xmm0 x3; lea rax,[rip+..]; mov [rcx],rax`), then function_start_of. They are in the same function by construction (a string and a body anchor), the only independent facts are the string and the code shape. In 6ad79ee4-17d51891a1119914 (26.60) the constructor is at 0x57E2AC0 with the same body (the eight materials at +38h to +0A8h in the same order); both recipes derived on the cold run. Class: survived."

  recipe {
    step "string_xref" {
      text = "name_text_depth_tested"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 0F 57 C0 0F 11 41 ?? 0F 11 41 ?? 0F 11 41 ?? 48 8D 05 ?? ?? ?? ?? 48 89 01"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

}

entry "BaseActorRenderer.size" {
  kind     = "field"
  notes    = "sizeof(BaseActorRenderer): 0xB8 (184) in 26.51 (ActorShaderManager 0x38, then eight MaterialPtr of 16 bytes). The derived renderers start their own members right after it. Recipe 0 is a derived constructor (0x55a43d0): `push rbp; push rsi; sub rsp,imm; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; mov rsi,rcx; call BaseActorRenderer::ctor; lea rax,[rip+..]; mov [rsi],rax; lea rcx,[rsi+0B8h]`; the lea takes the address of the first derived member, which sits at the end of the base. Recipe 1 is the base constructor: the last material is stored through `lea rcx,[rax+0A8h]` (the eighth MaterialPtr, 16 bytes), so the size is that displacement plus 16. They are in two functions. The 26.51 header also gives the eighth material as the last member. The size is the number the mod passes to the game heap before it calls the constructor, and the size check of the BaseActorRenderer mirror of the mod (the depth-tested materials at 0x78 and 0x98 are read from the 26.51 header layout, not from entries). In 6ad79ee4-17d51891a1119914 (26.60) the value is still 0xB8: the constructor stores the eight materials at the same offsets. Both recipes derived on the cold run. Class: survived."

  recipe {
    step "pattern" {
      bytes  = "55 56 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE E8 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 06 48 8D 8E ?? ?? ?? ??"
      offset = 47
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" {
      entry = "BaseActorRenderer.ctor"
    }
    step "pattern" {
      bytes  = "48 8D 88 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 4C 8D 45 ?? E8"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" {
      value = 16
    }
  }

  check "range" {
    min = 128
    max = 256
  }

}

entry "BaseActorRenderer._extractRenderTextObject" {
  kind     = "function"
  notes    = "static NameTagRenderObject BaseActorRenderer::_extractRenderTextObject(Tessellator&, std::string const&, std::vector<int> const&, Vec3 const&, mce::Color const&, MaterialPtr const*, MaterialPtr const*, mce::Color const&, float, optional<Quaternion> const&, float): a static function that returns a class, so the 144-byte result buffer is the FIRST argument (rcx) and every other argument moves one register to the right: rdx = the Tessellator&, r8 = the string, r9 = the widths vector, then on the stack from [rsp+20h]: the Vec3 pointer, the text colour pointer, the tag material pointer, the text material pointer, the tag colour pointer, the scale (a float, in the same 8-byte slot), the rotation pointer and the line gap. The two call sites show it: `lea rcx,[rbp-20h]; mov rdx,rdi; mov r8,[rbp+88h]; call` after the stack stores in BaseActorRenderer::extractRenderTextObjects (the tag and text material pointers are `lea r8,[rdx+78h]` or `+38h`, `lea rcx,[rax+98h]` or `+58h`: a pointer into the renderer), and `lea rcx,[rbp-8]; lea r9,[rbp+130h]; mov rdx,[rbp+118h]; mov r8,[rbp+228h]; call` in ActorRenderDispatcher::extractRenderTextObjects. The body starts with `mov r15,r9; mov rbx,r8; mov r14,rdx; mov rsi,rcx; mov rcx,r8; mov dl,0Ah; call` (it counts the newlines of the string). Recipe 0 is the call in BaseActorRenderer::extractRenderTextObjects (0x1ed3410), read as a branch target. Recipe 1 is the call in ActorRenderDispatcher::extractRenderTextObjects (0x1d55a60). Recipe 2 is the body head, then function_start_of. They are in three functions. Every register, displacement, immediate and branch byte is a wildcard. In 6ad79ee4-17d51891a1119914 (26.60) the function is at 0x57E3B10; both callers pass the same arguments in the same places (a `nop` follows each call); the three recipes derived on the cold run. Class: survived."

  recipe {
    step "pattern" {
      bytes  = "4C 89 54 24 ?? 48 8D 4D ?? 48 89 FA 4C 8B 85 ?? ?? ?? ?? E8"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 48 8D 4D ?? 4C 8D 8D ?? ?? ?? ?? 48 8B 95 ?? ?? ?? ?? 4C 8B 85 ?? ?? ?? ?? E8"
      offset = 41
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 89 CF 4C 89 C3 49 89 D6 48 89 CE 4C 89 C1 B2 ?? E8"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

}
