entry "BlockSourceListener.vtable" {
  kind     = "global"
  notes    = "A 9-slot BlockSourceListener table whose slots 1, 3 and 5 to 8 hold the BlockSourceListener defaults. The exe has no table of BlockSourceListener itself (no code stores one; the LeviLamina header has no vftable for it), so this is the table of BlockSourceHandle, the smallest BlockSourceListener subclass (LeviLamina BlockSourceHandle.h: it overrides only the destructor and onSourceDestroyed). Its slots: 0 the deleting destructor, 1 the shared `ret` stub, 2 BlockSourceHandle::onSourceDestroyed (`mov qword ptr [rcx+8],0; ret`), 3 and 4 the `ret` stub, 5 the default onBrightnessChanged (`mov rax,[rcx]; mov rax,[rax+18h]; mov r10,[guard]; mov r9,r8; jmp r10`, which calls onAreaChanged at slot 3 with the position twice), 6 to 8 the `ret` stub. 28 tables hold that onBrightnessChanged at slot 5; the Dimension LevelListener table holds the same defaults at 1, 2, 3, 5, 6 and 7. The mod copies the table and overwrites slots 0, 2 and 4, so slot 2 of the handle never runs. Recipe 0 is the BlockSourceHandle deleting destructor (`mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rcx,[rcx+8]; test; je; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,rsi; call [guard]; mov qword ptr [rsi+8],0`, which calls BlockSource.removeListener). Recipe 1 is an inlined BlockSourceHandle constructor in a make_shared function (`mov [r12],rax; add r15,10h; lea rax,[rip+..]; mov [r12+10h],rax; mov [r12+18h],rsi; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; mov rdx,r15`, which calls BlockSource.addListener). Both read the RIP target of the lea. Every displacement, immediate and branch byte is a wildcard. The section check keeps the address inside .rdata. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 0 is now a pattern_any: the old pattern, and the same bytes with the nop after the `call [guard]` (the lea of the table is before the call). Recipe 1 (the BlockSource constructor) still matched. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern_any" {
      patterns = ["89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 49 ?? 48 85 C9 74 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 FF 15 ?? ?? ?? ?? 48 C7 46 ?? ?? ?? ?? ??", "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 49 ?? 48 85 C9 74 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 FF 15 ?? ?? ?? ?? 90 48 C7 46 ?? ?? ?? ?? ??"]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 04 24 49 83 C7 ?? 48 8D 05 ?? ?? ?? ?? 49 89 44 24 ?? 49 89 74 24 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 4C 89 FA"
      offset = 8
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

entry "BlockSourceListener.onSourceDestroyed" {
  kind     = "vslot"
  notes    = "Slot of void BlockSourceListener::onSourceDestroyed(BlockSource&) in a BlockSourceListener table: 2 in 26.51 (LeviLamina header BlockSourceListener.h: destructor 0, onSourceCreated 1, onSourceDestroyed 2). Recipe 0 is the BlockSource destructor (the function of BlockSource.vtable recipe 1): it loops over the listener vector at +98h/+0A0h and calls the slot with the source (`mov rcx,[rax+rdi*8]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,rsi; call [guard]; inc rdi`). Recipe 1 is the onBlockChanged call site in BlockSource::fireBlockChanged (BlockSourceListener.onBlockChanged recipe 0) minus 2: a fixed slot distance inside one 9-slot interface. A virtual added between the two makes the recipes disagree and the entry refuse; it never gives a value on its own. Both read a displacement and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is the 9 slots of the interface. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 0 (the BlockSource destructor loop) now ends at the `call [guard]`; it is unique on 26.51 and 26.60. Both recipes give 2 on 26.60. Evidence on 26.60: the destructor call site, drafted from the old function, matches once and reads 10h; the fireBlockChanged call site gives 4 - 2. Ruling R17: held on 26.60. The 9-slot listener table did not change. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern" {
      bytes  = "48 8B 0C F8 48 8B 01 48 8B 40 ?? 48 89 F2 FF 15"
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
      bytes  = "4A 8B 0C E0 48 8B 01 48 8B 40 ?? 48 8B 94 24 ?? ?? ?? ?? 48 89 54 24 ?? 40 88 6C 24"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" { value = -2 }
  }

  check "range" {
    min = 0
    max = 8
  }
}

entry "BlockSourceListener.onBlockChanged" {
  kind     = "vslot"
  notes    = "Slot of void BlockSourceListener::onBlockChanged(BlockSource&, BlockPos const&, uint layer, Block const& block, Block const& oldBlock, int updateFlags, ActorBlockSyncMessage const*, BlockChangedEventTarget, Actor*) in a BlockSourceListener table: 4 in 26.51 (LeviLamina header: onAreaChanged 3, onBlockChanged 4). Recipe 0 is BlockSource::fireBlockChanged (BlockSource.vtable slot 63): it loops over the listener vector at +98h and calls the slot with rdx = the source, r8 = pos, r9d = layer and the stack arguments block (+20h), oldBlock (+28h), flags (+30h), syncMsg (+38h), eventTarget (+40h, a byte) and source actor (+48h), the header order (`mov rcx,[rax+r12*8]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[rsp+..]; mov [rsp+48h],rdx; mov [rsp+40h],bpl`). It is the only call site of the slot. Recipe 1 is the onSourceDestroyed call in the BlockSource destructor (BlockSourceListener.onSourceDestroyed recipe 0) plus 2: a fixed slot distance inside one interface; a virtual added between them makes the recipes disagree and the entry refuse. Both read a displacement and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is the 9 slots of the interface. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 1 (the BlockSource destructor loop) now ends at the `call [guard]`: `mov rcx,[rax+rdi*8]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,rsi; call [guard]`; it is unique on 26.51 and 26.60. Both recipes give 4 on 26.60. Evidence on 26.60: the fireBlockChanged call site (recipe 0) still matches once; the destructor call site, drafted from the old function, matches once. Ruling R17 (the shared call sites, slot distance 2): held on 26.60 (onSourceDestroyed 2, onBlockChanged 4). The 9-slot listener table did not change. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern" {
      bytes  = "4A 8B 0C E0 48 8B 01 48 8B 40 ?? 48 8B 94 24 ?? ?? ?? ?? 48 89 54 24 ?? 40 88 6C 24"
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
      bytes  = "48 8B 0C F8 48 8B 01 48 8B 40 ?? 48 89 F2 FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" { value = 2 }
  }

  check "range" {
    min = 0
    max = 8
  }
}
