entry "MeshData.clear" {
  kind     = "function"
  notes    = "void mce::MeshData::clear(): rcx = this. A leaf without .pdata on both builds. It resets mMode, the bounds and the field flags of a MeshData whose vectors are already empty (it does not free them). The 26.51 MeshData constructor is `memset(this+8, 0, 148h)` and then this call; on 26.60 that constructor is not a function any more (it is inlined into the Tessellator constructor, and of the 288 direct callers of clear on 26.60 the only small one is Tessellator.clear), so the mod builds an empty MeshData itself: zeroed bytes, then this call. Tessellator.clear calls it on this+8 (`add rcx,8; call`), and Mesh.moveCtor calls it on its MeshData after the memset (`xor edx,edx; call memset; mov [rbp+..],r64; mov rcx,r64; call`). Recipe 0 is the call in Tessellator.clear, read as a branch target. Recipe 1 is the call in Mesh.moveCtor (offset 14; it runs past the memset call, with no nop on either build), read as a branch target; the REX and ModRM bytes of the store and the mov are wildcards. Check leaf_start. This entry replaces MeshData.ctor, which does not exist on 6ad79ee4-17d51891a1119914 (26.60): real code change."

  recipe {
    step "ref" { entry = "Tessellator.clear" }
    step "pattern" {
      bytes  = "48 83 C1 ?? E8"
      offset = 4
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "ref" { entry = "Mesh.moveCtor" }
    step "pattern" {
      bytes  = "31 D2 E8 ?? ?? ?? ?? ?? 89 ?? ?? ?? 89 ?? E8"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}

entry "MeshData.dtor" {
  kind      = "function"
  msvc_name = "??1MeshData@mce@@QEAA@XZ"
  notes     = "mce::MeshData::~MeshData(): rcx = this. It frees the 13 vectors from the last one (mGeoType at +110h, capacity end at +120h) down to mPositions (+8). Mesh.dtor calls it on Mesh+58h. Recipe 0 is that call (`lea rcx,[rsi+..]; call`) inside Mesh.dtor. Recipe 1 is the reset of a unique_ptr<MeshData> in the region tessellation function (the one that calls BlockTessellator.setRegion and tessellateInWorld): `mov rdi,[rbx+..]; mov [rbx+..],rsi; test rdi,rdi; je; mov rcx,rdi; call`, followed by the sized delete of 160h bytes (352, the MeshData size), then `lea rcx,[rbp+..]; call` of the same destructor on a local MeshData. The pattern runs past the first two calls (no nop on either build) to tell this reset from a unique_ptr reset of another 0E8h-byte type that has the same head on 26.60; the delete size is a wildcard. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): a second function got the same reset head, so recipe 1 had 2 matches; the longer pattern matches once on each build. Would have survived with a better recipe."

  recipe {
    step "ref" { entry = "Mesh.dtor" }
    step "pattern" {
      bytes  = "48 8D 4E ?? E8"
      offset = 4
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B BB ?? ?? ?? ?? 48 89 B3 ?? ?? ?? ?? 48 85 FF 74 ?? 48 89 F9 E8 ?? ?? ?? ?? BA ?? ?? ?? ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "MeshData.moveAssign" {
  kind      = "function"
  msvc_name = "??4MeshData@mce@@QEAAAEAV01@$$QEAV01@@Z"
  notes     = "mce::MeshData& mce::MeshData::operator=(MeshData&&): rcx = this, rdx = the source. It copies mMode (the first byte), returns at once on self-assignment, then for each vector frees its own block and takes the source pointers. Mesh's move assignment calls it on +58h; a LevelRenderer helper builds an empty MeshData with MeshData.ctor and moves it in (`call ctor; add rsi,8; mov rcx,rsi; mov rdx,rdi; call`). Recipe 0 is the body tail: the last bound store, the move of the two trailing qwords (+150h, +157h), `mov rax,rsi; movaps xmm6,[rsp+..]; add rsp,..`, then function_start_of. Every displacement and immediate is a wildcard. Recipe 1 is the call in Mesh's move assignment: `mov eax,[rsi+..]; mov [rdi+..],eax; lea rdx,[rsi+..]; lea rcx,[rdi+..]; call` (MeshData at +58h on 26.51, +50h on 26.60). Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): another move assignment (of a larger type with the same first vectors) has the same body head, so the 26.51 head pattern (`mov rdi,rdx; mov rsi,rcx; movzx eax,byte ptr [rdx]; mov [rcx],al; cmp rcx,rdx; je; ..`) had 2 matches. The tail with the +150h/+157h moves is in this function only. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "F3 0F 11 86 ?? ?? ?? ?? 48 8B 87 ?? ?? ?? ?? 48 8B 8F ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 89 8E ?? ?? ?? ?? 48 89 F0 0F 28 74 24 ?? 48 83 C4 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "8B 46 ?? 89 47 ?? 48 8D 56 ?? 48 8D 4F ?? E8"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
