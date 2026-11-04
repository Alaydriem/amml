entry "ResourceLocation.ctor" {
  kind     = "function"
  notes    = "`ResourceLocation::ResourceLocation(Core::PathView path)`, the one-argument constructor: 0x3946b0 on 26.60, 0x2709a0 on 26.51 (1183 direct callers on 26.60). rcx = the 38h-byte ResourceLocation to build, rdx = a pointer to the 16-byte PathView {char const* data; uint64 size}; it returns rcx. The file system is fixed: `mov dword ptr [rcx],0` (ResourceFileSystem::UserPackage). Layout it writes: +0 mFileSystem, +8 mPath (std::string), +28h mPathHash (FNV-1a 64 of the C string, 0 when empty), +30h mFullHash ((fs ^ 0CBF29CE484222325h) * 100000001B3h ^ mPathHash). Recipe 0 is the body after the pushes (`mov dword ptr [rcx],imm; mov r14,[rdx]; mov rbx,[rdx+8]; xorps xmm0,xmm0; movups [rcx+18h],xmm0; movups [rcx+8],xmm0; test rbx,rbx; js`), then function_start_of. Recipe 1 is the call in HudLocatorBarRenderer.render that turns a waypoint's texture path into a ResourceLocation (`lea rcx,[rbp+..]; mov [rbp+..],rcx; mov [rbp+..],rax; lea r12,[rbp+..]; mov rcx,r12; lea rdx,[rbp+..]; call`), read as a branch target. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. There is no destructor entry: the header marks it MCFOLD, and render frees mPath inline as a std::string."

  recipe {
    step "pattern" { bytes = "C7 01 ?? ?? ?? ?? 4C 8B 32 48 8B 5A ?? 0F 57 C0 0F 11 41 ?? 0F 11 41 ?? 48 85 DB 0F 88" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 4D ?? 48 89 8D ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 4C 8D A5 ?? ?? ?? ?? 4C 89 E1 48 8D 95 ?? ?? ?? ?? E8"
      offset = 35
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ResourceLocation.size" {
  kind     = "field"
  notes    = "sizeof(ResourceLocation): 38h on 26.60 (+0 mFileSystem, +8 mPath std::string, +28h mPathHash, +30h mFullHash). It is the size of the buffer the one-argument constructor builds into, and the size check of the ResourceLocation mirror of a mod. One recipe: the end of ResourceLocation.ctor (0x3946b0 on 26.60), which stores the path hash, then folds the file system into the full hash and stores it (`mov [rsi+..],rcx; movzx eax,byte ptr [rsi]; xor rax,r14; imul rax,rbx; xor rax,rcx; mov [rsi+..],rax`), read as the displacement of the last store (mFullHash, the last member, 8 bytes), then `add 8`. The pattern is scanned in the constructor only. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "ref" { entry = "ResourceLocation.ctor" }
    step "pattern" {
      bytes  = "48 89 4E ?? 0F B6 06 4C 31 F0 48 0F AF C3 48 31 C8 48 89 46 ??"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = 8 }
  }

  check "range" {
    min = 32
    max = 128
  }
}
