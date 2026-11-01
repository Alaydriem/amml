entry "Tessellator.begin" {
  kind      = "function"
  msvc_name = "?begin@Tessellator@@QEAAXUDebugContextCallback@1@W4PrimitiveMode@mce@@H_N@Z"
  notes     = "void Tessellator::begin(DebugContextCallback, mce::PrimitiveMode, int reserved, bool): rcx = this, dl = the empty DebugContextCallback (not read), r8b = mode, r9d = reserved, [rsp+20h] at the call = the bool. The body returns at once when this+2A0h (mTessellating, header 672) or this+255h is set, then zeroes this+298h (mCount, header 664), 255h, 2A0h and 1CAh and calls MeshData::clear on this+8 (mMeshData). BaseActorRenderer._extractRenderTextObject calls it before its vertex loop with r8b = 1. Recipe 0 is the body after the two guards (`mov r32,r9d|r8d; movzx r32,byte ptr [rsp+..]; mov dword ptr [this+..],0; mov byte ptr [this+..],0` x2), then function_start_of. The register bytes of the mov, the movzx and the three stores are wildcards. Recipe 1 is that call site, `mov byte ptr [rsp+20h],0; mov rcx,r14; mov r8b,1; call`, inside BaseActorRenderer._extractRenderTextObject, read as a branch target. Every displacement and immediate byte is a wildcard. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): `this` moved from rcx to rsi before the second guard, and the body saves r8d as well as r9d before the movzx, so the stores use rsi and recipe 0 with fixed register bytes did not match. The body is the same (guards 2A0h and 255h, stores 298h, 255h, 2A0h, 1CAh, then MeshData::clear on this+8). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "44 89 ?? 0F B6 ?? 24 ?? ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "BaseActorRenderer._extractRenderTextObject" }
    step "pattern" {
      bytes  = "C6 44 24 ?? ?? 4C 89 F1 41 B0 ?? E8"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Tessellator.end" {
  kind      = "function"
  msvc_name = "?end@Tessellator@@QEAA?AVMesh@mce@@W4UploadMode@1@V?$basic_string_view@DU?$char_traits@D@std@@@std@@W4SupplementaryFieldAutoGenerationMode@@@Z"
  notes     = "mce::Mesh Tessellator::end(UploadMode, std::string_view debugName, SupplementaryFieldAutoGenerationMode): rcx = this, rdx = the 632-byte Mesh buffer (hidden return pointer), r8d = upload mode, r9 = pointer to the string_view, [rsp+20h] = the 2-byte mode. BaseActorRenderer._extractRenderTextObject passes the \"Untagged Tessellator Mesh\" view (length 19h), then moves the result into a make_shared<Mesh> block of 288h bytes (632 + 16) with Mesh.moveCtor and destroys the local with Mesh.dtor. The body checks this+255h and the positions vector (this+10h/18h, MeshData+8), then resets mCount (298h) and the flags as clear does. Recipe 0 is the head of that body: `mov r64,rdx` (keeps the result pointer), the two guards `cmp byte ptr [rcx+2A0h],0; je` and `cmp byte ptr [rcx+255h],1; je` with `mov r64,rcx` between them, then function_start_of. The REX and ModRM bytes of both movs, every displacement, immediate and branch byte are wildcards. Recipe 1 is the call site in BaseActorRenderer._extractRenderTextObject: `lea r9,[rbp+..]; mov rcx,r14; xor r8d,r8d; call`. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): `this` goes to rsi (r12 on 26.51) and the vertex count test after the guards is a 32-bit `imul` instead of the 64-bit multiply, so the 26.51 body pattern (which ran into that test) did not match. The two guards are the same. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 ?? 80 B9 ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? ?? 89 ?? 80 B9 ?? ?? ?? ?? ?? 0F 84" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "BaseActorRenderer._extractRenderTextObject" }
    step "pattern" {
      bytes  = "4C 8D 8D ?? ?? ?? ?? 4C 89 F1 45 31 C0 E8"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Tessellator.clear" {
  kind      = "function"
  msvc_name = "?clear@Tessellator@@QEAAXXZ"
  notes     = "void Tessellator::clear(): rcx = this. The body zeroes this+298h (mCount), 255h, 2A0h (mTessellating) and 1CAh, calls MeshData::clear on this+8, then clears the other state bytes (+0, +170h, 1B4h, 188h, ..). begin inlines the same stores but uses `lea rax,[rcx+8]` and checks the guards first. It has one caller. Recipe 0 starts at the first byte (`push rsi; sub rsp,..; mov rsi,rcx`, then the four stores and `add rcx,8; call`), so the value is the match start and function_start is a real check. Recipe 1 is its only call site, in a function that calls Tessellator.begin, Tessellator.end, then clear (`mov rcx,r14; call`) and Mesh.dtor right after it. It is one pattern_any with one alternative per shape, both with the clear call at offset 11 and both running past calls: alternative 0 (26.51) `mov rcx,rbx; call; mov rcx,r14; call; lea rcx,[rbp+..]; call`; alternative 1 (26.60) `nop; lea rsi,[rbp+..]; mov rcx,r14; call; nop; lea rcx,[rbp+..]; call`. There is no other route, because the tail stores are the same in a sibling function and inlined in 17 others. Each alternative matches once on its own build and not on the other. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): the compiler adds a nop after the call before clear and after clear, and loads a frame address into rsi between them, so the 26.51 call-site pattern did not match. The new call site was accepted on two facts from the 26.51 caller: it is still the only direct call to clear (callers on both builds), and the caller has the same call order (begin, a vertex call, end, clear, Mesh.dtor). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "56 48 83 EC ?? 48 89 CE C7 81 ?? ?? ?? ?? ?? ?? ?? ?? C6 81 ?? ?? ?? ?? ?? C6 81 ?? ?? ?? ?? ?? C6 81 ?? ?? ?? ?? ?? 48 83 C1 ?? E8" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 D9 E8 ?? ?? ?? ?? 4C 89 F1 E8 ?? ?? ?? ?? 48 8D 4D ?? E8",
        "90 48 8D B5 ?? ?? ?? ?? 4C 89 F1 E8 ?? ?? ?? ?? 90 48 8D 4D ?? E8",
      ]
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Tessellator.mTessellating" {
  kind     = "field"
  notes    = "Offset of bool Tessellator::mTessellating: 2A0h on 26.51 (header 672; Tessellator has no vtable, mMeshData at 8, mCount at 298h), the anchor of TessellatorLayout. Tessellator.begin returns at once when it is set (`cmp byte ptr [rcx+2A0h],0; jne`) and clear and begin set it to 0. The recipe reads the third zero store in Tessellator.begin (`movzx r32,byte ptr [rsp+..]; mov dword ptr [this+298h],0; mov byte ptr [this+255h],0; mov byte ptr [this+2A0h],0`, offset 25 to the 2A0h store); registers, displacements and immediates are wildcards. Range window 290h..2B0h. Value 2A0h on 26.51 and 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): begin moves `this` to rsi between its two guards, so the old pattern on the two guard compares did not match; the member did not move (the mirror check passed: the guard and the store both use 2A0h on 26.60). Would have survived with a better recipe."

  recipe {
    step "ref" { entry = "Tessellator.begin" }
    step "pattern" {
      bytes  = "0F B6 ?? 24 ?? ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ?? C6 ?? ?? ?? ?? ?? ??"
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 656
    max = 688
  }
}
