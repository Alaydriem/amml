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
  notes    = "Slot of BlockType::getMapColor (mce::Color getMapColor(BlockSource&, BlockPos const&, Block const&) const) in the BlockType vtable: 128 (80h) on 26.51, 129 (81h) on 26.60. When a block has no BlockMapColorComponent the map-colour functions call it on Block+68h (the block type): `mov rcx,[reg+68h]; mov rax,[rcx]; mov rax,[rax+disp]; mov r10,[rip+..]` (the guard), then `mov [rsp+20h],reg`. Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rcx,[r12+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r10,[rip+..]; mov [rsp+..],r12`, operand 1 displacement as a slot. Recipe 1: function B, then `mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r10,[rip+..]; mov [rsp+..],rbx`, the same. They are in two functions. Check range 110..150. Evidence on 26.60: the call on the block type in the no-component branch of function A and of function B (the same place as on 26.51, after the tryGetComponent test) both read 408h, so the slot moved by one (an earlier BlockType virtual was added). The old recipe 1 matched both twins on 26.60 and the derive refused; no wrong value. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus range."

  recipe {
    step "pattern" { bytes = "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15" }
    step "pattern" {
      bytes  = "49 8B 4C 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 8B 15 ?? ?? ?? ?? 4C 89 64 24"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 85 ?? ?? ?? ?? 48 8B 7F ?? 48 89 D9 E8", "89 45 ?? 48 8B 7F ?? 48 89 D9 E8"]
    }
    step "pattern" {
      bytes  = "48 8B 4B ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 8B 15 ?? ?? ?? ?? 48 89 5C 24"
      offset = 7
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
  notes    = "Slot of bool BlockType::canBeBuiltOver(Block const&, BlockSource const&, BlockPos const&) const, the overload that the easy place C++ calls (four arguments with this: rcx the type, rdx the block, r8 the region, r9 the position), in the BlockType vtable: 83 on 26.51. The LeviLamina header order puts the overload with the extra BlockType const& first (header 83/84 after the same shift as the known slots getCollisionShape 4, canProvideSupport 18 and getMapColor 128), but MSVC lays the pair out the other way: in the base table slot 84 reads a fifth argument (`mov rax,[rsp+60h]; test byte ptr [rax+130h],8`) and first calls slot 83 (`mov rax,[rcx]; mov rax,[rax+298h]; call [guard]`), so 84 is the five-argument overload and 83 is this one. Both recipes are calls through Block::mBlockType (`mov rcx,[block+68h]; mov rax,[rcx]; mov rax,[rax+disp]`) with the block in rdx, the region in r8 and the position in r9, followed by `test al,al`. Recipe 0 is in a BlockSource function (`mov rcx,[rdx+..]; ...; mov r8,rsi; mov r9,rbx; call [guard]`). Recipe 1 is in another function (`mov rax,[rax+disp]; lea r9,[rbp+..]; mov rdx,rdi; mov r8,r14; call [guard]`). Both read the displacement as a slot. The two recipes are in two functions. Every displacement and branch byte is a wildcard. Range window 75..91. Recipe 1 is now another site with the block in r12: `mov rcx,[r12+68h]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,r12; mov r8,rdi; mov r9,r14; call [guard]`, then `test al,al; je`. Value 83 on 26.51, 84 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): BlockType gained one virtual before this pair (asItemInstance 77 to 78, getMapColor 128 to 129, this 83 to 84); the 26.51 site of recipe 1 changed shape. The new site was found by drafting each 26.60 call of slot 84 and keeping one whose pattern matches once on each build and calls slot 83 on 26.51; recipe 0 (the BlockSource site) agrees on 84. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 8B 4A ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 49 89 F0 49 89 D9 FF 15"
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
  notes    = "Slot of bool BlockType::isInteractiveBlock() const in the BlockType vtable: 119 on 26.51 (LeviLamina header 126 with the same shift of 7 as getMapColor 128). Of the 270 BlockType vtables that hold the named `BlockType::getMobToSpawn` (slot 86), 62 hold `mov al,1; ret` at slot 119 and the rest the shared `xor eax,eax; ret`; the slots next to it are overridden by 2 and 5 tables only. Both recipes test a BlockType flag byte and then call this slot on the type: `test byte ptr [rcx+..],..; jne; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 0 is in GameMode::useItemOn (GameMode slot 13), after `mov al,..`. Recipe 1 is in ClientInputCallbacks::handleBuildAction, after the type load `mov rcx,[rax+..]`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 111..127. Recipe 0 now starts at the block load (`mov rax,[rbp+..]; mov rcx,[rax+68h]; mov al,..; test; jne; ..`, offset 25), because a second function has the same flag test and call on 26.60. Recipe 1 is now inside ClientInputCallbacks.handleBuildAction (ref), the call after the flag test with the near jne (`test byte ptr [rcx+..],..; jne near; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`). Value 119 on 26.51, 120 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): BlockType gained one virtual before this slot; another function got the same test-and-call shape (recipe 0 had 2 matches, both slot 120), and in handleBuildAction the type load before the short-jne site changed. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 8B 85 ?? ?? ?? ?? 48 8B 48 ?? B0 ?? F6 81 ?? ?? ?? ?? ?? 75 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 25
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "ref" { entry = "ClientInputCallbacks.handleBuildAction" }
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
