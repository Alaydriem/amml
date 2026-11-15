entry "DefaultDataLoadHelper.vtable" {
  kind  = "global"
  notes = "The vtable of DefaultDataLoadHelper (a DataLoadHelper with no members, only the vptr; no RTTI). 18 slots on 26.63 (6aed1906-b127418e74d2f2f9): 0 the destructor, 1..3 loadPosition / loadBlockPosition / loadBlockPositionOffset (one folded body that copies 12 bytes from r8 to the out pointer in rdx), 4..7 the four rotation loaders (`movaps xmm0,xmm1; ret`), 8, 9, 11 and 12 the byte and enum loaders (`mov eax,edx; ret`), 10 loadDirection(Vec3) (the copy body), 13 and 14 loadActorUniqueID / loadOwnerID (`mov rax,rdx; mov [rdx],r8; ret`), 15 loadActorInternalComponentInfo, 16 shouldResetTime and 17 getType (`xor eax,eax; ret`: false and DataLoadHelperType::Default). Every slot returns its input unchanged, which is the DefaultDataLoadHelper behaviour; the structure helper (vtable 0xf1fc110, used by 0x88fb880) has real bodies. Code builds it on the stack (`lea rax,[vtable]; mov [rbp+..],rax`) and passes the address as the DataLoadHelper& of BlockActor.loadStatic; 27 code references load it. Recipe 0 is in 0x781b820, which calls BlockActor.loadStatic with this helper: `mov r14,[rdi+disp8]; test r14,r14; je rel32; lea rax,[rip+..]; mov [rbp+..],rax; mov rax,[rbx]; mov rax,[rax+disp32]; mov rcx,rbx; call [guard]`, offset 13. Recipe 1 is in 0x1fa4940, which also calls loadStatic with it: `test r14,r14; je rel32; test al,al; je rel32; lea rax,[rip+..]; mov [rbp+disp32],rax; lea rax,[rip+..]`, offset 17. Both read the RIP target of the `lea`. The two recipes are in two functions and share no bytes. Every displacement and branch byte is a wildcard. The section check keeps the value in .rdata. Value 0xf0c7d50 on 26.63."

  recipe {
    step "pattern" {
      bytes  = "4C 8B 77 ?? 4D 85 F6 0F 84 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 03 48 8B 80 ?? ?? ?? ?? 48 89 D9 FF 15"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 85 F6 0F 84 ?? ?? ?? ?? 84 C0 0F 84 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 05"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}
