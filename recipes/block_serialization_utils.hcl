entry "BlockSerializationUtils.tryGetBlockFromNBT" {
  kind      = "function"
  msvc_name = "?tryGetBlockFromNBT@BlockSerializationUtils@@YA?AU?$pair@W4NBTState@BlockSerializationUtils@@PEBVBlock@@@std@@AEBVCompoundTag@@PEAUNbtToBlockCache@1@@Z"
  notes     = "std::pair<NBTState, Block const*> BlockSerializationUtils::tryGetBlockFromNBT(CompoundTag const&, NbtToBlockCache*): a free function, rcx = the 16-byte result (hidden return pointer; the Block* is at +8), rdx = the tag, r8 = the cache (may be null). It first looks up the 4-character name key in the tag (CompoundTag contains, then get). The structure palette parser calls it for each palette entry and reads the Block* at result+8. Recipe 0 is the body head (`mov rdi,r8; mov r14,rdx; mov rsi,rcx; lea r13,[rip+..]; mov [rbp+..],r13; mov qword ptr [rbp+..],4; lea rdx,[rbp+..]; mov rcx,r14`), then function_start_of. Recipe 1 is the palette parser call (`test rax,rax; je; lea rcx,[rbp+..]; mov rdx,rax; lea r8,[rbp+..]; call`). Check function_start. Recipe 0 now starts at `mov r14,rdx` and wildcards the register of the name key (r13 on 26.51, r15 on 26.60); it ends at the CompoundTag contains call. Recipe 1 is a pattern_any, one alternative per shape, the call at offset 19 in both: alternative 0 (26.51) from `test rax,rax`, with `lea r8,[rbp+disp8]`; alternative 1 (26.60) from the je, with `lea r8,[rbp+disp32]` and the nop and the result read after the call. What changed in 6ad79ee4-17d51891a1119914 (26.60): the name key is in r15 and the palette parser has its locals at disp32 frame offsets. The 26.60 function was accepted on two facts from the 26.51 function: the same body (the 4-character key, contains, then get and `lea rdx,[rax+8]`, the same texts name, states, version and minecraft), and the same caller shape (the structure palette parser passes a non-null cache in r8 right after a null test of the tag). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "49 89 D6 48 89 CE 4C 8D ?? ?? ?? ?? ?? 4C 89 ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 55 ?? 4C 89 F1 E8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 85 C0 74 ?? 48 8D 8D ?? ?? ?? ?? 48 89 C2 4C 8D 45 ?? E8",
        "74 ?? 48 8D 8D ?? ?? ?? ?? 48 89 C2 4C 8D 85 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8B 85 ?? ?? ?? ?? 48 85 C0",
      ]
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
