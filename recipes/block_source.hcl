entry "BlockSource.clip" {
  kind     = "vslot"
  notes    = "Slot of `HitResult BlockSource::clip(Vec3 const& A, Vec3 const& B, bool checkAgainstLiquid, ShapeType, int maxDistance, bool ignoreBorderBlocks, bool fullOnly, Actor*, std::function<bool(BlockSource const&, Block const&, bool)> const&, bool stopOnFirstLiquidHit) const` in the BlockSource vtable: 51 in 26.51. The HitResult comes back through a hidden buffer in rdx; A in r8, B in r9, the other eight arguments on the stack from [rsp+20h]. The overload `clip(ClipParameters const&)` is slot 50: it unpacks the parameters (`mov r10,[r8]; mov r9,[r8+8]; ...; mov rax,[rcx]; mov rax,[rax+198h]; mov [rsp+58h],dl; mov [rsp+50h],r11 (the std::function inside the parameters); ...; call`) and calls slot 51 with them, which fixes the argument order. The implementation (0x2d8aaa0) uses the std::function only through its callable pointer at +38h and `_Do_call` (slot 2, `call [rax+10h]`); it never copies, moves or destroys it, so a Rust-built std::function passed by const& is safe. Recipe 0 is that call in slot 50 (`mov rax,[rcx]; mov rax,[rax+disp]; mov [rsp+..],dl; mov [rsp+..],r11; mov [rsp+..],rdi`), read as a slot. Recipe 1 is the head of the implementation (`mov rsi,rdx; movss xmm0,[r8]; movss xmm1,[r8+4]; ucomiss xmm0,xmm0; movss xmm2,[r8+8]`, the NaN test of A), then function_start_of and slot_of the BlockSource vtable. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. LeviLamina header order does not give BlockSource slots (getChunk is 42, addListener 28). What changed in 6ad79ee4-17d51891a1119914: the start position check of clip is vectorised (`movsd xmm0,[r8]; ucomiss; movss xmm1,[r8+8]; jp; movaps; shufps`), so recipe 1 is a pattern_any with one alternative per shape; the slot (51) did not change. Evidence on 26.60: recipe 0 (the call site) and the recipe 1 alternative drafted from the 26.51 body reach the same function, slot 51."

  recipe {
    step "pattern" {
      bytes  = "48 8B 01 48 8B 80 ?? ?? ?? ?? 88 54 24 ?? 4C 89 5C 24 ?? 48 89 7C 24 ??"
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
        "48 89 D6 F3 41 0F 10 00 F3 41 0F 10 48 ?? 0F 2E C0 F3 41 0F 10 50 ??",
        "48 89 D6 F2 41 0F 10 00 0F 2E C0 F3 41 0F 10 48 ?? 0F 8A ?? ?? ?? ?? 0F 28 D0 0F C6 D0 ??",
      ]
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "BlockSource.vtable"
    }
  }

  check "range" {
    min = 40
    max = 70
  }
}

