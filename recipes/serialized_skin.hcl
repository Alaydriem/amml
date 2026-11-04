entry "SerializedSkinImpl.mSkinImage" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mSkinImage (SkinImage, an mce::Image) in the object that a SerializedSkinRef's shared_ptr element points to: 80h on 26.60, A0h on 26.51 (26.60 removed std::string mPlayFabId after mId). SerializedSkinImpl sits at +0 of Bedrock::Application::ThreadOwner<SerializedSkinImpl>: every read below is relative to the element pointer, and the offsets match the header layout counted from 0 (mDefaultGeometryName 60h, mGeometryData F8h on 26.60). mce::Image is the same on both builds: +0 format, +4 width, +8 height, +0Ch depth, +10h usage, +18h the mce::Blob (deleter +18h, data pointer +20h, byte size +28h). Found in the function that returns the byte size of a skin (0x298f4e0 on 26.60, 0x2650180 on 26.51; called by the skin memory check of the PlayerListPacket and PlayerSkinPacket handlers): it adds the cape image size, the skin image size and the size of each animated image (`mov rcx,[rcx]; mov rbx,[rcx+capeSize]; mov rax,[rcx+..]; add rbx,[rcx+skinSize]; mov rdx,[rcx+..]; cmp rax,rdx; je; xchg ax,ax; add rbx,[rax+30h]; add rax,40h; cmp rax,rdx; jne`). One recipe: that pattern from `mov rdi,rcx`, read as the displacement of `add rbx,[rcx+..]` (the skin image byte size: A8h on 26.60, C8h on 26.51), then `add -28h` (Image.mImageBytes 18h + Blob.mSize 10h) to reach the image start. Every displacement, immediate and branch byte is a wildcard, the two-byte nop included. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 09 48 8B 99 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 03 99 ?? ?? ?? ?? 48 8B 91 ?? ?? ?? ?? 48 39 D0 74 ?? ?? ?? 48 03 58 ?? 48 83 C0 ?? 48 39 D0 75"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -40 }
  }

  check "range" {
    min = 96
    max = 192
  }
}

entry "SerializedSkinImpl.mGeometryData" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mGeometryData (Json::Value, 16 bytes; the skin geometry, with the head bone UV of persona and custom skins) in the object that a SerializedSkinRef's shared_ptr element points to: F8h on 26.60, 118h on 26.51 (26.60 removed std::string mPlayFabId after mId). It matches the header layout counted from 0: mSkinImage 80h, mCapeImage B0h, mSkinAnimatedImages E0h (24 bytes), then mGeometryData F8h on 26.60. The validity test of a SerializedSkinRef (0x6bdee80 on 26.60) also reads a Json::Value at +F8h. Found in the skin size function (0x298f4e0 on 26.60, 0x2650180 on 26.51): after the animated image loop it serialises the geometry with JsonValue.toStyledString (`cmp rax,rdx; jne; add rcx,imm32; lea rdx,[rsp+..]; call`) to count its bytes. One recipe: that pattern, read as the immediate of the add. The same bytes match once on both builds. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "48 39 D0 75 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 54 24 ?? E8"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 192
    max = 320
  }
}

entry "SerializedSkinImpl.mResourcePatch" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mResourcePatch (std::string, the skin resource patch JSON) in the object that a SerializedSkinRef's shared_ptr element points to: 40h on 26.60 (mId 0h, mFullId 20h). The function that writes a skin to the connection request JSON (0x4b9b60 on 26.60, the only loader of the key strings `SkinResourcePatch`, `PremiumSkin` and `PersonaSkin`) reads the string through the skin reference (`mov rax,[r14+..]; mov rdx,[rax]; mov r8d,[rdx+..] (the size); cmp qword ptr [rdx+..],imm (the capacity); jb; mov rdx,[rdx+..]; jmp; add rdx,imm8`) before it stores it under `SkinResourcePatch`. One recipe: string_xref of `SkinResourcePatch`, then that pattern in the function, read as the immediate of the add (the inline buffer of the string is its start). Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "string_xref" { text = "SkinResourcePatch" }
    step "pattern" {
      bytes  = "49 8B 46 ?? 48 8B 10 44 8B 42 ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ?? EB ?? 48 83 C2 ??"
      offset = 24
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 32
    max = 128
  }
}

entry "SerializedSkinImpl.mDefaultGeometryName" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mDefaultGeometryName (std::string) in the object that a SerializedSkinRef's shared_ptr element points to: 60h on 26.60. The validity test of a SerializedSkinRef (0x6bdee80 on 26.60; the PlayerListPacket and PlayerSkinPacket handlers call it) takes the element, returns true at once when the name is empty, and otherwise checks the geometry data (`mov r13,[rcx]; mov al,imm; cmp qword ptr [r13+..],imm (the size of the name); je; lea rsi,[r13+..] (the name); lea rdi,[r13+..] (SerializedSkinImpl.mGeometryData); mov rcx,rdi; call`). One recipe: that pattern, read as the displacement of `lea rsi,[r13+..]`. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "4C 8B 29 B0 ?? 49 83 7D ?? ?? 0F 84 ?? ?? ?? ?? 49 8D 75 ?? 49 8D BD ?? ?? ?? ?? 48 89 F9 E8"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 160
  }
}

