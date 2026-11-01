entry "BiomeColorSampling.getMapPolicy" {
  kind     = "function"
  notes    = "BiomeColorSampling::MapPolicy const& BiomeColorSampling::getMapPolicy(TintMethod tint). A free function: cl = the tint, the policy reference in rax. The body returns `policies[tint < 9 ? tint : 0]` from a static array that it builds once: a plain policy for None, five BiomeAndPatternMapPolicy objects whose pattern comes from a helper (`mov ecx,n; call`, 6 calls) with the samplers default, birch, evergreen and dry foliage (pattern 1), grass (BiomeColorSampling.getMapGrassColor, pattern 3) and water (pattern 2), and two plain policies for Stem and RedStoneWire, in TintMethod order. Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `movzx ecx,byte ptr [r15+..]; call`, read as a branch target. Recipe 1: function B, then `mov r14,rax; test rax,rax; je; movzx ecx,byte ptr [r14+..]; call`, read as a branch target. They are in two functions. Check function_start. Evidence on 26.60: (1) the pattern helper, drafted from the old body (`dec ecx; cmp ecx,..; jae; mov eax,ecx; lea rcx,[rip+..]; mov rax,[rcx+rax*8]`), matches once on 26.60; as on 26.51 it has 7 call sites, 6 of them in one function, the candidate; (2) the candidate is called from 6 sites in 5 functions, as on 26.51, and from function A at the same place after the tryGetComponent call. The candidate body has the same shape (`cmp cl,9`, the static array, 6 pattern-helper calls); the allocation is now a virtual call. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus function_start."

  recipe {
    step "pattern" { bytes = "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15" }
    step "pattern" {
      bytes  = "41 0F B6 4F ?? E8"
      offset = 5
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 85 ?? ?? ?? ?? 48 8B 7F ?? 48 89 D9 E8", "89 45 ?? 48 8B 7F ?? 48 89 D9 E8"]
    }
    step "pattern" {
      bytes  = "49 89 C6 48 85 C0 0F 84 ?? ?? ?? ?? 41 0F B6 4E ?? E8"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BiomeColorSampling.getWaterColor" {
  kind     = "function"
  notes    = "int BiomeColorSampling::getWaterColor(Biome const& biome, BlockPos const&) (client only). A free function: rcx = biome, rdx = pos (not used), the ARGB colour in eax. The body finds a biome component in the biome component vector (or uses a default colour), clamps its four floats to 0..1, scales them by 255 and packs a << 24 | r << 16 | g << 8 | b. On 26.51 it does this per float with scalar code; on 26.60 with one movups, maxps/minps, mulps, cvttps2dq and a pmuludq/paddd pack. Recipe 0 is a static table of nine sampler pointers indexed by TintMethod (getWhite for None, the four foliage samplers, the tessellation grass sampler, this function for Water = 6, getWhite for Stem and RedStoneWire); a client tint function indexes it. The recipe is a pattern_any with one alternative per shape, both 13 bytes before the `lea rdx,[rip+table]`: `mov rcx,[r8]; test rcx,rcx; je; cmp al,9; movzx eax,al` (26.51, then `lea rax,[rdx+rax*8]; cmovae`) and `xor edx,edx; cmp al,9; movzx eax,al; cmovae eax,edx; movzx eax,al` (26.60, then `mov rax,[rdx+rax*8]`). It reads the table address and the pointer at index 6 (+30h). The index is the TintMethod value, which the header fixes. Recipe 1 is a body anchor, a pattern_any with one alternative per shape: the scalar packing tail (`shl ecx,18h; add ecx,r8d; shl edx,10h; shl eax,8; add eax,edx; add eax,ecx; add rsp; pop rsi; ret`, 26.51) or the default-colour select and clamp (`mov rax,[rdx+..]; lea rcx,[rax+..]; test rax,rax; lea rax,[rip+..]; cmovne rax,rcx; movups xmm0,[rax]; xorps xmm1,xmm1; maxps xmm1,xmm0`, 26.60), then function_start_of. The recipes share no anchor bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Evidence on 26.60: (1) the client tint function is found from the old function by its prologue (`push r15..rbx; sub rsp; mov rsi,rdx; mov rbx,rcx; mov rcx,[rcx+..]; test rcx,rcx; je`, drafted from the old body, one match on 26.60); its table at index 6 holds the candidate, and indexes 0, 7 and 8 again hold one shared stub, as on 26.51; (2) the candidate body does what the old body does: the component lookup with a default colour, clamp of four floats to 0..1, scale, pack into one int (read from the disassembly of both). What changed in 6ad79ee4-17d51891a1119914 (26.60): the body was vectorised and the tint function now selects index 0 with cmovae on the index, not on the address. Class: real code change. Grade: two recipes plus function_start."

  recipe {
    step "pattern_any" {
      patterns = ["49 8B 08 48 85 C9 74 ?? 3C ?? 0F B6 C0 48 8D 15 ?? ?? ?? ?? 48 8D 04 C2", "31 D2 3C ?? 0F B6 C0 0F 43 C2 0F B6 C0 48 8D 15 ?? ?? ?? ?? 48 8B 04 C2"]
      offset   = 13
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "add" { value = 48 }
    step "read_pointer" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["C1 E1 ?? 44 01 C1 C1 E2 ?? C1 E0 ?? 01 D0 01 C8 48 83 C4 ?? 5E C3", "48 8B 42 ?? 48 8D 48 ?? 48 85 C0 48 8D 05 ?? ?? ?? ?? 48 0F 45 C1 0F 10 00 0F 57 C9 0F 5F C8"]
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "BiomeColorSampling.getMapGrassColor" {
  kind     = "function"
  notes    = "int BiomeColorSampling::getMapGrassColor(Biome const& biome, BlockPos const& pos). A free function: rcx = biome, rdx = pos, the RGB colour in eax. The body uses a map grass colour biome component when there is one; otherwise it computes the colour from the biome temperature and downfall and, for the swamp-style component, from a noise sample at the position (two fixed colours 4C763Ch / 6A7039h). Recipe 0 is getMapPolicy (BiomeColorSampling.getMapPolicy), which builds the grass policy with this function as its sampler, after the pattern-3 helper call: `lea rdx,[rip+..]; lea rdi,[rbp+..]; mov rcx,rdi; mov r8,rax; call` (it starts after the helper call, so the nop after that call does not matter); the recipe reads the lea. The grass policy is the fifth sampler policy, which is TintMethod::Grass in the array order. Recipe 1 is the end of the body: `cmova eax,ecx; jmp; mov eax,imm32; movaps xmm6..xmm10,[rbp+..]` (the fixed-colour branch and the xmm restores), then function_start_of. The recipes share no anchor bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Evidence on 26.60: (1) getMapPolicy (repaired with its own evidence) builds its fifth sampler policy after `mov ecx,3; call` with the candidate as the sampler, as on 26.51; (2) two patterns drafted from the old body match once on 26.60, both in the candidate: the end of the body (recipe 1) and the component search (`cmp rax,rdx; je; movzx r8d,word ptr [rip+..]; nop; cmp [rax],r8w; je`). What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching. Recipe 0 started before the helper call. The old recipe 1 started at a call in the noise loop. Class: would have survived with a better recipe. Grade: two recipes plus function_start."

  recipe {
    step "pattern" { bytes = "48 8D 15 ?? ?? ?? ?? 48 8D 7D ?? 48 89 F9 49 89 C0 E8" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" { bytes = "0F 47 C1 EB ?? B8 ?? ?? ?? ?? 0F 28 75 ?? 0F 28 7D ?? 44 0F 28 45 ?? 44 0F 28 4D ?? 44 0F 28 55 ??" }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "BiomeColorSampling.getTessellationPolicy" {
  kind      = "function"
  msvc_name = "?getTessellationPolicy@BiomeColorSampling@@YAAEBVTessellationPolicy@1@W4TintMethod@@@Z"
  notes     = "BiomeColorSampling::TessellationPolicy const& BiomeColorSampling::getTessellationPolicy(TintMethod): a free function, cl = the tint method, the policy reference in rax. It returns `policies[tint < 9 ? tint : 0]` from a static array that it builds once (a plain policy for None, then the sampler policies), the same shape as BiomeColorSampling.getMapPolicy; it sits next to BiomeColorSampling.getWaterColor. The 27 callers in the block tessellator pass `movzx ecx,byte ptr [blockType+17Ch]` (BlockType.mTintMethod). Recipe 0 is the array read (`cmp bl,9; movzx eax,bl; lea rcx,[rip+..]; lea rax,[rcx+rax*8]; cmovae rax,rcx; mov rax,[rax]; add rsp,..; pop rbx; pop rsi; pop rbp`), then function_start_of. Recipe 1 is a call site in the block tessellator: `cmp qword ptr [rcx+8],0; je; mov rax,[r15+68h]; movzx ecx,byte ptr [rax+..]; call`. Check function_start. Recipe 0 is now a pattern_any, one alternative per shape of the index code: alternative 0 (26.51) `cmp bl,9; movzx eax,bl; lea rcx,[rip+array]; lea rax,[rcx+rax*8]; cmovae rax,rcx; mov rax,[rax]; add rsp; pop x3`; alternative 1 (26.60) `xor eax,eax; cmp cl,9; movzx ecx,cl; cmovae ecx,eax; movzx eax,cl; lea rcx,[rip+array]; mov rax,[rcx+rax*8]; add rsp; pop x3`. The limit 9 is a wildcard. What changed in 6ad79ee4-17d51891a1119914 (26.60): the compiler clamps the index before the load instead of the address after it, so alternative 0 did not match. The function is the same (the guarded static array of 9 policies, then the clamped load); recipe 1 (the tessellator call site with the BlockType.mTintMethod read) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "80 FB ?? 0F B6 C3 48 8D 0D ?? ?? ?? ?? 48 8D 04 C1 48 0F 43 C1 48 8B 00 48 81 C4 ?? ?? ?? ?? 5B 5E 5D",
        "31 C0 80 F9 ?? 0F B6 C9 0F 43 C8 0F B6 C1 48 8D 0D ?? ?? ?? ?? 48 8B 04 C1 48 81 C4 ?? ?? ?? ?? 5B 5E 5D",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 79 ?? ?? 74 ?? 49 8B 47 ?? 0F B6 88 ?? ?? ?? ?? E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
