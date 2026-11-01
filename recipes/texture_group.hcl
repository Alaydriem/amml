entry "TextureGroup.getTexture" {
  kind     = "function"
  notes    = "`mce::TexturePtr mce::TextureGroup::getTexture(ResourceLocation const&, bool forceReload, std::optional<uint>, cg::TextureSetLayerType)`, the TextureGroupBase override: 0x541f580 on 26.60, 0x1d0cfd0 on 26.51 (159 direct callers on both). It is called on the TextureGroupBase subobject: rcx = TextureGroup+18h, rdx = the out TexturePtr (32 bytes), r8 = ResourceLocation const*, r9b = forceReload, [rsp+20h] = the optional<uint>, [rsp+28h] = the layer type. Recipe 0 is the call in HudLocatorBarRenderer.render (`add r14,18h; mov byte ptr [rsp+28h],imm; mov qword ptr [rsp+20h],imm; mov rcx,r14; lea rdx,[rbp+..]; mov r8,r12; xor r9d,r9d; call`). Recipe 1 is the call in the image upload helper (0x64db900 on 26.60, 0x5c28150 on 26.51) that calls TextureGroup.uploadTexture and then this function on the same ResourceLocation (`mov rcx,[rdi+..]; add rcx,18h; mov byte ptr [rsp+28h],imm; mov qword ptr [rsp+20h],imm; lea rdx,[rbp+..]; mov r8,rbx; xor r9d,r9d; call`). Both are read as branch targets. The two call sites are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. function_start confirms the start."

  recipe {
    step "pattern" {
      bytes  = "49 83 C6 ?? C6 44 24 ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 4C 89 F1 48 8D 95 ?? ?? ?? ?? 4D 89 E0 45 31 C9 E8"
      offset = 34
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 48 83 C1 ?? C6 44 24 ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 55 ?? 49 89 D8 45 31 C9 E8"
      offset = 32
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "TextureGroup.uploadTexture" {
  kind     = "function"
  notes    = "`BedrockTexture& mce::TextureGroup::uploadTexture(ResourceLocation const&, cg::ImageBuffer)`: 0x54204f0 on 26.60, 0x1d0df40 on 26.51. rcx = the TextureGroup itself (not the +18h subobject that getTexture takes; the body passes rcx on to the overload that reads the loaded-texture map at +1A8h, which getTexture reaches as +190h from +18h), rdx = ResourceLocation const*, r8 = a pointer to the caller's 30h-byte ImageBuffer temporary; rax returns the BedrockTexture&. The callee destroys the by-value ImageBuffer: at the end it calls the blob deleter ([r8]) on the data pointer ([r8+8]) when that is not null, so the caller must not free it again. The body wraps the buffer in a TextureContainer (`mov r8b,1; call`) and calls the uploadTexture(ResourceLocation const&, TextureContainer&&, string_view) overload with an empty string_view. Recipe 0 is that body after the prologue (`mov rsi,rdx; mov rdi,rcx; lea rax,[rip+..] (the empty string); mov [rbp+..],rax; mov qword ptr [rbp+..],imm; lea rcx,[rbp+..]; mov [rbp+..],r8; mov rdx,r8; mov r8b,imm; call`), then function_start_of. Recipe 1 is the call in the image upload helper (0x64db900 on 26.60, 0x5c28150 on 26.51: `mov rcx,rbx; mov rbx,[rbp+..]; mov rdx,rbx; mov r8,r14; call`), read as a branch target; that helper moves an ImageBuffer into the argument temporary (ImageBuffer move constructor) just before the call, and calls TextureGroup.getTexture right after it. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" { bytes = "48 89 D6 48 89 CF 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 4D ?? 4C 89 45 ?? 4C 89 C2 41 B0 ?? E8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 D9 48 8B 5D ?? 48 89 DA 4D 89 F0 E8"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "TextureGroup.unloadTexture" {
  kind     = "function"
  notes    = "`bool mce::TextureGroup::unloadTexture(ResourceLocation const&, bool assertOnFailure)`: 0x541d8b0 on 26.60, 0x1d0b340 on 26.51 (26 direct callers on 26.60). rcx = the TextureGroup itself (not +18h), rdx = ResourceLocation const*, r8b = assertOnFailure (the release body does not read it); al returns true when the location was loaded. The body looks the location up in the loaded-texture map at +1A8h, tells the LRU cache at +240h, and when found releases the BedrockTexture in the map node (node+58h) and erases the location from the map at +230h; the node in +1A8h stays. The name comes from the shape (two arguments, bool result, the maps of TextureGroup) and from the image upload helper that calls TextureGroup.uploadTexture and this function on the same location; there is no symbol. Recipe 0 is that call in the helper (0x64db900 on 26.60, 0x5c28150 on 26.51: `mov rcx,[rdi+..]; xor edi,edi; xor r8d,r8d; call`), read as a branch target. Recipe 1 is the body after the first lookup (`mov r15,[rbx]; cmp rsi,r15; je; lea r14,[rsi+..]; lea rcx,[rsi+..]; call`, the end test and the release of node+58h), then function_start_of. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 31 FF 45 31 C0 E8"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "4C 8B 3B 4C 39 FE 74 ?? 4C 8D 76 ?? 48 8D 4E ?? E8" }
    step "function_start_of" {}
  }

  check "function_start" {}
}
