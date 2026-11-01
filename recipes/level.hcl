entry "Level.getCameraPresets" {
  kind     = "vslot"
  notes    = "Slot of `CameraPresets const& Level::getCameraPresets() const` in the Level vtable: 29 in 26.51 (ILevel interface). ClientNetworkHandler keeps the level at +C0h; both the CameraInstructionPacket handler and the CameraPresetsPacket handler call this slot on it and pass the result to the camera code as the preset list (the CameraInstructionPacket path, 0x5679ad0, reads the preset vector from it: `mov rbx,[r14]; mov rcx,[r14+8]`, stride 130h; the CameraPresetsPacket path, 0x5679a30, assigns the packet's presets to it). Recipe 0 is the CameraPresetsPacket handler (0x131bc60: `mov r14,[rbp-8]; mov r15,[rbp-28h]; mov rax,[rdi]; mov rax,[rax+0E8h]; mov rcx,rdi; call`), recipe 1 the CameraInstructionPacket handler (0x131b260: `lock inc dword ptr [rax+0Ch]; mov rbx,[rbp-10h]; mov rax,[rdi]; mov rax,[rax+0E8h]; mov rcx,rdi; call`); both read the displacement as a slot. The two recipes are in two functions and share only the call shape. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" {
      bytes  = "4C 8B 75 ?? 4C 8B 7D ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15"
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
      bytes  = "F0 FF 40 ?? 48 8B 5D ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 10
    max = 60
  }
}

entry "Level.getSeed" {
  kind     = "vslot"
  notes    = "Slot of uint ILevel::getSeed() in the Level vtable: 134 (86h) in 26.51 (LeviLamina ILevel header order: setTime 133, getSeed 134, getLevelSeed64 135; no overloads move it). The result comes back in eax. Level has no RTTI and no vtable global here, so both recipes are call sites whose result seeds a std::mt19937 (the next instructions run the 6C078965h initialisation on eax), which is how the game makes a seeded random from the level seed. Recipe 0 is in a function next to the Dimension code: `mov [rbx+..],rax; mov rcx,[rbx+20h]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; lea rcx,[rip+..]; mov [rbp+..],rcx; lea rcx,[rbp+..]` (rbx+20h is the level). Recipe 1 is in another function: `mov rsi,rcx; mov rax,[rdx]; mov rax,[rax+disp]; mov rcx,rdx; call [guard]; lea rcx,[rip+..]; mov [rbp+..],rcx; lea rcx,[rbp+..]` (the level is the second argument). 19 call sites of slot 134 seed a mt19937 this way. Both read the displacement and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. and the store before the call uses another register. Recipe 0 is now a pattern_any: the old bytes with the store ModRM byte as a wildcard, and the same with the nop after the call (the slot read is before the call). 86h on both. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern_any" {
      patterns = ["48 89 ?? ?? ?? ?? ?? 48 8B 4B ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 8D 4D ??", "48 89 ?? ?? ?? ?? ?? 48 8B 4B ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 8D 4D ??"]
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8B 02 48 8B 80 ?? ?? ?? ?? 48 89 D1 FF 15 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 8D 4D ??"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 126
    max = 142
  }
}

entry "Level.getBiomeRegistry" {
  kind     = "vslot"
  notes    = "Slot of BiomeRegistry& ILevel::getBiomeRegistry() (the non-const overload) in the Level vtable: 84 (54h) in 26.51. The LeviLamina ILevel header declares the const overload at 84 and the non-const one at 85; MSVC places overloads in reverse, so the table holds the non-const overload at 84 and the const one at 85. The two bodies are not told apart here (both return the registry); every call site below uses the result as a mutable registry. Recipe 0 is LevelChunk::getBiome (LevelChunk.getBiome), the 2D path: `mov rcx,[rsi+50h]` (LevelChunk::mLevel), the virtual call, then the biome vector of the registry at +3B0h/+3B8h. Recipe 1 is BlockSource::getBiome(BlockPos const&): after BlockSource slot 44 (getILevel) it calls the slot on the result, then binary-searches the same biome vector. Recipe 2 is a function that loops over biome ids: `mov rcx,[r14+50h]; ...; call [guard]; movzx edx,word ptr [rdi+10h]; mov rcx,rax; call BiomeRegistry::lookupById`. All three read the displacement and turn it into a slot. They are in three functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: after the call the registry members are loaded into another register. Recipe 0 is now LevelChunk.getBiome (ref) and, in it, `jmp; mov rcx,[rsi+..]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`, offset 12 (it ends at the call). 54h on both. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "ref" { entry = "LevelChunk.getBiome" }
    step "pattern" {
      bytes  = "E9 ?? ?? ?? ?? 48 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
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
      bytes  = "48 8B 08 48 8B 91 ?? ?? ?? ?? 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? 48 89 C7 48 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24"
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
      bytes  = "49 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 0F B7 57 ?? 48 89 C1 E8 ?? ?? ?? ?? F3 0F 10 40"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 76
    max = 92
  }
}

