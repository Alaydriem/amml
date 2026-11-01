entry "BiomeRegistry.lookupById" {
  kind     = "function"
  notes    = "Biome* BiomeRegistry::lookupById(BiomeIdType id). Signature: rcx = registry, dx = the id (BiomeIdType is a struct of one ushort, passed by value), the result in rax (null when the id is missing). The body binary-searches the registry's vector of Biome* sorted by id (begin +3B0h, end +3B8h) and compares `[biome+168h]` (Biome::mId) with dx. Recipe 0 is the start of the body (`mov r10,[rcx+disp]; mov rcx,[rcx+disp]; mov r11,rcx; sub r11,r10; sar r11,3; xor eax,eax; test r11,r11; jle; mov rsi,r11; jmp`), then function_start_of. Recipe 1 is a direct call after Level.getBiomeRegistry recipe 2 (`movzx edx,word ptr [rdi+..]; mov rcx,rax; call`), read as a branch target. It has 5 direct calls; 4 of them take the registry from Level.getBiomeRegistry and pass the id in dx. The recipes share no anchor bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Grade: two recipes only (no game run in this task). What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the body was compiled with other registers. Recipe 0 keeps its shape with the register bytes as wildcards (`mov reg,[rcx+..]; mov rcx,[rcx+..]; mov reg,rcx; sub; sar; xor eax,eax; test; jle; mov reg,reg; jmp`), unique on both builds. Evidence on 26.60: the call from Level.getBiomeRegistry users (recipe 1, drafted from the old caller) goes to the candidate; the old prologue with the registers wildcarded matches once on 26.60, in the candidate (the same vector at +3B0h/+3B8h and the binary search on the word at +168h). Class: real code change (register allocation)."

  recipe {
    step "pattern" {
      bytes = "4C 8B ?? ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 49 89 ?? 4D 29 ?? 49 C1 ?? ?? 31 C0 4D 85 ?? 0F 8E ?? ?? ?? ?? ?? 89 ?? EB"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 0F B7 57 ?? 48 89 C1 E8 ?? ?? ?? ?? F3 0F 10 40"
      offset = 27
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