entry "SerializedSkinImpl.mSkinAnimatedImages" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mSkinAnimatedImages (std::vector<AnimatedImageData>: begin at +0, end at +8) in the object that a SerializedSkinRef's shared_ptr element points to: E0h on 26.60 (after mSkinImage 80h and mCapeImage B0h). Recipe 0 is the skin size function (0x298f4e0 on 26.60; the anchor of SerializedSkinImpl.mSkinImage): `mov rdi,rcx; mov rcx,[rcx]; mov rbx,[rcx+..]; mov rax,[rcx+..] (begin); add rbx,[rcx+..]; mov rdx,[rcx+..] (end); cmp rax,rdx; je; ...; add rbx,[rax+..]; add rax,imm8; cmp rax,rdx; jne`, read as the displacement of the load of the begin. Recipe 1 is the function that writes a skin to the connection request JSON (0x4b9b60 on 26.60), which walks the vector to write `AnimatedImageData` (`mov rax,[r14+..]; mov rax,[rax]; mov rsi,[rax+..] (begin); mov r12,[rax+..] (end)`), read as the displacement of the load of the begin. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 09 48 8B 99 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 03 99 ?? ?? ?? ?? 48 8B 91 ?? ?? ?? ?? 48 39 D0 74 ?? ?? ?? 48 03 58 ?? 48 83 C0 ?? 48 39 D0 75"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 46 ?? 48 8B 00 48 8B B0 ?? ?? ?? ?? 4C 8B A0 ?? ?? ?? ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 160
    max = 320
  }
}

entry "SerializedSkinImpl.mIsPremium" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mIsPremium (a bool) in the object that a SerializedSkinRef's shared_ptr element points to: 1E9h on 26.60 (mIsTrustedSkin 1E8h, mIsPremium 1E9h, mIsPersona 1EAh). The function that writes a skin to the connection request JSON (0x4b9b60 on 26.60) writes three flags in a row from the same element (`mov rax,[rbx+..]; mov rax,[rax]`): `TrustedSkin` (`cmp byte ptr [rax+..],imm; sete dl`, the trusted flag is an enum compared with 2), then `PremiumSkin` and `PersonaSkin` (`movzx edx,byte ptr [rax+..]`); the key strings follow the values, which the rip targets of the key leas show. The two bool blocks have the same bytes as the blocks after them, so the recipe anchors on the unique trusted compare and runs through the TrustedSkin block to the first movzx. One recipe: string_xref of `PremiumSkin`, then that pattern in the function, read as the displacement of the movzx. The pattern runs across calls with their rel32 and the nops after them wildcarded. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "string_xref" { text = "PremiumSkin" }
    step "pattern" {
      bytes  = "80 B8 ?? ?? ?? ?? ?? 0F 94 C2 48 8D 4D ?? E8 ?? ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 8B 7D ?? 48 89 F9 E8 ?? ?? ?? ?? ?? 48 8D 75 ?? 48 89 C1 48 89 F2 E8 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8B 43 ?? 48 8B 00 0F B6 90 ?? ?? ?? ??"
      offset = 70
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 768
  }
}

entry "SerializedSkinImpl.mIsPersona" {
  kind     = "field"
  notes    = "The offset of SerializedSkinImpl::mIsPersona (a bool) in the object that a SerializedSkinRef's shared_ptr element points to: 1EAh on 26.60, the byte after mIsPremium. Found in the same three-flag run of the connection request JSON writer (0x4b9b60 on 26.60) as SerializedSkinImpl.mIsPremium: the `PersonaSkin` value is the second movzx after the unique `TrustedSkin` compare. One recipe: string_xref of `PersonaSkin`, then the pattern from the trusted compare through the PremiumSkin block to the second movzx, read as its displacement. The pattern runs across calls with their rel32 and the nops after them wildcarded. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "string_xref" { text = "PersonaSkin" }
    step "pattern" {
      bytes  = "80 B8 ?? ?? ?? ?? ?? 0F 94 C2 48 8D 4D ?? E8 ?? ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 8B 7D ?? 48 89 F9 E8 ?? ?? ?? ?? ?? 48 8D 75 ?? 48 89 C1 48 89 F2 E8 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8B 43 ?? 48 8B 00 0F B6 90 ?? ?? ?? ?? 48 8D 4D ?? E8 ?? ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 89 F9 E8 ?? ?? ?? ?? ?? 48 89 C1 48 89 F2 E8 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8B 43 ?? 48 8B 00 0F B6 90 ?? ?? ?? ??"
      offset = 129
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 768
  }
}
