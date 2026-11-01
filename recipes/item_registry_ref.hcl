entry "ItemRegistryRef.getCreativeItemRegistry" {
  kind     = "function"
  notes    = "Bedrock::NonOwnerPointer<CreativeItemRegistry> ItemRegistryRef::getCreativeItemRegistry() (the non-const overload): rcx the ItemRegistryRef, rdx the 24-byte result buffer, returned in rax. It locks the weak_ptr (the inlined `ItemRegistryRef::_lockRegistry` assert), reads the creative registry pointer of the ItemRegistry (`mov rbx,[rax+430h]`), zeroes the result, and when the registry is set copies its control block (EnableNonOwnerReferences: the pointer at +8 and the shared count at +10h) and the raw pointer. Its callers use the result with the text `T *Bedrock::NonOwnerPointer<CreativeItemRegistry>::_get() const [T = CreativeItemRegistry]` (the non-const form); the const overload is a separate function with other callers. Recipe 0 is that copy (`mov rbx,[rax+..]; xorps xmm0,xmm0; movups [rsi],xmm0; mov qword ptr [rsi+..],0; test rbx,rbx; je; mov rax,[rbx+..]; test rax,rax; je; lock inc dword ptr [rax+..]`), then function_start_of. Recipe 1 is a caller that gets the registry from a level: `mov rax,[r15+..]; mov rcx,[rax+..]` (Actor::mLevel), the Level.getItemRegistry call, then `lea rdx,[rbp-..]; mov rcx,rdi; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 0 is now a pattern_any, one alternative per shape of the copy: alternative 1 (26.60) keeps the registry in r14 and uses a near je. The creative registry pointer is at ItemRegistry+430h on 26.51 and +428h on 26.60. Recipe 1 is now another level caller: the Level.getItemRegistry call into a local, then `lea rdx,[rbp+..]; mov rcx,local; call` (the same site as Level.getItemRegistry recipe 1), read as the call target. What changed in 6ad79ee4-17d51891a1119914 (26.60): register allocation of the copy and a near je; the 26.51 site of recipe 1 changed shape. The 26.60 function was accepted on two facts from the 26.51 function: the same body (lock of the weak_ptr, the registry pointer load, the zeroed 24-byte result, the control block copy), and the same caller (the 26.51 caller of the shared site calls it right after Level.getItemRegistry, and the 26.60 counterpart too; 16 and 15 callers). Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 98 ?? ?? ?? ?? 0F 57 C0 0F 11 06 48 C7 46 ?? ?? ?? ?? ?? 48 85 DB 74 ?? 48 8B 43 ?? 48 85 C0 0F 84 ?? ?? ?? ?? F0 FF 40 ??",
        "4C 8B B0 ?? ?? ?? ?? 0F 57 C0 0F 11 06 48 C7 46 ?? ?? ?? ?? ?? 4D 85 F6 0F 84 ?? ?? ?? ?? 49 8B 46 ?? 48 85 C0 0F 84",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D ?? ?? ?? ?? ?? 48 89 ?? FF 15 ?? ?? ?? ?? 48 8D 55 ?? 48 89 ?? E8"
      offset = 40
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
