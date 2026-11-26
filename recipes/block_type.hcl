entry "BlockType.mMaterialInline" {
  kind     = "field"
  notes    = "Which layout BlockType uses for its Material: 0 when it holds a `Material const&` stored as a pointer (26.51), 1 when it holds the Material by value (26.60, 6ad79ee4-17d51891a1119914). `amml-game` (MaterialLayout) reads it to select the BlockType mirror: value 0 selects the mirror with the Material pointer at 28h and the 26.51 Material (seven bool bytes, mSolid at 5); value 1 selects the mirror with the Material value at 28h (MaterialType at +0, the flag byte at +1, solid bit 10h). Any other value refuses. The mirror offsets are not entries: they do not move between the two builds. Both recipes are a pattern_any whose alternative 0 is the by-reference code shape and alternative 1 the by-value shape, followed by the `alternative` step, which gives the index of the alternative that matched. Recipe 0 is the body of BlockSource::getMaterial(BlockPos const&) (a virtual of the BlockSource table): `sub rsp,imm; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]` (getBlock) and then `mov rax,[rax+68h]; mov rax,[rax+28h]` (26.51: load the Material pointer) or `mov rax,[rax+68h]; add rax,28h` (26.60: the address of the inline Material), then `add rsp,imm; ret`. Recipe 1 is a function that tests the material type against a bit set after a virtual getBlock call (`call [guard]; mov rcx,[rax+68h]` then `mov rcx,[rcx+28h]; movzx ecx,byte ptr [rcx]` or `movzx ecx,byte ptr [rcx+28h]`, then `cmp ecx,12h; ja; bt ebp,ecx`). They are in two functions. Each alternative matches once in its own build and not at all in the other. Every displacement, immediate and branch byte is a wildcard. Evidence for the 26.60 layout: the getMaterial body returns the address BlockType+28h instead of the pointer stored there; 26.51 has 419 Block to BlockType to Material pointer chains and 26.60 none; the call sites of getMaterial read the solid flag at Material+5 in 26.51 (`cmp byte ptr [rax+5],imm`) and at Material+1 bit 10h in 26.60 (`test byte ptr [rax+1],10h`); each of the 10 sites in 26.51 that tests Block to BlockType to Material+5 has a twin at the same place of the same function in 26.60 that tests BlockType+29h with mask 10h. The check is range 0..1. Keep one alternative per value: `alternative` gives the index of the alternative, so a third alternative would give 2 and fail the range; widen an alternative with wildcards instead of adding one. New in 6ad79ee4-17d51891a1119914. Class: real code change (layout)."

  recipe {
    step "pattern_any" {
      patterns = ["48 83 EC ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B 40 ?? 48 8B 40 ?? 48 83 C4 ?? C3", "48 83 EC ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B 40 ?? 48 83 C0 ?? 48 83 C4 ?? C3"]
      offset   = 0
    }
    step "alternative" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["FF 15 ?? ?? ?? ?? 48 8B 48 ?? 48 8B 49 ?? 0F B6 09 83 F9 ?? 77 ?? 0F A3 CD", "FF 15 ?? ?? ?? ?? 48 8B 48 ?? 0F B6 49 ?? 83 F9 ?? 77 ?? 0F A3 CD"]
      offset   = 0
    }
    step "alternative" {}
  }

  check "range" {
    min = 0
    max = 1
  }
}

