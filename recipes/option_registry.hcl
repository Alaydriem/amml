entry "OptionRegistry.vtable" {
  kind     = "global"
  notes    = "The vtable that the OptionRegistry constructor stores at this+0: the IOptionRegistry part of the options object that IClientInstance::getOptions returns (IOptionRegistry is at offset 0). The exe has no RTTI for game classes. Found through std RTTI: `<lambda_2>` declared in the OptionRegistry constructor has a type descriptor, the `_Target_type` of its std::function impl returns it, and the impl's vtable is loaded by the OptionRegistry constructor (and its own copy function). The constructor first stores the base class vtables at this+0 and this+8, sets a bool, then stores the OptionRegistry vtables at this+0 and this+8. The pattern is `mov byte ptr [r14+member],imm; lea rax,[rip+..]; mov [r14],rax; lea rax,[rip+..]; mov [r14+8],rax; movups [r14+..],xmm0`; the operand is the RIP target of the first `lea`. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). It is unique in both exes. The section check keeps the address inside .rdata. It exists so `slot_of` has a vtable to read. Values: 0xe7fc490 in 26.51, 0xf276f30 in 26.60, 0xf277f10 in 26.63, 0x12d6ecb0 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed in 6ae4eeee-4dd4fa7acacb4c41: OptionRegistry gained two more bases, so the constructor stores four vtables at this+0, +8, +10h and +18h (twice: the bases, then OptionRegistry), the members moved by 70h, and the `mov byte ptr [r14+member],1` before the second run is gone; the old pattern (that byte store, then two stores and a `movups`) found no match. The recipe is now one `pattern_any` that starts earlier, at the copy of the shared_ptr argument after the first run (`cmp qword ptr [rdi+..],0; je; mov rax,[rdi]; mov [r14+..],rax; mov rax,[rdi+..]; mov [r14+..],rax; lock inc dword ptr [rax+..]`), so that the `lea` of the OptionRegistry vtable is 40 bytes in for both shapes: alternative 0 (26.51 to 26.63) is that copy, the byte store, two vtable stores and a `movups`; alternative 1 (the Preview) is the `movups` that clears the shared_ptr, the same copy, then the first two of the four vtable stores and the third `lea`. Alternative 0 matches once on 26.51, 26.60, 26.62 and 26.63 (the same `lea` as before) and not on the Preview; alternative 1 matches once on the Preview and on no other build. Evidence on the Preview, both from the 26.63 constructor and destructor: (1) the constructor shape (memset of the members, base vtables, the shared_ptr copy, then the OptionRegistry vtables followed by the same `movups`, `mov qword ptr [..],0Fh`, `mov byte ptr [..],0`, `mov [..],bl` member stores, 70h higher) is found once, and its first OptionRegistry `lea` gives the table; (2) the OptionRegistry destructor (the same prologue and `mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax` as on 26.63) stores the same table at this+0, and the setHideGUI and getHideHud bodies of 26.63 (the same option reads, ids one higher) sit in it at slots 92 and 93, two below the 26.63 slots like their neighbours. Would have survived with a better recipe (the bases changed, not the meaning of the table)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 83 7F ?? ?? 74 ?? 48 8B 07 49 89 86 ?? ?? ?? ?? 48 8B 47 ?? 49 89 86 ?? ?? ?? ?? F0 FF 40 ?? 41 C6 86 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 06 48 8D 05 ?? ?? ?? ?? 49 89 46 ?? 41 0F 11 86",
        "41 0F 11 86 ?? ?? ?? ?? 48 83 7F ?? ?? 74 ?? 48 8B 07 49 89 86 ?? ?? ?? ?? 48 8B 47 ?? 49 89 86 ?? ?? ?? ?? F0 FF 40 ?? 48 8D 05 ?? ?? ?? ?? 49 89 06 48 8D 05 ?? ?? ?? ?? 49 89 46 ?? 48 8D 05",
      ]
      offset = 40
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

