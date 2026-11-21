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
  notes    = "Slot of uint ILevel::getSeed() in the Level vtable: 134 (86h) in 26.51, 87h in 26.70 previews (ILevel header order: setTime, getSeed, getLevelSeed64). The result comes back in eax. Level has no vtable global, so both recipes are call sites whose result seeds a std::mt19937 (rax result stored, then the 6C078965h initialisation). Recipe 0 is the call `mov rcx,[rsi+20h]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]` followed by the seed-engine setup (`lea rcx,[rip+..]; mov [rbp+..],rcx; lea rcx,[rbp+..]; lea rdx,[rip+..]; mov [rbp+..],rdx; mov rdx,imm64`); a pattern_any with one alternative for a nop after the call and one without; the base register is pinned to rsi, which leaves out the same call in a sibling function (it loads the level through rax or rbx); the slot load is at offset 7. Recipe 1 is another function: `mov rsi,rcx; mov rax,[rdx]; mov rax,[rax+disp]; mov rcx,rdx; call [guard]; lea rcx,[rip+..]; mov [rbp+..],rcx; lea rcx,[rbp+..]` (the level is the second argument). Both read the displacement and turn it into a slot. Every displacement, immediate and branch byte is a wildcard. Range window 126..142. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the store before the call in the old recipe 0 no longer has the old register shape, so recipe 0 now starts at the level load of the rsi site; the slot moved from 86h to 87h (Level.getBiomeRegistry moved 54h to 55h the same way). Evidence on 70.26: the rsi call site of the old build (after `cmp bl,1; jne`) has one counterpart in the new build with the same following seed-engine code, and it reads displacement 438h (slot 87h); the old recipe-1 function maps to the new match of recipe 1 with the same displacement. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = ["48 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 8D 4D ?? 48 8D 15 ?? ?? ?? ?? 48 89 55 ?? 48 BA", "48 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 8D 4D ?? 48 8D 15 ?? ?? ?? ?? 48 89 55 ?? 48 BA"]
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
  notes    = "Slot of LevelSeed64 ILevel::getLevelSeed64() const in the Level vtable: 135 on 26.51, the slot after Level.getSeed, 88h in 26.70 previews. The body returns the 8-byte value through a hidden pointer (rdx), so a caller passes a buffer in rdx. Recipe 0 is the call site in the LevelChunk function that builds a random engine: `mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]; (nop); mov rdx,[rbp+..]; lea rax,[rip+..]; mov [..],rax`, a pattern_any with one alternative for a nop after the call and one without. Recipe 1 is the call site in a small function that reads the seed to float state: `mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rsp+..]; call [guard]; mov rax,[rsp+..]; mov rcx,rax; shr rcx,20h; xor ecx,eax; cmp ecx,0FFFFFFFFh` (the high word xor the low word, tested for -1), also with a nop alternative. Both read the displacement as a slot. Range window 127..143. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the slot moved from 87h to 88h, and the call sites have different surrounding code (a null check of Actor::mLevel and other registers), so the old patterns that started before the level load stopped matching; both recipes now start at the level vtable load. Evidence on 70.26: the old recipe-0 function (same size class, same random engine constants) and the old recipe-1 function each have one new counterpart with the same body shape that reads displacement 440h (slot 88h); Level.getSeed moved 86h to 87h the same way. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8B 95 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 48 8B 95 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89",
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
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? FF 15 ?? ?? ?? ?? 48 8B 44 24 ?? 48 89 C1 48 C1 E9 ?? 31 C1 83 F9 ??",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? FF 15 ?? ?? ?? ?? 90 48 8B 44 24 ?? 48 89 C1 48 C1 E9 ?? 31 C1 83 F9 ??",
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
    min = 127
    max = 143
  }
}

entry "Level.getHitResult" {
  kind     = "vslot"
  notes    = "Slot of HitResult& ILevel::getHitResult() in the Level vtable: 335 on 26.51 (14Fh in the later builds). The callers read the result as a HitResult (mType at +18h, copies of 28h/1Ch/40h). Level has no vtable global, so both recipes are call sites. Recipe 0 is the consumer that reads mType and the next 16 bytes after the call (`mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; nop; mov r32,[rax+18h]; movups xmm0,[rax+28h]`), a pattern_any with one alternative per register shape (44 8B for r14d, 8B for edi). Recipe 1 is a small wrapper function that takes the result and tests mType (`mov rcx,[rcx+20h]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; mov rdi,rax; ... cmp dword ptr [rdi+18h],0 ... mov eax,[rdi+28h]`). The next function holds the same code with [rdi+34h], so the byte 28h is kept (literal_ok). Both read the displacement as a slot. Range window 327..343. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the old flag-test recipe and the store-pair recipe no longer match (Actor::mLevel loads now have a null check), the slot did not move (14Fh on 26.51 to 26.70). Evidence on 70.26: the old consumer function and the old wrapper function each have one new counterpart with the same body that reads displacement A78h; a third pair (old function with the pair A78h then A80h reads, the new function with the same two reads in the same order) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 44 8B ?? ?? 0F 10 40 ??",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 8B ?? ?? 0F 10 40 ??",
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
      bytes  = "48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 89 C7 48 8B 4B ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 83 7F ?? ?? 74 ?? 31 C0 88 46 ?? 48 89 F0 48 83 C4 ?? 5B 5F 5E C3 8B 47 28"
      offset = 7
      literal_ok = [
        "71: mov eax,[rdi+28h]: the twin function right after it reads [rdi+34h] and holds the same code otherwise",
      ]
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
  notes    = "Slot of ItemRegistryRef ILevel::getItemRegistry() const in the Level vtable: 392 on 26.51, 18Bh in 26.70 previews. The ItemRegistryRef (a 16-byte weak_ptr) comes back through a hidden pointer in rdx and every caller then passes it to ItemRegistryRef::getCreativeItemRegistry. Level has no vtable global, so both recipes are call sites. Recipe 0 is a small function (`mov rbx,r8; mov rsi,rdx; mov rdi,rcx; mov rcx,[rcx+50h]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp-..]; call [guard]`). Recipe 1 is a larger sibling that starts with eight pushes (`sub rsp,imm32; lea rbp,[rsp+..]; mov rsi,rcx; mov rcx,[rcx+50h]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]; mov byte ptr [rbp+..],0; xorps xmm0,xmm0; movups; movaps; mov rax,[rbp+..]; test rax,rax; je`). The two recipes are in two different functions. Both read the displacement as a slot. Range window 384..400. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the slot moved from 188h to 18Bh, and the call sites that started with a Actor::mLevel load changed shape (null check); the two recipes now anchor on the functions that load the level through [rcx+50h]. Evidence on 70.26: the old small function and the old sibling each have one new counterpart with the same body (same prologue, same weak_ptr handling) that reads displacement C58h (slot 18Bh); the function order (sizes, first site offsets) of the five consecutive callers matches across builds. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "4C 89 C3 48 89 D6 48 89 CF 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ?? C6 45 ?? ?? 0F 57 C0 0F 11 45 ?? 0F 29 45 ?? 48 8B 45 ?? 48 85 C0 74 ??"
      offset = 45
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