entry "BlockSource.fetchEntities" {
  kind     = "vslot"
  notes    = "Slot of `gsl::span<gsl::not_null<Actor*>> BlockSource::fetchEntities(Actor const* except, AABB const& bb, bool useHitbox, bool getDisplayEntities)` in the BlockSource vtable: 31 in 26.51. The span (the count at +0, the pointer at +8, as the implementation stores them: `mov [rdi],r8; mov [rdi+8],rcx`) comes back through a hidden buffer in rdx; except in r8, the box in r9, the two bools on the stack. The span points into a member vector of the BlockSource and is valid until the next fetch. One recipe: the body (`mov rsi,rdx; movzx edx,byte ptr [rsp+0D8h]; movzx eax,byte ptr [rsp+0D0h]; test r8,r8`, the two bool arguments and the null test of except), then function_start_of and slot_of the BlockSource vtable. The function then calls the implementation (0x2d845f0). Grade: the argument shape (two stack bools after a pointer and a reference) and the span store of the implementation. Every displacement, immediate and branch byte is a wildcard. Recipe 1 anchors in another part of the same body: the branch that wraps a non-null except pointer in a one-element span (`mov [rsp+58h],r8; mov qword ptr [rsp+98h],0; mov qword ptr [rsp+40h],1; lea r8,[rsp+58h]; mov [rsp+48h],r8; jmp`), then function_start_of and slot_of. The function is reached only through the vtable (no direct caller), so both recipes anchor in the body, as the ClientNetworkHandler handle slots do."

  recipe {
    step "pattern" {
      bytes = "48 89 D6 0F B6 94 24 ?? ?? ?? ?? 0F B6 84 24 ?? ?? ?? ?? 4D 85 C0"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "BlockSource.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "4C 89 44 24 ?? 48 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 4C 8D 44 24 ?? 4C 89 44 24 ?? EB"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "BlockSource.vtable"
    }
  }

  check "range" {
    min = 20
    max = 45
  }
}

