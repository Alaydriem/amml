entry "ChestBlockActor.mLargeChestPaired" {
  kind  = "field"
  notes = "Offset of `ChestBlockActor* mLargeChestPaired` in ChestBlockActor: 308h on 26.63 (6aed1906-b127418e74d2f2f9). It points to the other half of a large chest, or is null. The LeviLamina header puts it after mDeferredPairX and mDeferredPairZ (2FCh and 300h on 26.63, written by ChestBlockActor::load from `pairx` and `pairz`) and before mLargeChestPairedPosition (a BlockPos, 310h). How it was found: the only function that loads the text `pairlead` is ChestBlockActor::load (0x3314380); its `forceunpair` path reads the pointer at +308h, tests it, then clears +308h..+31Bh (`movups [rsi+308h],xmm0; mov dword ptr [rsi+318h],0`), the pointer and the position together. ChestBlockActor::save (0x3314aa0, it writes `pairlead`, `pairx` from +310h and `pairz` from +318h) writes the pair only when the pointer is not null. ChestBlockActor::pairWith (0x3316fb0) stores the other chest at +308h and copies its position (BlockActor+8, 12 bytes) to +310h. Recipe 0 is in save, after the first call: `test al,al; je rel32; cmp qword ptr [rdi+disp32],imm8; je rel32; movzx r8d,byte ptr [rdi+disp32]`, offset 8, the displacement of operand 0 of the cmp. Recipe 1 is in pairWith: `mov [rsi+disp32],r15; mov rax,[r15+disp8]; mov [rsi+disp32],rax; mov eax,[r15+disp8]; mov [rsi+disp32],eax`, offset 0, the displacement of operand 0 of the first store. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range check is 2A0h..380h."

  recipe {
    step "pattern" {
      bytes  = "84 C0 0F 84 ?? ?? ?? ?? 48 83 BF ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 44 0F B6 87 ?? ?? ?? ??"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes = "4C 89 BE ?? ?? ?? ?? 49 8B 47 ?? 48 89 86 ?? ?? ?? ?? 41 8B 47 ?? 89 86"
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 672
    max = 896
  }
}

entry "ChestBlockActor.mLargeChestPairedPosition" {
  kind  = "field"
  notes = "Offset of `BlockPos mLargeChestPairedPosition` in ChestBlockActor: 310h on 26.63 (6aed1906-b127418e74d2f2f9). It holds the position of the other half of a large chest (x, y, z as three ints). The LeviLamina header puts it right after mLargeChestPaired (308h). See ChestBlockActor.mLargeChestPaired for how the functions were found. Recipe 0 is in ChestBlockActor::save (0x3314aa0): after the call that writes `pairlead`, it loads the x of the position for `pairx`: `call rel32; mov r8d,[rdi+disp32]; mov qword ptr [rsp+disp32],imm32` (three times, the small-string set-up of the next key) `; mov dword ptr [rsp+disp32],imm32`, offset 5, the displacement of operand 1 of the load. Recipe 1 is in ChestBlockActor::pairWith (0x3316fb0): the copy of the other chest's position after the pointer store (`mov [rsi+disp32],r15; mov rax,[r15+disp8]; mov [rsi+disp32],rax; ...`), offset 11, the displacement of operand 0 of the qword store (the x and y of the position). The two recipes are in two functions. Recipe 1 uses the same pairWith bytes as recipe 1 of ChestBlockActor.mLargeChestPaired, but reads another instruction; recipe 0 shares no bytes with it. Every displacement, immediate and branch byte is a wildcard. The range check is 2A0h..380h."

  recipe {
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 44 8B 87 ?? ?? ?? ?? 48 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? C7 84 24 ?? ?? ?? ?? ?? ?? ?? ??"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 89 BE ?? ?? ?? ?? 49 8B 47 ?? 48 89 86 ?? ?? ?? ?? 41 8B 47 ?? 89 86"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 672
    max = 896
  }
}
