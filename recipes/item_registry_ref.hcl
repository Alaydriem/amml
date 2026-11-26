entry "ItemRegistryRef.getCreativeItemRegistry" {
  kind     = "function"
  notes    = "Bedrock::NonOwnerPointer<CreativeItemRegistry> ItemRegistryRef::getCreativeItemRegistry() (the non-const overload): rcx the ItemRegistryRef, rdx the 24-byte result buffer, returned in rax. The signature is the same on 26.63 and 70.26. It locks the weak_ptr (ItemRegistryRef::_lockRegistry), reads the creative registry pointer of the ItemRegistry (`mov reg,[registry+428h]` on 26.63 and 70.26, 430h on 26.51), zeroes the result, and when the registry is set copies its control block (EnableNonOwnerReferences: the pointer at +8 and the shared count at +10h) and stores the raw pointer at +10h. About 15 callers on each build. Recipe 0 is the registry load and the zeroed result, then function_start_of; a pattern_any with one alternative per shape: alternative 0 (26.51) and 1 (26.60 to 26.63) are the inline control block copy (`mov reg,[rax+..]; xorps xmm0,xmm0; movups [rsi],xmm0; mov qword ptr [rsi+..],0; test reg,reg; je; mov rax,[reg+..]; test rax,rax; je`), alternative 2 (70.26) is `mov rsi,[rdi+..]; xorps xmm0,xmm0; movups [rbx],xmm0; mov qword ptr [rbx+..],0; mov rcx,rbx; mov rdx,rsi; mov [rbp+..],rbx; call` (the copy is a call of the NonOwnerPointer constructor; the pattern ends at the call). Each alternative matches on its own builds only. Recipe 1 is a call site that has the same shape on 26.63 and 70.26, in the item transaction code: `movzx eax,word ptr [reg+0AAh]; mov [rbp+..],ax; mov rcx,[rbp+..]; lea rdx,[rbp+..]; call`, read as the call target. The low byte AAh of the movzx displacement is kept: another 26.63 function has the same five instructions with a frame read (`movzx eax,word ptr [rbp+136h]`). The two recipes are in two functions. Every other displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 can fail it). No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a: the weak_ptr lock got an out-of-line assert and the control block copy became a call, so alternatives 0 and 1 matched nothing; the old recipe 1 call site (an Actor::mLevel load, the Level getItemRegistry virtual call, then this call) now has a nop after the virtual call. Class: would have survived with a better recipe. Evidence on 70.26, from the 26.63 function: (1) the 26.63 caller with the recipe 1 shape (the AAh word copy, then the call with the result buffer in rdx) is found once on 70.26 with the same five instructions, and it calls this function; (2) the 26.63 body (weak_ptr lock of [rcx+8], the creative registry load at 428h from the locked ItemRegistry, the zeroed 24-byte result, the raw pointer stored at +10h only when a control block was copied, the weak_ptr release) is in this function in the same order with the same 428h offset, and the callers that get the ItemRegistryRef from the Level getItemRegistry slot (Level.getItemRegistry, C40h on 26.63 and C58h on 70.26) pass it to this function in rcx on both builds (15 call sites on each)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 98 ?? ?? ?? ?? 0F 57 C0 0F 11 06 48 C7 46 ?? ?? ?? ?? ?? 48 85 DB 74 ?? 48 8B 43 ?? 48 85 C0 0F 84 ?? ?? ?? ?? F0 FF 40 ??",
        "4C 8B B0 ?? ?? ?? ?? 0F 57 C0 0F 11 06 48 C7 46 ?? ?? ?? ?? ?? 4D 85 F6 0F 84 ?? ?? ?? ?? 49 8B 46 ?? 48 85 C0 0F 84",
        "?? 8B ?? ?? ?? ?? ?? 0F 57 C0 0F 11 ?? 48 C7 ?? ?? ?? ?? ?? ?? 48 89 D9 48 89 F2 48 89 5D ?? E8",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "0F B7 ?? AA ?? ?? ?? 66 89 85 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8"
      offset     = 28
      literal_ok = ["3: movzx eax,word ptr [reg+0AAh]: the member read tells this site from a 26.63 function with the same instructions and a frame read (26.63, 70.26)"]
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