entry "BlockSource.vtable" {
  kind     = "global"
  notes    = "The primary vtable of BlockSource (no RTTI). Found through the dimension: Dimension::init (Dimension.vtable slot 15) creates its BlockSource with a make_shared helper that allocates 188h bytes and calls the BlockSource constructor with (Level, Dimension, ChunkSource, true, false, false); the constructor stores this table at this+0 and then the level, chunk source and dimension at +20h, +28h and +30h, and copies the dimension height range (two words at dimension+0C8h/0CAh) to +38h/+3Ah. The table has the IConstBlockSource virtuals first: slot 19 (shouldFireEvents) tests `chunk+0F0h == 11`, slots 37 to 39 return +30h, slots 41 and 42 are the two getChunk overloads. Recipe 0 is the constructor store (`xorps; movups [rcx+8],xmm; lea rax,[rip+..]; mov [rcx],rax; call; mov [r14+..],eax` and three byte stores of the bool arguments). Recipe 1 is the destructor store (`mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax`, then the listener loop over the vector at +98h/+0A0h). Both read the RIP target of the lea. Every displacement, immediate and branch byte is a wildcard; the `[rcx]` store keeps its ModRM byte (a register, not a field). The section check keeps the address inside .rdata."

  recipe {
    step "pattern" {
      bytes  = "0F 57 ?? 0F 11 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 01 E8 ?? ?? ?? ?? 41 89 ?? ?? 45 88 ?? ?? 45 88 ?? ?? 45 88 ?? ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 81 ?? ?? ?? ?? 48 39 81 ?? ?? ?? ?? 74 ?? 31 FF"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "BlockSource.getChunk" {
  kind     = "vslot"
  notes    = "Slot of LevelChunk* BlockSource::getChunk(int x, int z) const in the BlockSource vtable: 42 in 26.51. The LeviLamina header (IBlockSource) declares getChunk(int, int) before getChunk(ChunkPos const&), at header-order slots 41 and 42; MSVC places overloads in reverse, so the table holds getChunk(ChunkPos const&) at 41 and getChunk(int, int) at 42. The slot 42 body proves it: it stores edx and r8d as a ChunkPos on the stack and calls slot 41 with its address (`mov [rsp+28h],edx; mov [rsp+2Ch],r8d; mov rax,[rcx]; mov rax,[rax+148h]; lea rdx,[rsp+28h]; call [guard]`); slot 41 compares the ChunkPos with the cached position and tests the chunk state against 11 (Loaded). Recipe 0 is that body, then function_start_of, then slot_of in BlockSource.vtable. Recipe 1 is a call site in a non-virtual BlockSource function that loops over chunk columns (`mov rax,[rdi]; mov rax,[rax+disp]; mov rcx,rdi; mov edx,r12d; mov r8d,r15d; call [guard]; mov r14,rax; test rax,rax; je`). Recipe 2 is a call site in a function next to the Dimension code (`mov rcx,[rbp+..]; mov rax,[rcx]; mov rax,[rax+disp]; mov edx,edi; mov r8d,r14d; call [guard]; mov rdi,rax; test rax,rax; je`), which then reads the chunk. Recipes 1 and 2 read the displacement and turn it into a slot. The three recipes are in three functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: register allocation changed (`mov edx,r12d` became `mov edx,r13d`). Recipe 1 now wildcards that ModRM byte and ends at the `call [guard]`. The old recipe 2 (a third call site) did not match on 26.60, and its function could not be found again with two pieces of evidence, so it was removed; recipes 0 and 1 are in two functions and agree (2Ah on both). Class: would have survived with a pattern that ends at the call and wildcards the register byte."

  recipe {
    step "pattern" {
      bytes = "89 54 24 ?? 44 89 44 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? FF 15"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "BlockSource.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 44 89 ?? 45 89 F8 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 34
    max = 50
  }
}

entry "BlockSource.getBrightnessPair" {
  kind     = "function"
  notes    = "BrightnessPair BlockSource::getBrightnessPair(BlockPos const&) const, a non-virtual member. The pair (2 bytes, sky first, block second) is returned through a hidden pointer: rcx = BlockSource this, rdx = the 2-byte result buffer, r8 = BlockPos const&; the function returns the buffer in rax. The body shows it: it looks the chunk up with BlockSource slot 41 (`mov rax,[rcx]; mov rax,[rax+148h]` on a ChunkPos of x>>4, z>>4), and when there is no chunk it copies the 2-byte default brightness at BlockSource+0E8h (`movzx eax,word ptr [rbx+0E8h]; mov [rsi],ax`); otherwise it reads the sky nibble from the sub chunk array at +8 and the block nibble from the array at +10h and stores them as `mov [rsi],r9b; mov [rsi+1],dl`, so sky is first. A thin wrapper (a function that forwards `mov rcx,[rcx+4A0h]` with rdx and r8 untouched) and a call site that passes `lea rdx,[rsp+..]; mov rcx,rsi; mov r8,rdi` and then reads the second byte of the buffer (`cmp byte ptr [rsp+2Dh],0Bh`, the block light) show the same registers. Recipe 0 is the tail of the body, the two byte stores and the stack-guard check, then function_start_of. Recipe 1 is that call site in a spawn-light function (a call right after a float compare against a constant): the target of the call is the function. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard; the patterns stop before the stack-guard call. The check is function_start, which Recipe 1 can fail. What changed in 6ad79ee4-17d51891a1119914 (26.60): the function is at a new place with the same body; only the end differs: the second byte store uses another register (`mov [rsi],r9b; mov [rsi+1],r10b`) and the stack-guard check is done in another order (`mov rax,[rsp+28h]; xor rax,rsp; mov rcx,[rip+..]; cmp rcx,rax; jne` instead of `mov rcx,[rsp+28h]; xor rcx,rsp; cmp rcx,[rip+..]; jne`). The old recipe 0 matched nothing; it is a pattern_any with one alternative per build now. Class: would have survived with a better recipe (register and guard-order bytes wildcarded, or an anchor away from the guard). Cold run result: degraded (recipe 0 no match, the call site agreed)."

  recipe {
    step "pattern_any" {
      patterns = [
        "44 88 ?? 88 ?? ?? 48 8B 4C 24 ?? 48 31 E1 48 3B 0D ?? ?? ?? ?? 75 ?? 48 89 F0",
        "44 88 ?? 44 88 ?? ?? 48 8B 44 24 ?? 48 31 E0 48 8B 0D ?? ?? ?? ?? 48 39 C1 75 ?? 48 89 F0",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 54 24 ?? 48 89 F1 49 89 F8 E8 ?? ?? ?? ?? 80 7C 24 ?? ?? 76 ??"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockSource.addListener" {
  kind     = "vslot"
  notes    = "Slot of void BlockSource::addListener(BlockSourceListener&) in BlockSource.vtable: 28 (1Ch) in 26.51 (LeviLamina header order: IConstBlockSource has 25 slots, then IBlockSource fetchAABBs 25, fetchCollisionShapes 26, getWeakRef 27, addListener 28, removeListener 29). The body finds the listener in the vector at +98h/+0A0h (a find helper) and appends it when it is missing (`mov rcx,[rcx+98h]; mov rdx,[rsi+0A0h]; mov r8,rbx; call; mov rdi,[rsi+0A0h]; cmp rax,rdi; jne`). Recipe 0 is that body, then function_start_of, then slot_of in BlockSource.vtable. Recipe 1 is an inlined BlockSourceHandle constructor (BlockSourceListener.vtable recipe 1) that calls the slot on the source with the new handle; the displacement is turned into a slot. Several such constructors exist. The recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot."

  recipe {
    step "pattern" {
      bytes = "48 8B 89 ?? ?? ?? ?? 48 8B 96 ?? ?? ?? ?? 49 89 D8 E8 ?? ?? ?? ?? 48 8B BE ?? ?? ?? ?? 48 39 F8 0F 85"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "BlockSource.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 04 24 49 83 C7 ?? 48 8D 05 ?? ?? ?? ?? 49 89 44 24 ?? 49 89 74 24 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 4C 89 FA"
      offset = 28
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 22
    max = 36
  }
}

entry "BlockSource.removeListener" {
  kind     = "vslot"
  notes    = "Slot of void BlockSource::removeListener(BlockSourceListener&) in BlockSource.vtable: 29 (1Dh) in 26.51 (LeviLamina header order: the slot after addListener). The body finds the listener in the vector at +98h and erases it with a memmove (`add qword ptr [rsi+0A0h],-8`). Another class has a byte-identical body with other member offsets, so the body is not used. Recipe 0 is the BlockSourceHandle deleting destructor (BlockSourceListener.vtable recipe 0): when the handle still has a source it calls the slot with itself (`mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,rsi; call [guard]`); the displacement is turned into a slot. Recipe 1 is BlockSource.addListener plus 1: a fixed slot distance inside one interface (the header declares the two next to each other). A virtual added between them makes the recipes disagree and the entry refuse. The recipes anchor on two functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 0 is now a pattern_any: the old pattern, and the same bytes with the nop after the `call [guard]` (the slot read is before the call, so the offset is the same). Both recipes give 1Dh on 26.60: recipe 0 reads the call in the BlockSourceHandle deleting destructor, recipe 1 is addListener + 1. Evidence on 26.60: the destructor call site, drafted from the old function, matches once; addListener (1Ch) resolves on its own. Ruling R17 (removeListener = addListener + 1): held on 26.60. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern_any" {
      patterns = ["89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 49 ?? 48 85 C9 74 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 FF 15 ?? ?? ?? ?? 48 C7 46 ?? ?? ?? ?? ??", "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 49 ?? 48 85 C9 74 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 FF 15 ?? ?? ?? ?? 90 48 C7 46 ?? ?? ?? ?? ??"]
      offset = 27
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "ref" { entry = "BlockSource.addListener" }
    step "add" { value = 1 }
  }

  check "range" {
    min = 22
    max = 36
  }
}

entry "BlockSource.getDimensionId" {
  kind     = "vslot"
  notes    = "Slot of DimensionType BlockSource::getDimensionId() const in BlockSource.vtable: 18 (12h) in 26.51 (LeviLamina IConstBlockSource header order: hasChunksAt 15 to 17, getDimensionId 18, shouldFireEvents 19). DimensionType is returned through a hidden out pointer in rdx, not in eax: the slot 18 body is `mov rsi,rdx; mov rcx,[rcx+30h]; mov rax,[rcx]; mov rax,[rax+10h]; call [guard]; mov rax,rsi` (it forwards to Dimension.getDimensionId, slot 2, which writes the id to [rdx]). 43 functions have that body, so it is not used. Recipe 0 is a call site after Actor.getDimensionBlockSource (`mov byte ptr [rbp+..],0; mov rcx,rbx; call; mov rcx,[rax]; mov r8,[rcx+disp]; lea rdx,[rbp+..]; mov rcx,rax; mov rax,r8; call [guard]; cmp dword ptr [rbp+..],1; jne`). Recipe 1 is a call site after Dimension.getBlockSourceFromMainChunkSource on a dimension weak reference (`mov rax,[rbx]; mov rax,[rax+disp]; mov rcx,rbx; call [guard]; mov rcx,[rax]; mov r8,[rcx+disp]; lea rbx,[rsp+..]; mov rcx,rax; mov rdx,rbx; mov rax,r8; call [guard]`). Both read the displacement of the r8 load and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 0 is now a pattern_any that starts at the slot read and ends after the compare: `mov r8,[rcx+disp]; lea rdx,[rbp+..]; mov rcx,rax; mov rax,r8; call [guard]; cmp dword ptr [rbp+..],imm8; jne near` without (26.51) or with (26.60) the nop after the call. On 26.51 the function has two such calls (the first with a short jne); the near jne picks one. 12h on both. Evidence on 26.60: the function is found from the old function by a draft (one match); the call after Actor.getDimensionBlockSource reads 90h; recipe 1 still matched. Class: real code change (one of the two calls was removed) plus the nops."

  recipe {
    step "pattern_any" {
      patterns = ["4C 8B 81 ?? ?? ?? ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? 83 7D ?? ?? 0F 85", "4C 8B 81 ?? ?? ?? ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? 90 83 7D ?? ?? 0F 85"]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 03 48 8B 40 ?? 48 89 D9 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 81 ?? ?? ?? ?? 48 8D 5C 24 ?? 48 89 C1 48 89 DA 4C 89 C0 FF 15"
      offset = 19
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 12
    max = 24
  }
}

entry "BlockSource.getBlock" {
  kind      = "function"
  msvc_name = "?$getBlock@BlockSource@@QEBAAEBVBlock@@AEBVBlockPos@@@Z"
  notes     = "Block const& BlockSource::getBlock(BlockPos const&) const, the body behind the `$getBlock` thunk: rcx = this, rdx = pos. It is slot 2 of BlockSource.vtable (IConstBlockSource order: getBlock(pos, layer) is slot 1 after the MSVC overload reverse, and its body jumps to slot 2 for layer 0 and slot 4, getExtraBlock, for layer 1). The body checks y against the height range (+38h/+3Ah), gets the chunk with slot 41 and reads the block from the sub chunk storage at +30h (getExtraBlock reads +38h). Recipe 0 is the storage read (`imul rax,rcx,68h; mov rcx,[rdx+rax+30h]; test rcx,rcx; je; movzx eax,word ptr [rsi+..]; movzx r9d,word ptr [rsi]`), then function_start_of; the storage byte 30h is kept. Recipe 1 is a direct call (`lea rdx,[rsp+..]; mov rbx,rcx; mov rcx,rdi; mov r14,r8; call`). Checks function_start and in_vtable BlockSource.vtable 0..3 (getExtraBlock at 4 is left out)."

  recipe {
    step "pattern" {
      bytes      = "48 6B C1 ?? 48 8B 4C 02 30 48 85 C9 74 ?? 0F B7 46 ?? 44 0F B7 0E"
      literal_ok = ["8: sub chunk storage +30h (layer 0): getExtraBlock, slot 4, reads +38h with the same code"]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 54 24 ?? 48 89 CB 48 89 F9 4D 89 C6 E8"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "BlockSource.vtable"
    min_slot = 0
    max_slot = 3
  }
}
