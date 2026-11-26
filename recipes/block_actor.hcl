entry "BlockActor.loadStatic" {
  kind  = "function"
  notes = "static std::shared_ptr<BlockActor> BlockActor::loadStatic(BlockType const&, BlockPos const&, ILevel&, CompoundTag const&, DataLoadHelper&): rcx = the hidden return pointer (16 bytes), rdx = the BlockType, r8 = the position, r9 = the ILevel, [rsp+20h] at the call = the CompoundTag, [rsp+28h] = the DataLoadHelper; it returns rcx in rax. The body calls the global block actor factory std::function (impl pointer at 0x1296cf98 on 26.63, slot 2 `[rax+10h]` = _Do_call, with rdx = a local out pointer, r8 = the position, r9 = the BlockType); that function is VanillaBlockActorFactory.createBlockActor. It returns an empty pointer when the global is empty or the factory gives none. Then it reads the `id` string of the tag (`lea rax,[rip+..]; mov qword ptr [rbp-48h],2`), hashes it (FNV-1a) and looks it up. How it was found: MovingBlockActor::load (0x4c426d0, the only function that loads the string `movingEntity`) reads the `movingEntity` compound and its x, y and z, calls DataLoadHelper slot 2 (loadBlockPosition), and then calls this function with the moving block's BlockType (`mov rax,[rsi+0F8h]; mov rdx,[rax+68h]`), the position, the level, the tag and the helper, and stores the result in its block actor member (+108h). Callers on 26.63: 0x1fa4940, 0x385c690, 0x4c426d0, 0x69869d0, 0x781b820, 0x88fb880. Recipe 0 is the body after the call of the block actor factory (`cmp qword ptr [rbp+..],imm; je rel32; mov rcx,[rbp+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov qword ptr [rbp+..],imm32`: the compare of the factory's out pointer, then the load of the `id` string), then function_start_of. Recipe 1 is the call in MovingBlockActor::load (`mov rax,[rsi+disp32]; mov rdx,[rax+disp8]; mov [rsp+28h],rbx; mov [rsp+20h],r14; lea rcx,[rsp+..] or lea rcx,[rbp-..]; mov r8,r15; mov r9,rdi; call`), read as a branch target; a pattern_any with one alternative for each form of the `lea` (5 bytes on 26.63, 4 bytes on 70.26, a wildcard in front of the second so that the call stays at offset 32). The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Value 0x3301fb0 on 26.63 (6aed1906-b127418e74d2f2f9). What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the function head changed (a virtual call on the level before the factory call, whose result goes to the factory as a new by-pointer bool argument and a stack argument; the old head pattern had `mov rsi,rcx` first and does not match), so recipe 0 now anchors after the factory call, in the part that did not change; the factory call has a new shape, which is for the owner of VanillaBlockActorFactory.createBlockActor. The caller MovingBlockActor::load keeps its call but uses the frame (`lea rcx,[rbp-..]`) for the return slot. The string `movingEntity` is not found by xrefs on 70.26, so the callers were matched by shape. Evidence from the old function: (1) the body after the factory call, drafted from the old function, matches once on 70.26, in a function with 6 direct callers (26.63: 6); (2) the old call site in MovingBlockActor::load (member +0F8h to the BlockType, position, level, tag, helper in the same registers) matches once on 70.26 as the call of that same function, from the function whose block actor member is at +100h. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes = "48 83 7D ?? ?? 0F 84 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 86 ?? ?? ?? ?? 48 8B 50 ?? 48 89 5C 24 ?? 4C 89 74 24 ?? 48 8D 4C 24 ?? 4D 89 F8 49 89 F9 E8",
        "?? 48 8B 86 ?? ?? ?? ?? 48 8B 50 ?? 48 89 5C 24 ?? 4C 89 74 24 ?? 48 8D 4D ?? 4D 89 F8 49 89 F9 E8",
      ]
      offset = 32
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockActor._getRenderComponent" {
  kind  = "vslot"
  notes = "Slot of IVanillaRenderBlockActorComponent* BlockActor::_getRenderComponent() (the non-const overload) in the BlockActor vtable: 21 on 26.63 (6aed1906-b127418e74d2f2f9). rcx = the block actor; it returns the component pointer (an interior base of the actor) in rax, or null. The LeviLamina header order gives 21 for the first of the two overloads after _playerCanUpdate (dtor 0, load 1, ... _playerCanUpdate 20). Both call sites pass the result as the component (r9) of BlockActorRenderDispatcher.render, which then calls component slots 2, 4 and 6, so the slot belongs to BlockActor and returns the render component. Recipe 0 is in the block selection overlay draw (0x539b520): the block actor in rax (`mov rdx,rax; mov rax,[rax]; mov rax,[rax+disp32]; mov rcx,rdx; mov [rbp+..],rdx; call [guard]`), offset 6; the result goes to the local that is later passed to the first render form, and the same function tests the actor type at +14h (BlockActor::mType). Recipe 1 is in MovingBlockActorRenderer's draw (0x6a83a10), on the moving block's actor member (`mov qword ptr [rax+..],imm32; mov rcx,[r12+disp32]; test rcx,rcx; je rel32; mov rax,[rcx]; mov rax,[rax+disp32]; call [guard]`), offset 28 (a pattern_any: on 70.26 the draw gets the actor from a call, not from a member, and loads the slot with `mov rdx,[rcx+disp32]; mov rcx,rax; mov rax,rdx`, so the second alternative has seven wildcards in front to keep the displacement at offset 28); the result is passed straight to the renderPos form. Both displacements are A8h and are turned into a slot. The two recipes are in two functions. Every displacement and immediate is a wildcard. The range check 17..26 covers the BlockActor virtuals from triggerEvent to the last component getter. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): MovingBlockActorRenderer's draw still makes the call (displacement A8h), but gets the block actor from a function call, not from the member, and the vcall loads its target into rdx first, so recipe 1 got a second alternative. Evidence from the old function: (1) the draw function is found on both builds by the one function that loads the string `MovingBlock Override`; (2) both draws pass the result of the call as r9 to the same function, BlockActorRenderDispatcher.render (26.63 at 0x6a84853, 70.26 at 0x8923bb8, both inside that draw), so the new call site is the old one. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 89 C2 48 8B 00 48 8B 80 ?? ?? ?? ?? 48 89 D1 48 89 95 ?? ?? ?? ?? FF 15"
      offset = 6
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
        "48 C7 40 ?? ?? ?? ?? ?? 49 8B 8C 24 ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15",
        "?? ?? ?? ?? ?? ?? ?? 4C 89 ?? E8 ?? ?? ?? ?? 90 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 08 48 8B 91 ?? ?? ?? ?? 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? 90 48 85 C0",
      ]
      offset = 28
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 17
    max = 26
  }
}