entry "OptionRegistry.getEffectiveMaxFramerateOption" {
  kind     = "vslot"
  notes    = "Slot of Option* IOptionRegistry::getEffectiveMaxFramerateOption() (the non-const form) in the OptionRegistry.vtable global: 341 in 26.51, 339 in 26.60. The const form is the slot before it (340, 338); MSVC puts the const overload first. The function returns a gsl::not_null<Option*> through a hidden out pointer: rcx = the registry, rdx = the out pointer (8 bytes, the Option*); it returns rax = rdx. Body: a self virtual call (`call [rax+68h]`), `cmp al,2; sete r8b; or r8d,0F6h` (option id 0F6h or 0F7h), then `get(id)` (slot 27, vtable offset 0D8h) into the same out pointer. The const twin calls slot 1 (`mov rax,[rax+8]`, a disp8 form) at that place, so the disp32 form of the last `mov rax,[rax+disp]` tells the two apart without a kept byte. Recipe 0 is that body (`mov rsi,rdx; mov rdi,rcx; mov rax,[rcx]; mov rax,[rax+disp8]; call [guard]; xor r8d,r8d; cmp al,imm; sete r8b; or r8d,imm32; mov rax,[rdi]; mov rax,[rax+disp32]`), then `function_start_of` and `slot_of` OptionRegistry.vtable. Recipe 1 is a call site in a function that registers a value-changed listener on the option: `mov rsi,rcx; mov r14,[rcx]; mov rcx,[r14+member]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdi,rbp; mov rdx,rdi; call [guard]; mov rbx,[rbp+..]; mov rax,[rsi+..]; lea rcx,[rip+..]` (0x941f60 in 26.51, 0x65c5b60 in 26.60); it reads the displacement at offset 16 and turns it into a slot. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. The range check is a window around the two values. Values: slot 341 in 26.51 (0x2113e20), 339 in 26.60 (0x5b6cdc0). What changed in 26.60: the slot moved down by two, and the body ends with a tail jump (`jmp r9`) instead of a call."

  recipe {
    step "pattern" {
      bytes = "48 89 D6 48 89 CF 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 45 31 C0 3C ?? 41 0F 94 C0 41 81 C8 ?? ?? ?? ?? 48 8B 07 48 8B 80"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "OptionRegistry.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 4C 8B 31 49 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 EF 48 89 FA FF 15 ?? ?? ?? ?? 48 8B 5D ?? 48 8B 46 ?? 48 8D 0D"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 330
    max = 350
  }
}

entry "OptionRegistry.getIfValid" {
  kind     = "vslot"
  notes    = "Slot of std::optional<Option*> IOptionRegistry::getIfValid(OptionID) (the non-const form) in the OptionRegistry.vtable global: 29 in both builds. Slot 26 is isValidOption, 27 get(OptionID) (a not_null<Option*>), 28 the const getIfValid, 29 this one. The optional comes back through a hidden out pointer: rcx = the registry, rdx = the out pointer (16 bytes: the Option* at +0, the engaged flag at +8), r8d = the option id; it returns rax = rdx. Ids above 335h (26.51) or 338h (26.60) give an empty optional. Body: `isValidOption(id)` (slot 26), then `get(id)` (slot 27, vtable offset 0D8h) into the out pointer, and the flag. The const twin calls slot 1 (`mov rax,[rax+8]`, a disp8 form) at that place, so the disp32 form of that `mov` tells the two apart without a kept byte. Recipe 0 is that body (`mov edi,r8d; mov rsi,rdx; mov rbx,rcx; mov rax,[rcx]; mov rax,[rax+disp32]; mov edx,r8d; call [guard]; test al,al; je; mov rax,[rbx]; mov rax,[rax+disp32]; mov rcx,rbx; mov rdx,rsi; mov r8d,edi; call [guard]`), then `function_start_of` and `slot_of` OptionRegistry.vtable. Recipe 1 is a call site in a game function that sets an IntOption (IntOption.set) and then asks for another option: `mov r8b,imm; call <IntOption::set>; mov rax,[rsi]; mov rax,[rax+disp]; lea rdx,[rbp+..]; mov rcx,rsi; mov r8d,imm; call [guard]; cmp byte ptr [rbp+..],imm; jne rel32`. 26.60 puts a `nop` after each call and uses a 32-bit frame offset, so the recipe is a `pattern_any` with one alternative for each form; the 26.51 alternative starts with one wildcard so that the `mov rax,[rax+disp]` is at offset 12 in both. It reads that displacement and turns it into a slot. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. The range check is a window around slot 29. Values: slot 29 in 26.51 (0xfb4db0) and 26.60 (0x5af88a0). What changed in 26.60: nothing in this function."

  recipe {
    step "pattern" {
      bytes = "44 89 C7 48 89 D6 48 89 CB 48 8B 01 48 8B 80 ?? ?? ?? ?? 44 89 C2 FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 03 48 8B 80 ?? ?? ?? ?? 48 89 D9 48 89 F2 41 89 F8 FF 15"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "OptionRegistry.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? 41 B0 ?? E8 ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? 48 89 F1 41 B8 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 80 7D ?? ?? 0F 85",
        "41 B0 ?? E8 ?? ?? ?? ?? 90 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 48 89 F1 41 B8 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 80 BD ?? ?? ?? ?? ?? 0F 85",
      ]
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 24
    max = 34
  }
}