entry "BlockType.getCollisionShape" {
  kind     = "vslot"
  notes    = "Slot of AABB BlockType::getCollisionShape(Block const&, IConstBlockSource const&, BlockPos const&, optional_ref<GetCollisionShapeInterface const>) const in the BlockType vtable: 4 in 26.51 (LeviLamina header order: ~BlockType 0, getNextBlockPermutation 1, hasTag 2, clip 3, getCollisionShape 4; getMapColor sits at 128 by the same count, which the BlockType.getMapColor entry confirms). The AABB (24 bytes) is returned through a hidden pointer. The call site shows the registers: rcx = BlockType this (`mov rcx,[block+68h]`), rdx = the 24-byte result buffer (`lea rdx,[rsp+38h]`), r8 = Block const& (the block, copied from rcx), r9 = the region (copied from rdx), [rsp+20h] = BlockPos const& (copied from r8) and [rsp+28h] = the optional_ref, a null pointer (`mov qword ptr [rsp+28h],0`). The result is then read as six floats in the order min x, min y, min z, max x, max y, max z, which the compares and the `subss` of the call site show. Recipe 0 is the call in a Block function that tests whether the shape is a non-empty box, with the null pointer stored through `mov [rsp+20h],r8` and the block moved with `mov r8,r10`. Recipe 1 is the same test in a second function that keeps the block in r8 and the position in rsi. They are in two functions and each reads the displacement of the vtable call (`mov rax,[rax+20h]`) as a slot. Every displacement, immediate and branch byte is a wildcard; the patterns end at the call. The check is a window around the known slot; a vslot with a range check alone needs two agreeing recipes, which it has. In 6ad79ee4-17d51891a1119914 (26.60) the slot is still 4: the same two call sites match, with the same argument stores. Both recipes derived on the cold run. Class: survived."

  recipe {
    step "pattern" {
      bytes  = "48 8B ?? ?? 48 8B 01 48 8B 40 ?? 4C 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 54 24 ?? 4D 89 D0 FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B ?? ?? 48 8B 01 48 8B 40 ?? 48 89 74 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 54 24 ?? FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 12
  }
}

entry "BlockType.getMapColor" {
  kind     = "vslot"
  notes    = "Slot of BlockType::getMapColor (mce::Color getMapColor(BlockSource&, BlockPos const&, Block const&) const) in the BlockType vtable: 128 (80h) on 26.51, 129 (81h) on 26.60. When a block has no BlockMapColorComponent the map-colour functions call it on Block+68h (the block type): `mov rcx,[reg+68h]; mov rax,[rcx]; mov rax,[rax+disp]; mov r10,[rip+..]` (the guard), then `mov [rsp+20h],reg`. Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rcx,[r12+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r10,[rip+..]; mov [rsp+..],r12`, operand 1 displacement as a slot. Recipe 1: function B, then `mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r10,[rip+..]; mov [rsp+..],rbx`, the same. They are in two functions. Check range 110..150. Evidence on 26.60: the call on the block type in the no-component branch of function A and of function B (the same place as on 26.51, after the tryGetComponent test) both read 408h, so the slot moved by one (an earlier BlockType virtual was added). The old recipe 1 matched both twins on 26.60 and the derive refused; no wrong value. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus range. What changed in 6afa4e47-dadc375797a3d31a (26.70.26): the slot is still 129 (081h). The no-component branch and the component branch (the `mulps` of the biome colour) of functions A and B are no longer inline: one small function now holds both branches and is called from them (the function that called the map-colour call inline passes the block in rcx, the output in rdx, the region in r8 and the position in r9). Its call of the slot is `mov rcx,[reg+68h]; mov rax,[rcx]; mov rax,[rax+disp]; mov [rsp+20h],reg; mov rdx,..; mov r8,..; mov r9,..; call [guard]`. Both recipes are now one pattern_any each, with the cursor on the `mov rax,[rax+disp]` (offset 3 from `mov rax,[rcx]`). Alternative 0 of each is function A or B on the older builds, one contiguous run from the call through the code after it (the run is unique because of the code after the call; the earlier two-step form had a first step that still matched in an unrelated function of 26.70.26). Alternative 1 of recipe 0 is the call site with its argument moves, ending at the call; alternative 1 of recipe 1 keeps only the six opcode bytes of the call setup, wildcards the argument moves and keeps the code after the call (`jmp`, the stack cookie check). On 26.70.26 both land on the one call site in the shared function, so their independence is only the bytes around the call. Evidence on 26.70.26: (1) the colour-found branch of function A on 26.63 (`mov rcx,[rax]; mov r10,[rcx+8]; lea rdx; mov rcx,rax; mov r8,rbx; mov r9,..; mov rax,r10; call [guard]`) matches once in the shared function; the call on the block type in the no-component branch is the same call with the same four argument registers and the block on the stack, found once on 26.70.26 against six sites on 26.63; (2) the callers of the shared function are the functions that correspond to A and B (a size of 1830 against 1821 and 2783 against 2858 bytes, the call at the place where A and B had the inline test), and the vtable around it did not shift (BlockType.asItemInstance 4Eh and BlockType.getCollisionShape 4 are the same on both builds). Class: would have survived with a better recipe only in part; the inlined body became one function, so this is a real code change in shape."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 8B 15 ?? ?? ?? ?? 4C 89 64 24 ?? 48 8D 55 ?? 49 89 D8 4D 89 F1 41 FF D2 90 0F 28 45 ?? 0F 29 45 ?? 48 8B 1D ?? ?? ?? ?? 48 85 DB 75 ??",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 89 74 24 ?? 48 89 F2 49 89 D8 49 89 F9 FF 15",
      ]
      offset = 3
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
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 8B 15 ?? ?? ?? ?? 48 89 5C 24 ?? 48 8D 55 ?? 4C 8D 4D ?? 49 89 F8 41 FF D2 90 0F 28 45 ?? 0F 29 45 ?? 48 8B 3D ?? ?? ?? ?? 48 85 FF 75 ??",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? FF 15 ?? ?? ?? ?? EB ?? 48 8B 4C 24 ?? 48 31 E1 E8",
      ]
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 110
    max = 150
  }
}