entry "Level.getLevelSeed64" {
  kind     = "vslot"
  notes    = "Slot of LevelSeed64 ILevel::getLevelSeed64() const in the Level vtable: 135 on 26.51, the slot after Level.getSeed (134), which is the ILevel header order (Level::tick at 119 lines up). The body returns the 8-byte value through a hidden pointer (rdx; it writes [rdx] and returns rdx), so a caller passes a buffer in rdx. getSeed (134) reads the low 32 bits of the same value. Recipe 0 is a call site in a LevelChunk function: `mov r12d,[rdx+..]; mov r15d,[rdx+..]; mov rcx,[rdx+50h]` (LevelChunk::mLevel) `; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]`. Recipe 1 is a call site on Actor::mLevel (`mov rcx,[rax+1D8h]`; the caller then shifts the result right by 32, so it uses all 64 bits): `test rax,rax; je; mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rsp+..]; call [guard]`. Both read the displacement as a slot. Range window 127..143. Recipe 0 is a pattern_any, one alternative per shape of the LevelChunk function: alternative 0 (26.51) reads the chunk through rdx, alternative 1 (26.60) through r9 (REX bytes 45h and 49h). Value 135 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the LevelChunk function (the one that loads the 1 Actor _setPos text and has the same 7 texts and size on both builds) holds the chunk in r9, so the 26.51 pattern did not match; recipe 1 (Actor::mLevel site) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "44 8B ?? ?? 44 8B ?? ?? 48 8B 4A ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15",
        "45 8B ?? ?? 45 8B ?? ?? 49 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15",
      ]
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? FF 15"
      offset = 19
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 127
    max = 143
  }
}

entry "Level.getHitResult" {
  kind     = "vslot"
  notes    = "Slot of HitResult& ILevel::getHitResult() in the Level vtable: 335 on 26.51 (ILevel header order; Level::tick at 119 and getItemRegistry at 392 line up with the binary). In the three Level vtables, 332..334 forward to the random member (getIRandom, getRandom, getThreadRandom), 335 is the leaf `mov rax,[rcx+1E8h]; ret` (the hit result wrapper), 336 returns that wrapper plus 88h (getLiquidHitResult) and 337 returns the wrapper through a hidden pointer. The callers read the result as a HitResult (`cmp dword ptr [rax+18h],0`, mType; `[rax+1Ch]`, mFacing). Both recipes are calls through Actor::mLevel (`mov rcx,[reg+1D8h]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`). Recipe 0 follows a test of a player flag (`jg; cmp byte ptr [rsi+..],..; jne`). Recipe 1 follows two stores into a struct (`mov dword ptr [rax+..],edx; mov byte ptr [rax+..],cl; mov rax,[rdi+..]`). Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 327..343. Recipe 0 is now a pattern_any from the flag test, one alternative per shape (jne on 26.51, je on 26.60), running past the call to the test of the hit type (`cmp dword ptr [rax+18h],..`); the slot load is at offset 19 in both. Value 335 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the flag test is inverted and Actor::mLevel moved (1D8h to 200h); recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "80 BE ?? ?? ?? ?? ?? 75 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 83 78",
        "80 BE ?? ?? ?? ?? ?? 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 83 78",
      ]
      offset = 19
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "88 88 ?? ?? ?? ?? 48 8B 47 ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 327
    max = 343
  }
}

entry "Level.getItemRegistry" {
  kind     = "vslot"
  notes    = "Slot of ItemRegistryRef ILevel::getItemRegistry() const in the Level vtable: 392 on 26.51 (ILevel header order). The ItemRegistryRef (a 16-byte weak_ptr) comes back through a hidden pointer in rdx. Every caller of ItemRegistryRef::getCreativeItemRegistry that starts from a level does `mov rcx,[actor+1D8h]` (Actor::mLevel), `mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[local]; call [guard]`, then passes the local as the ItemRegistryRef this. Recipe 0 is one of them: `mov r14b,r8b; mov rsi,rcx; mov rax,[rcx+..]; mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp-..]; call [guard]`. Recipe 1 is another: `mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; lea rbx,[rbp+..]; mov rdx,rbx; call [guard]`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 384..400. Recipe 1 is now the caller that passes the result to ItemRegistryRef.getCreativeItemRegistry right after the call (`mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; lea r64,[rbp+..]; mov rdx,r64; call [guard]; lea rdx,[rbp+..]; mov rcx,r64; call`), with the local register wildcarded. Value 392 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 pattern of recipe 1 matched another call (Level slot 27) on 26.60, so the recipes disagreed and the entry was refused (no wrong value); 15 sites load slot 392 through Actor::mLevel on each build. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "45 88 C6 48 89 CE 48 8B 41 ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ?? 48 89 ?? FF 15 ?? ?? ?? ?? 48 8D 55 ?? 48 89 ?? E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 384
    max = 400
  }
}
