entry "ClientLevel.getLevelData" {
  kind     = "vslot"
  notes    = "Slot of LevelData& Level::getLevelData() in the Level vtable: 172 on 26.51 (172 and 173 hold one body, the two overloads; ClientLevel does not override it). The `$getLevelData` symbol is MCFOLD, so the slot is not graded by name. Three Level vtables hold Level::tick (named by its `virtual void __cdecl Level::tick(void)` profiler zone) at slot 119, which is the ILevel header order; getLevelData is header 172/173. The body asserts `T *Bedrock::NonOwnerPointer<LevelData>::_get() const` on this+80h. Recipe 0 is Level::getLevelSeed64 (slot 135), which calls this slot and then reads the seed from the result: `push rsi; sub rsp,..; mov rsi,rdx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 1 is Level::getSeed (slot 134): `sub rsp,..; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; lea rdx,[rip+..]; mov rcx,rax; call; mov rcx,rax; xor eax,eax`; it calls the const overload, which is the next slot, so the recipe subtracts 1. Recipe 1 is a pattern_any with one alternative per shape: on 26.60 to 26.63 getSeed ends `mov rcx,rax; xor eax,eax; test rcx,rcx; ...`, on 70.26 `test rax,rax; je; mov eax,[rax]; ret; xor eax,eax`; the plain head also matches two other getters on 70.26, so the alternative ends in the tail. Range window 164..180. What changed in 6afa4e47-dadc375797a3d31a (70.26): the Level vtable gained one virtual before this slot (getBiomeRegistry moved from 54h to 55h), so getLevelData is slot 173 there; both recipes read it from their callers (getLevelSeed64 calls it, getSeed calls the next slot) and agree. Class: would have survived with a better recipe."

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
    step "pattern_any" {
      patterns = [
        "48 83 EC ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 89 C1 E8 ?? ?? ?? ?? 48 89 C1 31 C0",
        "48 83 EC ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 48 89 C1 E8 ?? ?? ?? ?? 48 85 C0 74 ?? 8B 00 48 83 C4 ?? C3 31 C0",
      ]
      offset   = 7
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