entry "BlockType.canBeBuiltOver" {
  kind     = "vslot"
  notes    = "Slot of bool BlockType::canBeBuiltOver(Block const&, BlockSource const&, BlockPos const&) const, the overload that the easy place C++ calls (four arguments with this: rcx the type, rdx the block, r8 the region, r9 the position), in the BlockType vtable: 83 on 26.51. The LeviLamina header order puts the overload with the extra BlockType const& first (header 83/84 after the same shift as the known slots getCollisionShape 4, canProvideSupport 18 and getMapColor 128), but MSVC lays the pair out the other way: in the base table slot 84 reads a fifth argument (`mov rax,[rsp+60h]; test byte ptr [rax+130h],8`) and first calls slot 83 (`mov rax,[rcx]; mov rax,[rax+298h]; call [guard]`), so 84 is the five-argument overload and 83 is this one. Both recipes are calls through Block::mBlockType (`mov rcx,[block+68h]; mov rax,[rcx]; mov rax,[rax+disp]`) with the block in rdx, the region in r8 and the position in r9, followed by `test al,al`. Recipe 0 is in a BlockSource function (`mov rcx,[rdx+..]; ...; mov r8,rsi; mov r9,rbx; call [guard]`). Recipe 1 is in another function (`mov rax,[rax+disp]; lea r9,[rbp+..]; mov rdx,rdi; mov r8,r14; call [guard]`). Both read the displacement as a slot. The two recipes are in two functions. Every displacement and branch byte is a wildcard. Range window 75..91. Recipe 1 is now another site with the block in r12: `mov rcx,[r12+68h]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,r12; mov r8,rdi; mov r9,r14; call [guard]`, then `test al,al; je`. Value 83 on 26.51, 84 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): BlockType gained one virtual before this pair (asItemInstance 77 to 78, getMapColor 128 to 129, this 83 to 84); the 26.51 site of recipe 1 changed shape. The new site was found by drafting each 26.60 call of slot 84 and keeping one whose pattern matches once on each build and calls slot 83 on 26.51; recipe 0 (the BlockSource site) agrees on 84. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (26.70.26): the BlockSource function of recipe 0 now loads the block into another register and passes it to the call with an extra `mov rdx,reg` before `mov r8,rsi`, so recipe 0 is a pattern_any with one alternative per shape (alternative 0 the older shape, alternative 1 the one with the extra register move; each matches once on its own builds and zero times on the other). The function is the one that starts `mov ebp,r9d; mov rbx,r8; mov rdi,rdx; mov rsi,rcx` and compares a block id with 13Fh; the face-offset table code `cmp bpl,5; ja; movzx eax,bpl; lea rax,[rax+rax*2]` sits right before the call in both. Evidence on 26.70.26: the old body pattern drafted from that BlockSource function in 26.63 matches once in one function of 26.70.26, and the call of slot 84 in it is at the same place after the face-offset code; the second recipe (a call site in a different function) still matches once and gives the same slot. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 4A ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 49 89 F0 49 89 D9 FF 15",
        "?? 8B ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? ?? 89 ?? 49 89 F0 49 89 D9 FF 15",
      ]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 4C 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 89 E2 49 89 F8 4D 89 F1 FF 15"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 75
    max = 91
  }
}

