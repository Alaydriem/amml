entry "Biome.mId" {
  kind  = "field"
  notes = "Offset in Biome of mId, the BiomeIdType (a ushort) that BiomeRegistry.lookupById compares: 168h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). Recipe 0 is the Biome constructor (the one function that BiomeRegistry::registerBiome calls after it allocates the 1D0h-byte (26.63) or 1D8h-byte (70.26) Biome): it stores the id argument (`mov [rcx+disp32],dx`) and then sets up the member after it (`lea rax,[rcx+disp32]; mov [rbp+..],rax; movups [rcx+..],xmm6; mov qword ptr [rcx+..],0; mov byte ptr [rcx+..],1`), offset 0, the displacement of operand 0. Recipe 1 is BiomeRegistry.lookupById: the binary search over the sorted vector of Biome* reads the id of the middle element (`movzx reg,word ptr [reg+disp32]; cmp reg,dx; jb; ja`), offset 0, the displacement of operand 1. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. Evidence on both builds: the constructor stores its second argument (dx, the id) at this member, and lookupById compares the same member with its id argument. Class: new entry."

  recipe {
    step "pattern" {
      bytes = "66 89 91 ?? ?? ?? ?? 48 8D 81 ?? ?? ?? ?? 48 89 45 ?? 0F 11 B1 ?? ?? ?? ?? 48 C7 81 ?? ?? ?? ?? ?? ?? ?? ?? C6 81 ?? ?? ?? ?? ??"
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "BiomeRegistry.lookupById" }
    step "pattern" { bytes = "0F B7 ?? ?? ?? ?? ?? 66 39 ?? 72 ?? 77 ??" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "Biome.mHash" {
  kind  = "field"
  notes = "Offset in Biome of mHash, the HashedString that holds the biome name (for example `plains`): 190h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). The HashedString itself is 30h bytes on 26.63 and 38h bytes on 70.26 (HashedString.layout 0 / 1), so a reader selects the HashedString mirror by HashedString.layout. Recipe 0 is the Biome constructor: it makes this+190h the HashedString from the name argument (`mov [rbp+..],rcx; add rcx,imm32; movups xmm0,[r8]; movaps [rbp+..],xmm0; lea rdx,[rbp+..]; mov [rbp+..],rcx; call`), offset 4, the immediate of the add. Recipe 1 is BiomeRegistry::registerBiome, the only caller of the constructor, which reads the hash of the new biome to insert it into the registry map: a pattern_any with one alternative per build, offset 0, the displacement of operand 1. Alternative 0 (26.63) `mov rax,[rdi+disp32]; mov [rbp+..],rax; lea rcx,[rsi+..]; lea rdx,[rbp+..]; lea r8,[rbp+..]; mov r9,rbp; call`; alternative 1 (70.26) `mov rbx,[r14+disp32]; mov rcx,[rbp+..]; mov rdx,rbx; call` (a lookup by hash comes first on 70.26). Each alternative matches once on its own build and not on the other. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. Evidence on both builds: the constructor builds the HashedString at this member from its name argument, and registerBiome reads the first 8 bytes (the hash) of the same member right after the constructor returns. Class: new entry."

  recipe {
    step "pattern" {
      bytes  = "48 89 4D ?? 48 81 C1 ?? ?? ?? ?? 41 0F 10 00 0F 29 45 ?? 48 8D 55 ?? 48 89 4D ?? E8"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 87 ?? ?? ?? ?? 48 89 45 ?? 48 8D 8E ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 45 ?? 49 89 E9 E8",
        "49 8B 9E ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 48 89 DA E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}
