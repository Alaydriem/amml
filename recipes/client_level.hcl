entry "ClientLevel.getLevelData" {
  kind     = "vslot"
  notes    = "Slot of LevelData& Level::getLevelData() in the Level vtable: 172 on 26.51 (172 and 173 hold one body, the two overloads; ClientLevel does not override it). The `$getLevelData` symbol is MCFOLD, so the slot is not graded by name. Three Level vtables hold Level::tick (named by its `virtual void __cdecl Level::tick(void)` profiler zone) at slot 119, which is the ILevel header order; getLevelData is header 172/173. The body asserts `T *Bedrock::NonOwnerPointer<LevelData>::_get() const` on this+80h. Recipe 0 is Level::getLevelSeed64 (slot 135), which calls this slot and then reads the seed from the result: `push rsi; sub rsp,..; mov rsi,rdx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 1 is Level::getSeed (slot 134): `sub rsp,..; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; lea rdx,[rip+..]; mov rcx,rax; call; mov rcx,rax; xor eax,eax`; it calls the const overload, which is the next slot, so the recipe subtracts 1. Range window 164..180."

  recipe {
    step "pattern" {
      bytes  = "56 48 83 EC ?? 48 89 D6 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 89 C1 E8"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 89 C1 E8 ?? ?? ?? ?? 48 89 C1 31 C0"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" { value = -1 }
  }

  check "range" {
    min = 164
    max = 180
  }
}