entry "BlockType.isInteractiveBlock" {
  kind     = "vslot"
  notes    = "Slot of bool BlockType::isInteractiveBlock() const in the BlockType vtable: 119 on 26.51 (LeviLamina header 126 with the same shift of 7 as getMapColor 128). Of the 270 BlockType vtables that hold the named `BlockType::getMobToSpawn` (slot 86), 62 hold `mov al,1; ret` at slot 119 and the rest the shared `xor eax,eax; ret`; the slots next to it are overridden by 2 and 5 tables only. Both recipes test a BlockType flag byte and then call this slot on the type: `test byte ptr [rcx+..],..; jne; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 0 is in GameMode::useItemOn (GameMode slot 13), after `mov al,..`. Recipe 1 is in ClientInputCallbacks::handleBuildAction, after the type load `mov rcx,[rax+..]`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 111..127. Recipe 0 now starts at the block load (`mov rax,[rbp+..]; mov rcx,[rax+68h]; mov al,..; test; jne; ..`, offset 25), because a second function has the same flag test and call on 26.60. Recipe 1 is now inside ClientInputCallbacks.handleBuildAction (ref), the call after the flag test with the near jne (`test byte ptr [rcx+..],..; jne near; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`). Value 119 on 26.51, 120 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): BlockType gained one virtual before this slot; another function got the same test-and-call shape (recipe 0 had 2 matches, both slot 120), and in handleBuildAction the type load before the short-jne site changed. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (26.70.26): the slot is still 120 (078h). Recipe 0 starts at the flag test (`test byte ptr [rcx+..],..; jne short; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`) and is a pattern_any with one alternative per code after the call (alternative 0 stores al to a member, `nop; mov [rbx+..],al`, on the older builds; alternative 1 moves it to a register, `nop; mov r15d,eax`), offset 12 to the `mov rax,[rax+disp]`; the block load before the test is no longer part of the pattern because the lookups of components before it changed. The site is in GameMode::useItemOn, slot 13 of GameMode.vtable on both builds. Recipe 1 no longer takes ClientInputCallbacks.handleBuildAction as its scope: on 26.70.26 the test and call of the near-jne shape moved out of that function into another one, so the shape is now a pattern over the executable; it matches once on 26.63 and once on 26.70.26 (the older gate builds are checked by the derive). Evidence on 26.70.26: (1) the near-jne shape drafted from handleBuildAction on 26.63 matches once, with the flag byte at 17Dh and the slot at 3C0h of recipe 0; (2) slot 13 of the GameMode vtable holds the old useItemOn function on 26.63 and the function that holds the recipe 0 site on 26.70.26, and the call in it has the same flag test and the same slot. Class: would have survived with a better recipe (the scope of recipe 1 and the block load of recipe 0 were build-specific)."

  recipe {
    step "pattern_any" {
      patterns = [
        "F6 81 ?? ?? ?? ?? ?? 75 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 88 43 ??",
        "F6 81 ?? ?? ?? ?? ?? 75 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 41 89 C7",
      ]
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F6 81 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 111
    max = 127
  }
}

entry "BlockType.asItemInstance" {
  kind     = "vslot"
  notes    = "Slot of ItemInstance BlockType::asItemInstance(Block const&, BlockActor const*) const in the BlockType vtable: 77 on 26.51 (LeviLamina header 84 with the shift of 7). The 128-byte ItemInstance comes back through a hidden pointer: rcx the type, rdx the buffer, r8 the block, r9 the block actor. The base body saves the buffer (`mov rsi,rdx`) and builds the item from a type member. Both recipes are calls through Block::mBlockType (`mov rcx,[block+68h]; mov rax,[rcx]; mov rax,[rax+disp]`) with a null block actor (`xor r9d,r9d`). Recipe 0 passes the caller's own result buffer (`mov rdx,rsi; mov r8,rdi; xor r9d,r9d; call [guard]`). Recipe 1 passes a local buffer after the block arrives in r8 (`movups [rcx],xmm0; mov rcx,[r8+..]; ...; lea rdx,[rbp-..]; xor r9d,r9d; call [guard]`). Both read the displacement as a slot. The two recipes are in two functions. Every displacement and branch byte is a wildcard. Range window 69..85."

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 49 89 F8 45 31 C9 FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "0F 11 01 49 8B 48 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? 45 31 C9 FF 15"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 69
    max = 85
  }
}