entry "BlockActor.mPosition" {
  kind  = "field"
  notes = "Offset of `BlockPos mPosition` in BlockActor: 8h on 26.63 (6aed1906-b127418e74d2f2f9). It holds the block actor's own position (x, y, z as three ints), right after the vtable pointer, as in the LeviLamina header. How it was found: SkullBlockRenderer::render (0x6a90c50) gets the block actor from the render data, tests isType(6) and then asks the BlockSource (vslot 2, getBlock) for the block at `lea rdx,[actor+8]`; ChestBlockActor's renderer (0x6a6ad80) picks the pair axis from bit 1 of ChestBlockActor+2F8h and compares mLargeChestPairedPosition[axis] (`[r14+rax*8+310h]`) with `[r14+rax*8+8]`, the same axis of the actor's own position; ChestBlockActor::pairWith (0x3316fb0) copies the other chest's +8h (12 bytes) to mLargeChestPairedPosition. Recipe 0 is in the chest renderer: `movzx eax,byte ptr [r14+disp32]; and al,imm8; shr al,1; movzx eax,al; mov ecx,[r14+rax*8+disp32]; xor edx,edx; cmp ecx,[r14+rax*8+disp8]; setl dl`, offset 25, the displacement of operand 1 of the cmp. Recipe 1 is in SkullBlockRenderer::render, after the isType test: `test al,al; je rel32; mov [rbp+..],r13; lea rdx,[r15+disp8]; mov rax,[rdi]; mov rax,[rax+disp8]; mov rcx,rdi; call [guard]`, offset 12, the displacement of operand 1 of the lea. The two recipes are in two functions of two classes and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range check is 8h..20h. Value 8h on 26.63. Class: new entry."

  recipe {
    step "pattern" {
      bytes  = "41 0F B6 86 ?? ?? ?? ?? 24 ?? D0 E8 0F B6 C0 41 8B 8C C6 ?? ?? ?? ?? 31 D2 41 3B 4C C6 ?? 0F 9C C2"
      offset = 25
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "84 C0 0F 84 ?? ?? ?? ?? 4C 89 6D ?? 49 8D 57 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15"
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 32
  }
}

entry "BlockActor.isType" {
  kind  = "vslot"
  notes = "Slot of `bool BlockActor::isType(BlockActorType) const` in the BlockActor vtable: 16 on 26.63 (6aed1906-b127418e74d2f2f9). rcx = the block actor, dl = the BlockActorType (a byte); it returns the answer in al. The base form compares the type with BlockActor::mType (+14h). ChestBlockActor's form (0x3313a10, a leaf with no unwind data: `cmp dl,2; sete r8b; cmp [rcx+14h],dl; sete al; or al,r8b; ret`) also answers true for 2 (Chest), so every ChestBlockActor and every class made from its constructor answers isType(2). VanillaBlockActorFactory::createBlockActor (0x7150f90) case 2 reads the chest kind byte of the BlockType (+378h) and calls 0x3313960, which allocates 380h bytes and runs the ChestBlockActor constructor (0x32f2d60) with type 2 and that kind: chest, trapped chest and the copper chests are one actor class that differs only in mChestType. How it was found: ChestRenderer::render (0x6a6ad80) and SkullBlockRenderer::render (0x6a90c50) both get the block actor from the render data, then `mov rax,[rax]; mov rax,[rax+80h]; mov rcx,actor; mov dl,type; call [guard]; test al,al; je` with type 2 and type 6. Recipe 0 is in the chest renderer, before the lead bit test (`mov rax,[rax]; mov rax,[rax+disp32]; mov rcx,r14; mov dl,imm8; call [guard]; nop; test al,al; je rel32; test byte ptr [r14+disp32],imm8`), offset 3. Recipe 1 is in the skull renderer, before the mPosition lea (`mov r15,rax; mov rax,[rax]; mov rax,[rax+disp32]; mov rcx,r15; mov dl,imm8; call [guard]; nop; test al,al; je rel32; mov [rbp+..],r13; lea rdx,[r15+disp8]`), offset 6, as a pattern_any with one alternative for each form of the stores after the branch (one store on 26.63, two on 70.26); the registers are wildcards. Both displacements are 80h and are turned into a slot. The two recipes are in two functions of two classes and do not overlap. Every displacement, immediate and branch byte is a wildcard. The range check 8..24 covers the BlockActor virtuals between load and the render component getter. Class: new entry. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the skull renderer keeps the call but the compiler chose other registers (r14 for the actor, r15 for the position store) and stores one more value before the `lea`, so recipe 1 wildcards the registers and has an alternative for the longer store run. Evidence from the old function: (1) the call site in the skull renderer, drafted from the old function (the type byte 6 is a wildcard), matches once on 70.26 with the same instruction sequence around it (movss from [rdx+0B0h], the virtual call on [rax+8], the displacement 80h, `mov dl,6`); (2) recipe 0, from the chest renderer, a different old function, gives the same slot on 70.26. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 F1 B2 ?? FF 15 ?? ?? ?? ?? 90 84 C0 0F 84 ?? ?? ?? ?? 41 F6 86"
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
        "49 89 ?? 48 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 ?? B2 ?? FF 15 ?? ?? ?? ?? 90 84 C0 0F 84 ?? ?? ?? ?? 4C 89 ?? ?? 49 8D ?? ??",
        "49 89 ?? 48 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 ?? B2 ?? FF 15 ?? ?? ?? ?? 90 84 C0 0F 84 ?? ?? ?? ?? 4C 89 ?? ?? 48 89 ?? ?? 49 8D ?? ??",
      ]
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 8
    max = 24
  }
}
