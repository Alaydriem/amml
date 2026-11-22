entry "Block.canProvideFullSupport" {
  kind     = "function"
  notes    = "bool Block::canProvideFullSupport(uchar face) const, a non-virtual member: rcx = Block this, dl = the face (1 is up), the bool is returned in al. The body fetches a support component of the block (`mov rsi,rcx; call getComponent; test rax,rax; je`), returns false when the component value is 0, calls the component support function with the face when the value is 1, and otherwise forwards to BlockType::canProvideFullSupport(Block const&, uchar) with `mov rcx,[rsi+68h]; mov rdx,rsi; mov r8d,ebx` (the block type, the block and the face) as a tail jump. Block::canProvideSupport(uchar, BlockSupportType), the function before it in the image, has the same shape but a type argument and calls the BlockType virtual at slot 18 instead. Recipe 0 is the head of the body: the face saved in ebx, the component call, the null test and the compare of the value against 1, then function_start_of. Recipe 1 is one of the two call sites, in a function that tests a flags word and then calls with `mov rcx,rsi; mov dl,1` (the face up): the target of the call is the function. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard. The check is function_start, which Recipe 1 can fail. In 6ad79ee4-17d51891a1119914 (26.60) the function is at 0x31533B0 with the same body (the component fetch, the 0/1 test and the tail jump to the BlockType function); both recipes derived on the cold run, with no change. Class: survived. Layout gate: 26.63 only; on 70.x the function takes a third argument (a bool in r8b; it is Block.canProvideFullSupportWithFlag with the flag 0), and the old two-argument form is gone; the replacement is Block.canProvideFullSupportWithFlag. The old call site of recipe 1 now loads `xor r8d,r8d` before the call."

  recipe {
    step "pattern" {
      bytes = "89 D3 48 89 CE E8 ?? ?? ?? ?? 48 85 C0 74 ?? 8B 00 85 C0 74 ?? 83 F8 ?? 75 ?? 48 89 F1 89 DA"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F7 81 ?? ?? ?? ?? ?? ?? ?? ?? 74 ?? 31 C0 EB ?? B0 ?? EB ?? 48 89 F1 B2 ?? E8 ?? ?? ?? ?? 48 8B 4C 24"
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Block.canProvideFullSupportWithFlag" {
  kind     = "function"
  notes    = "bool Block::canProvideFullSupport(uchar face, bool flag) const on 70.x, a non-virtual member: rcx = Block this, dl = the face (1 is up), r8b = a flag, the bool is returned in al. It replaces Block.canProvideFullSupport (26.63, two arguments), which has no out-of-line form on 70.x. Called with the flag 0 it does what the 26.63 function did: it fetches the support component of the block (two storage lookups, the block, then the block type at [block+68h] + 30h), returns false when the component value is 0, tail-jumps to the component support function with (block, face) when the value is 1, and without a component calls Block::canProvideSupport(face, type) with the type 0 and then 1 and returns the AND of both, which is the body of the 26.63 BlockType::canProvideFullSupport(Block const&, uchar) that the old function tail-jumped to (BlockType virtual canProvideSupport at 90h with r9d 0, then 1). A component value other than 0 and 1 (new on 70.x) reads an index at [component+4] and calls into a per-type table at [blocktype+368h]. With the flag set and no 0/1 component the function calls the BlockType virtual at 98h with (block type, block, face) instead. A mod that called canProvideFullSupport(face) calls this entry with (block, face, 0). Recipe 0 is the no-component fallback in the body: `mov rcx,rsi; mov edx,ebx; xor r8d,r8d; call; mov ecx,eax; xor eax,eax; test cl,cl; je; mov rcx,rsi; mov edx,ebx; mov r8d,imm; call` (the two canProvideSupport calls), then function_start_of; the type immediate and the call and branch bytes are wildcards. Recipe 1 is the call site that the 26.63 recipe 1 of Block.canProvideFullSupport used, in a function that tests a flags word and then calls with `mov rcx,rsi; mov dl,1`, now with `xor r8d,r8d` before the call: the target of the call. They are in two functions. Every displacement, immediate and branch byte is a wildcard. Check function_start, which recipe 1 can fail. Evidence from the 26.63 function: (1) its only call site of the flags-test shape (26.63 recipe 1, unique on 26.63) is found once on 70.26 with the added `xor r8d,r8d`, and it calls this function; (2) the 26.63 body (component fetch, return false on 0, tail jump with (block, face) on 1, else the BlockType full-support path that ANDs canProvideSupport with type 0 and 1) is found in this body in the same order with the same argument registers (`mov rcx,rsi; mov edx,ebx` before the tail jump; `mov rdx,rsi; mov r8d,ebx` before the BlockType call). Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). What changed in 6afa4e47-dadc375797a3d31a (70.26): a third argument, the component lookup now an EnTT storage lookup, the BlockType fallback inlined. Class: real code change."

  recipe {
    step "pattern" {
      bytes = "48 89 F1 89 DA 45 31 C0 E8 ?? ?? ?? ?? 89 C1 31 C0 84 C9 74 ?? 48 89 F1 89 DA 41 B8 ?? ?? ?? ?? E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F7 81 ?? ?? ?? ?? ?? ?? ?? ?? 74 ?? 31 C0 EB ?? B0 ?? EB ?? 48 89 F1 B2 ?? 45 31 C0 E8"
      offset = 28
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "Block.tryGetComponent.BlockMapColorComponent" {
  kind     = "function"
  notes    = "BlockMapColorComponent const* Block::tryGetComponent<BlockMapColorComponent>() const, the template instance. Signature: rcx = block, the result in rax (null when the block has no map colour component). On 26.51 the body searches the component index vectors inline; on 26.60 it calls a lookup helper twice (block, then block type) and returns the pointer + 8. Its callers are the map-colour path: call this function, test the result, read `byte ptr [result+10h]` (mTintMethod) into ecx, call BiomeColorSampling.getMapPolicy, call MapPolicy.get and multiply the colour at result+0 by the tint; without the component they call BlockType.getMapColor. Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rbx,[rbx+..]; mov rcx,rax; call`, read as a branch target. Recipe 1: function B, then `mov rdi,[rdi+..]; mov rcx,rbx; call` (the end of the anchor), read as a branch target. They are in two functions. Check function_start. Evidence on 26.60: (1) the anchor of function A, drafted from the old function A, matches once on 26.60, and the call at the same place (before the getMapPolicy call) goes to the candidate; (2) the old function had 8 call sites in 6 functions, each followed by the getMapPolicy call; the candidate has 8 call sites in 7 functions, and every getMapPolicy call site on 26.60 follows a call to it, also in function B. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus function_start. Layout gate: 26.63 only; on 70.x the template instance is inlined into Block::getMapColor (the only caller of getMapPolicy), which does the two storage lookups through a lookup that takes the storage pointer; the replacement is Block.getMapColor (the whole map colour) and BlockComponentStorage.tryGetComponent.BlockMapColorComponent (the lookup)."

  recipe {
    step "pattern" { bytes = "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15" }
    step "pattern" {
      bytes  = "48 8B 5B ?? 48 89 C1 E8"
      offset = 7
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
      bytes  = "48 8B 7F ?? 48 89 D9 E8"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Block.getMapColor" {
  kind     = "function"
  notes    = "mce::Color Block::getMapColor(BlockSource const& region, BlockPos const& pos) const on 70.x, a non-virtual member: rcx = Block this, rdx = the 16-byte colour out (returned in rax), r8 = the region, r9 = the position. It is the map-colour path that 26.63 inlined at each use of Block.tryGetComponent.BlockMapColorComponent: it looks up the BlockMapColorComponent in the block's component storage (block + 8), then in the block type's (the block type at [block+68h], + 30h); with a component it calls BiomeColorSampling.getMapPolicy with the tint byte at [component+10h], calls the policy's get (vtable +8) with (policy, a local colour, region, pos) and multiplies the component colour at [component+0] by it (movups/mulps); without one it calls the BlockType virtual at 408h with (block type, out, region, pos, block) (BlockType::getMapColor). A mod that read the component and tinted the colour itself can call this one function instead: rcx = block, rdx = a 16-byte colour (four floats), r8 = region, r9 = pos. Recipe 0 is the tint call in the body, `mov rcx,[rax]; mov r10,[rcx+8]; lea rdx,[rsp+..]; mov rcx,rax; mov r8,rbx; mov r9,rdi; mov rax,r10; call [guard]`, then function_start_of; the displacements are wildcards. Recipe 1 is a call site in a map renderer function: `cvttss2si eax,xmm0; mov [rbp+..],eax; mov r8,[rbx+..]; lea rdx,[rbp+..]; lea r9,[rbp+..]; mov rcx,r14; call`, read as a branch target; every displacement and the call bytes are wildcards. They are in two functions. Check function_start, which recipe 1 can fail. Evidence from the 26.63 function Block.tryGetComponent.BlockMapColorComponent: (1) its body (the lookup on block + 8, and when that gives null the lookup on [block+68h] + 30h, each returning the entry + 8) is the head of this body with the same offsets (`add rcx,8; call L; test rax,rax; jne; mov rcx,[r14+68h]; add rcx,30h; call L`, and L returns `lea rcx,[rax+8]` or null); (2) every 26.63 call site of it is followed by `movzx ecx,byte ptr [rax+10h]; call BiomeColorSampling.getMapPolicy; mov rcx,[rax]; mov r10,[rcx+8]; lea rdx; mov rcx,rax; mov r8; mov r9; mov rax,r10; call [guard]` (old function A), and on 70.26 this function holds the same sequence and is the only caller of BiomeColorSampling.getMapPolicy (a resolved entry). Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). What changed in 6afa4e47-dadc375797a3d31a (70.26): the map-colour code is one out-of-line function and the component lookup is inlined into it. Class: real code change."

  recipe {
    step "pattern" {
      bytes = "48 8B 08 4C 8B 51 ?? 48 8D 54 24 ?? 48 89 C1 49 89 D8 49 89 F9 4C 89 D0 FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F3 0F 2C C0 89 45 ?? 4C 8B 43 ?? 48 8D 55 ?? 4C 8D 4D ?? 4C 89 F1 E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "BlockComponentStorage.tryGetComponent.BlockMapColorComponent" {
  kind     = "function"
  notes    = "BlockMapColorComponent const* BlockComponentStorage::tryGetComponent<BlockMapColorComponent>() const on 70.x: rcx = a component storage (block + 8 for the block's own components, or the block type at [block+68h] + 30h for the type's), the component pointer or null in rax (the found entry + 8). Block::tryGetComponent<BlockMapColorComponent> of 26.63 is this lookup on block + 8 and, when it gives null, on [block+68h] + 30h; a mod that needs the component itself does the same two calls. Block.getMapColor is the simpler replacement for a colour. Recipe 0 is the first call in Block.getMapColor (`mov r14,rcx; <stack cookie>; add rcx,8; call`), recipe 1 the second (`mov rcx,[r14+68h]; add rcx,30h; call`), each read as a branch target; both scan only that function, so they share its anchor, and the check function_start is the strong check. Every displacement, immediate and branch byte is a wildcard. Evidence from the 26.63 function Block.tryGetComponent.BlockMapColorComponent: (1) its two lookups (block + 8, then [block+68h] + 30h, each `test; je` and the entry + 8) are the two calls of this function in Block.getMapColor in the same order and with the same storage offsets; (2) the body of this function ends with the same type-index load (a guarded static word passed by address) and the `lea rcx,[rax+8]; test rax,rax; cmovne` result that the 26.63 body has after each lookup call. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). What changed in 6afa4e47-dadc375797a3d31a (70.26): the block-level template instance is inlined; the storage-level lookup is out of line. Class: real code change."

  recipe {
    step "ref" { entry = "Block.getMapColor" }
    step "pattern" {
      bytes  = "49 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 83 C1 ?? E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "ref" { entry = "Block.getMapColor" }
    step "pattern" {
      bytes  = "49 8B 4E ?? 48 83 C1 ?? E8"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "Block.mDirectData.mLightEmission" {
  kind  = "field"
  notes = "Offset, from the start of the Block, of BlockComponentDirectData::mLightEmission (a 1-byte Brightness, 0..15, the light level the block emits), inside Block::mDirectData. LeviLamina headers world/level/block/Block.h and components/BlockComponentDirectData.h: the vtable (8), mComponents (60h), mBlockType at 68h, mCachedComponentData (8) at 70h, mDirectData at 78h; in mDirectData five component pointers (28h), four bools, then mLightEmission at +2Ch and mLight (the light the block absorbs) at +2Dh, so A4h and A5h from the Block. The game reads it from the Block, not through BlockType. Recipe 0 is the light spread core (the function that LightPropagationCoordinator.propagateForMesh calls first): for every block of a subchunk that is not air it loads the emission and the absorption, writes the absorption into a 4-bit-per-block array and, when the emission is not 0, pushes the block, its position and the emission byte (+0Bh of a 10h record) into the emitter list. It is a pattern_any with one alternative per shape, both starting at the emission load at offset 0, read as the displacement of operand 1. Alternative 0 (26.60, 26.62, 26.63): `movzx r9d,byte ptr [r14+..]; movzx eax,byte ptr [r14+..]; mov ecx,eax; or cl,r9b; je`; the fixed `mov ecx,eax; or cl,r9b` bytes tie the first load (r9b, the emitter byte) and the second (al, the absorption written to the array). Alternative 1 (Preview): the absorption, a new byte at +71h and the emission are loaded in that order, and the pattern starts at the third load: `movzx edx,byte ptr [rdi+..]; mov eax,ecx; mov [rbp+..],dl; or al,dl; setne al; or al,r9b; cmp al,..; jne`; the emission is kept in a frame slot and pushed into the emitter list from there. Recipe 1 is the point light function of Vibrant Visuals (26.62 0x790b9d0, the function just before PointLightCoordinator.getAnalyticalLights in the image): it loads the emission of the block before it asks the block for a component (`movzx r14d,byte ptr [rdi+..]; mov rcx,rdi; call`) and keeps it for the light it adds. Step 0 is a pattern_any that finds that function, one alternative per shape: `movaps xmm7,xmm8; movzx; mov; call` (26.60..26.63) and `xor r14d,r14d; movzx; mov; call` (Preview); step 1 scans only that function for `movzx r..,byte ptr [r..+disp32]; mov rcx,r..; call` (once in it on every build), offset 0, read as the displacement of operand 1. The two recipes are in two functions and share no pattern bytes; each pattern ends at a call or before one. Every displacement, immediate and branch byte is a wildcard. Range window 80h..100h. Values: A4h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63); A5h on 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23), where the members after it moved one byte (the absorption is at A6h) and the code also reads a new byte at +71h. On 6aa482fd-2658118eba0dbac5 (26.51) neither recipe matches and the entry refuses; 26.51 is out of scope. Evidence: (1) on 26.63 the emitter list of the spread core is filled from +A4h and the absorption array from +A5h; on the Preview the same loop (the first direct call of propagateForMesh, the same per-block virtual getBlock and air test) fills the emitter list from +A5h and the absorption array from +A6h; (2) the point light function, found on the Preview by the same body after the load (`call; nop; movaps xmm0,[rip+..]; movss xmm8,[rip+..]; test rax,rax; je; mov eax,[rax+10h]; cmp eax,2; je; cmp eax,71h; jne`), reads +A4h on 26.62 and +A5h on the Preview. Also on 26.60..26.63: LevelChunk::_lightingCallbacks (26.63 0x384e460) compares the old and the new block at +A4h and +A5h before it calls the lighting update, and Item::getLightEmission(int) (Item vtable slot 121 on 26.63, 0x3f976b0) returns the byte at +A4h of the item's block state. What changed in 6ae4eeee-4dd4fa7acacb4c41: the member moved one byte and the spread loop has a new shape; the meaning is the same."

  recipe {
    step "pattern_any" {
      patterns = [
        "45 0F B6 ?? ?? ?? ?? ?? 41 0F B6 ?? ?? ?? ?? ?? 89 C1 44 08 C9 74 ??",
        "0F B6 ?? ?? ?? ?? ?? 89 C8 88 95 ?? ?? ?? ?? 08 D0 0F 95 C0 44 08 C8 3C ?? 75 ??",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "41 0F 28 F8 44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8",
        "45 31 F6 44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8",
      ]
    }
    step "pattern" { bytes = "44 0F B6 ?? ?? ?? ?? ?? 48 89 ?? E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}

entry "Block.getRenderLayer" {
  kind     = "function"
  notes    = "BlockRenderLayer Block::getRenderLayer() const, the static form with no position. No exported name; the name follows Block::getRenderLayer(BlockSource&, BlockPos const&) const (at 0x6A3A3C0 on 26.63), which has the same body but ends in a tail call to BlockType::getRenderLayer (vslot 123). Signature: rcx = Block this, the layer (a BlockRenderLayer, one byte) returned in al. The body fetches the material instances component of the block (`push rsi; sub rsp,..; mov rsi,rcx; call Block::tryGetComponent; mov rcx,[rsi+68h]; test rax,rax; je`). When the block has the component and its block type's default state is not in the BlockGraphics static map that follows the getForBlock map, the result is the render layer of the first material instance (5 when the material map is empty); otherwise it reads BlockType::mRenderLayer directly (`movzx eax, byte ptr [rcx+162h]; ret`). BlockGraphics::mRenderLayer is not read on any layer path, so this function replaces it for the layer of a ghost. Callers: the block tessellator neighbour occlusion test (0x680A070, which treats layers 5, 9 and 17 as occluders) twice, and 0x530D094; the same code is inlined in BlockTessellator::tessellateInWorld for the extra block. Recipe 0 is the head of the body, then function_start_of (the pattern starts at the function start). Recipe 1 is the call site in the occlusion test: `mov r14,rax; mov rcx,rsi; call X; mov ecx,[r14+10h]; cmp al,11h; ja`, read as a branch target. They are in two functions. Every displacement, immediate and branch byte is a wildcard. Check function_start. Value 0x6A3A2D0 in 6aed1906-b127418e74d2f2f9 (26.63). Class: new entry."

  recipe {
    step "pattern" { bytes = "56 48 83 EC ?? 48 89 CE E8 ?? ?? ?? ?? 48 8B 4E ?? 48 85 C0 0F 84" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 C6 48 89 F1 E8 ?? ?? ?? ?? 41 8B 4E ?? 3C ?? 0F 87"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Block.mBlockType" {
  kind  = "field"
  notes = "Offset in Block of mBlockType, the pointer to the BlockType of the block state: 68h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26), the value of the amml-game BlockLayout mirror. Recipe 0 is the call site of BlockType.getCollisionShape recipe 0: `mov rcx,[reg+disp8]` (the block type of the block) `; mov rax,[rcx]; mov rax,[rax+..]; mov [rsp+..],r8; mov qword ptr [rsp+..],0; lea rdx,[rsp+..]; mov r8,r10; call [guard]`, offset 0, the displacement of operand 1. Recipe 1 is BlockSource::getMaterial(BlockPos const&) (BlockType.mMaterialInline recipe 0, the by-value shape that both builds have): `sub rsp,..; mov rax,[rcx]; mov rax,[rax+..]; call [guard]` (getBlock) `; mov rax,[rax+disp8]; add rax,..; add rsp,..; ret`, offset 17, the displacement of operand 1. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Range window 40h..100h. Evidence on both builds: Block.getRenderLayer, Block.canProvideFullSupport (26.63), Block.canProvideFullSupportWithFlag and Block.getMapColor (70.26) all load the block type from Block+68h before a BlockType member or virtual. Class: new entry."

  recipe {
    step "pattern" {
      bytes = "48 8B ?? ?? 48 8B 01 48 8B 40 ?? 4C 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 54 24 ?? 4D 89 D0 FF 15"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B 40 ?? 48 83 C0 ?? 48 83 C4 ?? C3"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 256
  }
}

entry "Block.mSerializationId" {
  kind  = "field"
  notes = "Offset in Block of mSerializationId, the CompoundTag the game builds once for every block state: `name` (the full name of the block type), `version` (an int) and `states` (a compound of the state values). F0h on 6aed1906-b127418e74d2f2f9 (26.63), F8h on 6afa4e47-dadc375797a3d31a (70.26). The tag is the CompoundTag object itself (vtable at +0, the std::map at +8, its size at +10h). The loader reads the `version` int of this tag (through the CompoundTag map mirror) to build a block tag that BlockSerializationUtils.tryGetBlockFromNBT reads with no upgrade. How it was found: on 26.63 only two functions load both the pooled `states` and `version` texts; one is tryGetBlockFromNBT, the other (0x3169970) is Block::buildSerializationId(uint version): it clears the map at Block+F8h, puts `name` (from BlockType+E8h) and `version` (the edx argument) into the tag at Block+F0h, then a `states` compound. Its one caller (0x35a6ad0) makes every Block of a block type (the Block constructor, then this function with the same version), and that caller's one caller (0x31f3890) passes one version for every block type of the registry. On 70.26 `inspect similar` ranks 0x5615d00 first (the same three strings); it holds the same `name` and `version` puts into the tag at Block+F8h (map at +100h). Recipe 0 is the `version` put in that function: `mov qword ptr [rbp+..],imm; mov qword ptr [rbp+..],imm (7, the key length); mov qword ptr [rbp+..],imm;mov eax,[rip+..]; mov [rbp+..],eax; mov eax,[rip+..]; mov [rbp+..],eax; mov byte ptr [rbp+..],imm; lea rdx,[rbp+..]; mov rcx,rdi; mov r8d,ebx; call` (unique on each build), then `lea rdi,[rsi+disp32]` (the tag; once in the function), the displacement of operand 1. Recipe 1 is the Block constructor (26.63 0x3165910, 70.26 0x56121a0; the one caller of recipe 0's caller allocates 128h bytes and calls it with the data index and the BlockType): `mov [rcx+..],r8; test r8,r8; je; mov si,dx; lea rax,[rcx+..]; mov [rbp+..],rax; mov word ptr [rcx+..],imm` (unique on each build), then the store of the CompoundTag vtable into the tag, `lea rax,[rip+..]; mov [rcx+disp32],rax; mov [rbp+..],rcx; movups [rcx+..],xmm0` (once in the function; the lea target is CompoundTag.vtable on both builds), offset 7, the displacement of operand 0. The Block destructor (26.63 0x31700f0, 70.26 0x5641390) stores the same vtable at the same offset. The two recipes are in two functions and share no bytes. Every other displacement, immediate and branch byte is a wildcard. Range window C0h..180h. What changed in 70.26: the members before the tag grew by 8 bytes (a real layout change). Class: new entry."

  recipe {
    step "pattern" { bytes = "48 C7 45 ?? ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 8B 05 ?? ?? ?? ?? 89 45 ?? 8B 05 ?? ?? ?? ?? 89 45 ?? C6 45 ?? ?? 48 8D 55 ?? 48 89 F9 41 89 D8 E8" }
    step "pattern" { bytes = "48 8D BE ?? ?? ?? ??" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" { bytes = "4C 89 41 ?? 4D 85 C0 0F 84 ?? ?? ?? ?? 66 89 D6 48 8D 41 ?? 48 89 45 ?? 66 C7 41 ?? ?? ??" }
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 81 ?? ?? ?? ?? 48 89 4D ?? 0F 11 81"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 192
    max = 384
  }
}