entry "BlockType.mTintMethod" {
  kind     = "field"
  notes    = "Offset of TintMethod BlockType::mTintMethod: 17Ch on 26.51 (header 380, after the 20-bit bool run), the anchor of the tint read. The block tessellator reads it from the block type (Block+68h) and passes it to BiomeColorSampling.getTessellationPolicy at 27 sites. Recipe 0 is `cmp qword ptr [rcx+8],0; je; mov rax,[r15+68h]; movzx ecx,byte ptr [rax+disp]; call`; recipe 1 is another site in another function (after `movups xmm0,[rbx+..]; jmp; cmp qword ptr [rbx+8],0; je; mov rax,[rsi+68h]`). Both read the displacement. Range window 170h..188h."

  recipe {
    step "pattern" {
      bytes  = "48 83 79 ?? ?? 74 ?? 49 8B 47 ?? 0F B6 88 ?? ?? ?? ?? E8"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 10 83 ?? ?? ?? ?? EB ?? 48 83 7B ?? ?? 74 ?? 48 8B 46 ?? 0F B6 88 ?? ?? ?? ?? E8"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 368
    max = 392
  }
}

entry "BlockType.getExtraRenderLayers" {
  kind     = "vslot"
  notes    = "Slot of int BlockType::getExtraRenderLayers() const in the BlockType vtable: 124 on 26.63 (LeviLamina header index 124, right after getRenderLayer 123; the header and the binary agree here, as they do for isInteractiveBlock 120 and getMapColor 129). It returns a bit mask of BlockRenderLayer: the chunk builder puts the block into one more layer queue for each set bit 0..21, besides its main layer. Of the 265 BlockType vtables with the base getRenderLayer at slot 123, 261 hold the shared `xor eax,eax; ret` at slot 124, three return 0x20 (Opaque) and one returns 8 (Blend); the header names the overriders CauldronBlock, ChemicalHeatBlock, HoneyBlock and SlimeBlock. Both recipes load the block type of a block and call this slot: `mov rcx,[reg+68h]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 0 is in the RenderChunkBuilder block sort (0x7919B60): the call with the guard load `mov rdx,[rip+..]` and the frame stores before `call rdx; mov [rbp+..],eax`. Recipe 1 is in function 0x6A83A10: the call that is followed by `nop; mov r15d,eax; test eax,eax; je; xor ebx,ebx`, the start of a loop over the 22 layer bits (`cmp bl,16h`). Both read the displacement as a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 116..132. Value 124 in 6aed1906-b127418e74d2f2f9 (26.63). Class: new entry. What changed in 6afa4e47-dadc375797a3d31a (26.70.26): the slot is still 124 (07Ch). Both functions are still there, each with the same call. In the block sort (recipe 0) the compiler saves the guard-call return in one more register, `mov r10,[rbp+..]`, between `call rdx` and the store of the result, so recipe 0 is a pattern_any with one alternative per shape; both start at `mov rax,[rcx]` (offset 3 to the `mov rax,[rax+disp]`) and end with the store of the result. In the layer loop (recipe 1) the block register moved (`mov rcx,[reg+68h]` from rdx to another register) and the loop counter register changed (`xor ebx,ebx` to `xor esi,esi`), so the register bytes of the pattern are wildcards. Evidence on 26.70.26: (1) each of the two functions of 26.63 has its twin in 26.70.26 with the same two call sites (one function with two sites in the layer loop, one large function with one site in the block sort), the same size to within a few percent and the same code after the call (the guard call and the store of the result; `nop; mov reg,eax; test eax,eax; je`); (2) the two recipes land in two different functions and give the same slot, and the neighbouring BlockType slots of other entries did not move (BlockType.asItemInstance 4Eh, BlockType.getMapColor 81h, BlockType.isInteractiveBlock 78h are the same on both builds). Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 4C 8B 85 ?? ?? ?? ?? 4C 89 85 ?? ?? ?? ?? FF D2 89 85 ?? ?? ?? ?? 8B 95",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 4C 8B 85 ?? ?? ?? ?? 4C 89 85 ?? ?? ?? ?? FF D2 4C 8B 95 ?? ?? ?? ?? 89 85",
      ]
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "?? 8B ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 41 89 C7 85 C0 0F 84 ?? ?? ?? ?? 31 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 116
    max = 132
  }
}

