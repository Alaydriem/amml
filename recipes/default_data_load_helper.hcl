entry "DefaultDataLoadHelper.vtable" {
  kind  = "global"
  notes = "The vtable of DefaultDataLoadHelper (a DataLoadHelper with no members, only the vptr; no RTTI). 18 slots on 26.63 (6aed1906-b127418e74d2f2f9): 0 the destructor, 1..3 loadPosition / loadBlockPosition / loadBlockPositionOffset (one folded body that copies 12 bytes from r8 to the out pointer in rdx), 4..7 the four rotation loaders (`movaps xmm0,xmm1; ret`), 8, 9, 11 and 12 the byte and enum loaders (`mov eax,edx; ret`), 10 loadDirection(Vec3) (the copy body), 13 and 14 loadActorUniqueID / loadOwnerID (`mov rax,rdx; mov [rdx],r8; ret`), 15 loadActorInternalComponentInfo, 16 shouldResetTime and 17 getType (`xor eax,eax; ret`: false and DataLoadHelperType::Default). Every slot returns its input unchanged, which is the DefaultDataLoadHelper behaviour; the structure helper (vtable 0xf1fc110, used by 0x88fb880) has real bodies. Code builds it on the stack (`lea rax,[vtable]; mov [rbp+..],rax`) and passes the address as the DataLoadHelper& of BlockActor.loadStatic; 27 code references load it. Recipe 0 is the code that stores the table in a stack object and then calls a virtual of the object that holds the loader: `lea rax,[rip+..]; mov [rbp+..],rax; mov rax,[rbx]; mov rax,[rax+disp32]; mov rcx,rbx; call [guard]`, offset 0, read as the RIP target of the `lea`. Recipe 1 is the same store followed by the same virtual call on another object in another function (`lea rax,[rip+..]; mov [rbp+..],rax; mov rax,[rdi]; mov rax,[rax+disp32]; mov rcx,rdi; call [guard]`), offset 0, read the same way. The two recipes are in two functions; they share the lea and the store, which are the shape of every user of this table, and no other bytes. Every displacement and RIP offset is a wildcard. The section check keeps the value in .rdata. Value 0xf0c7d50 on 26.63. What changed in 6afa4e47-dadc375797a3d31a (70.26): the function that recipe 0 used no longer tests a pointer before the store (`mov r14,[rdi+..]; test r14,r14; je` is gone), and the 26.63 function that recipe 1 used has no match on 70.26 (its argument handling changed); recipe 1 now reads another user of the table, which exists on both builds. The store and the virtual call after it are the same, and the new patterns start at the store. The table on 70.26 has the same content as on 26.63 (18 slots; slots 1..3 and 10 one body, 4..7 one body, 8, 9, 11, 12 one body, 13 and 14 one body, 16 and 17 one body, slot 0 the destructor): the two tables that have that content on 26.63 are told apart by slots 16 and 17 (the same body in the default table, two bodies in the other); 25 code references load the default table on 70.26 (27 on 26.63). Evidence on 70.26: each pattern, drafted from its 26.63 function, matches once on 70.26, and in both cases the lea loads the table with the 18-slot content above and slots 16 and 17 equal; the second function is another user of the table than the first. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 03 48 8B 80 ?? ?? ?? ?? 48 89 D9 FF 15"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15"
      offset = 0
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