entry "BlockType.mNameInfo.mFullName" {
  kind  = "field"
  notes = "Offset in BlockType of mNameInfo.mFullName, the HashedString that holds the block's full name (for example `minecraft:skeleton_skull`): E0h on 26.63 (6aed1906-b127418e74d2f2f9). The LeviLamina header order gives the same value: vtable 8, mDescriptionId (std::string, 20h), the Material and padding to 30h, mComponents (60h) to 90h, then NameInfo with mRawName (a HashedString, 30h) and mNamespaceName (std::string, 20h), so mFullName is at E0h. Every caller of SkullBlockRenderer::renderSkull (0x6a5ea50) passes BlockType+E0h as the skull name argument, and renderSkull compares it with the 7 static skull block names (hash, then text), so the member is the block name. Recipe 0 is in SkullBlockRenderer::render (0x6a90c50), right before the call to renderSkull: `mov eax,imm32; mov rcx,[rbp+..]; add rax,[rcx+disp8]` (the BlockType pointer of the world Block, Block+68h) `; movss [rsp+..],xmm6; mov [rsp+..],r14; ...; call`, offset 0, the immediate of the mov. Recipe 1 is in the item skull path 0x598e6f0: `mov rax,[rax+disp32]` (Item+178h, the block type of the item) `; test rax,rax; je; mov rcx,[r12+disp8]; test rcx,rcx; je; add rax,imm32; ...; call`, offset 22, the immediate of the add. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range check is 80h..140h. No build gate: builds before 26.63 are not supported. The HashedString of the Preview is 38h bytes long (it was 30h on 26.63), so a consumer must read the text through a mirror that matches the build. Value E0h on 26.63. Class: new entry. What changed in 6afa4e47-dadc375797a3d31a (26.70.26): the value is E8h (8 bytes later, as for Item.mFullName on the Preview) and the type of the member changed with it (see above). Both recipes are now a pattern_any with one alternative per shape, so that they run (and give E8h) on this build too. The item path of 26.63 (`mov rax,[item+..]; test; je; mov rcx,[reg+..]; test; je; add rax,imm32; ..; call`) is the same on 26.70.26 with the block type of the item loaded from another member, a different register for the second test and a shorter run of stores before the call; the alternative 1 of recipe 1 is that path in its function (one byte before the load is wildcarded so that the offset of 22 to the add is the same). The world-block path of recipe 0 (`mov eax,imm32; mov rcx,[rbp+..]; add rax,[rcx+68h]`) is no longer in the function on 26.70.26: the function that calls the skull function with the item-type pointer is now another one, and the alternative 1 of recipe 0 reads the immediate of the `add rax,imm32` right before that call (offset 0, the immediate of the add). Evidence for E8h on 26.70.26: (1) a pattern drafted from the item skull function on 26.63 (the code after the release of the shared pointer, `lock dec dword ptr [rcx+..]` and the loads of the item block type up to the add) matches once on 26.70.26, in one function, and the add there is E8h; (2) a second function that calls the same skull function (the one that all callers of the skull function share, with the same argument order) loads the block type of the item the same way and adds E8h; the two functions share no pattern bytes. Class: real code change (layout); mods that read the name must use a mirror for the new HashedString."

  recipe {
    step "pattern_any" {
      patterns = [
        "B8 ?? ?? ?? ?? 48 8B 4D ?? 48 03 41 ?? F3 0F 11 74 24 ?? 4C 89 74 24 ?? 48 8B 4D ?? 48 89 4C 24 ?? 48 89 44 24 ?? F3 0F 11 7C 24 ?? 89 5C 24 ?? C6 44 24 ?? ?? 4C 8D 45 ?? 48 8B 4D ?? 48 89 F2 4C 8B 4D ?? E8",
        "48 05 ?? ?? ?? ?? 40 88 7C 24 ?? F3 0F 11 44 24 ?? 48 89 44 24 ?? 4C 8D 0D ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 48 8B 95 ?? ?? ?? ?? E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4C 24 ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 7C 24 ?? 0F 57 C0 0F 11 44 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8",
        "?? 48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4D ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 64 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8",
      ]
      offset = 22
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 128
    max = 320
  }
}
