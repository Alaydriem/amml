entry "ContainerManagerController.vtable" {
  kind  = "global"
  notes = "The primary vtable of the base class ContainerManagerController (no RTTI, 37 slots, 0..36, the same count as the virtuals in the LeviLamina header; the next table starts at slot 37), a helper for the slot_of steps of ContainerManagerController.handleSwap. Found from the chest manager: the ChestContainerManagerController constructor calls the base constructor and then stores its own table; the base constructor stores this table at this+0 right after it clears +8h..+17h, then copies the shared_ptr argument (rdx) to +18h/+20h. Slots 1..32 hold the same functions in the base table and the chest table (the chest table differs at slot 0, its destructor, and at slots 33, 34 and 36), which confirms that the two tables have one layout. Recipe 0 is that store in the base constructor (`xorps xmm0,xmm0; movups [rcx+..],xmm0; lea rax,[rip+..]; mov [rcx],rax; movups [rcx+..],xmm0; cmp qword ptr [rdx+..],0; je; mov rax,[rdx]; mov [rcx+..],rax; mov rax,[rdx+..]; mov [rcx+..],rax; lock inc dword ptr [rax+..]`), read as the RIP target of the lea. Recipe 1 is the head of the base destructor (`mov rbx,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rcx,[rcx+..]; test rcx,rcx; je; mov eax,[rcx+..]`, the weak_ptr lock of the member at +20h), read the same way. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard; the `[rcx]` stores keep their ModRM byte (a register, not a field). The check is section .rdata. Values: E880760h on 6aa482fd-2658118eba0dbac5 (26.51), F1B4550h on 6ad79ee4-17d51891a1119914 (26.60), F1B5540h on 6aea437e-7eff13daabb430da (26.62), F1B5520h on 6aed1906-b127418e74d2f2f9 (26.63). The recipes match once on all four builds without a change."

  recipe {
    step "pattern" {
      bytes  = "0F 57 C0 0F 11 41 ?? 48 8D 05 ?? ?? ?? ?? 48 89 01 0F 11 41 ?? 48 83 7A ?? ?? 74 ?? 48 8B 02 48 89 41 ?? 48 8B 42 ?? 48 89 41 ?? F0 FF 40 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CB 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 49 ?? 48 85 C9 0F 84 ?? ?? ?? ?? 8B 41 ??"
      offset = 3
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

entry "ContainerManagerController.handleSwap" {
  kind  = "vslot"
  notes = "Slot of `bool ContainerManagerController::handleSwap(SlotData const& slotA, SlotData const& slotB)` in the ContainerManagerController vtable: 21 on every build in the data. rcx the manager, rdx and r8 the two SlotData pointers (a SlotData is an MSVC std::string, 32 bytes, then an int index); the bool result is in al. One Swap request between two slots, without the cursor. The chest manager table does not override it, so a mod that reads slot 21 of the live manager's vtable gets this base body for a chest. Found from the LeviLamina header order (handleSwap is virtual index 21: the two handleSplitMultiple overloads are slots 18 and 19 in reverse order, as MSVC places overloads, and the header has 37 virtuals, the size of the table) and from the body: the function converts each SlotData argument with the same helper (`mov rdi,r8; mov rbx,rdx; mov rsi,rcx; ...; mov r8,rbx; call <helper>; ...; mov r8,rdi; call <helper>`), and a subclass override (the LeviLamina headers list a handleSwap override only for MaterialReducerContainerManagerController) compares the collection name of each of the two SlotData arguments with one container name and then calls this function with both arguments unchanged. Recipe 0 is the head of the body (`mov rdi,r8; mov rbx,rdx; mov rsi,rcx; cmp qword ptr [rcx+..],0; je; lea r14,[rbp+..]; mov rcx,rsi; mov rdx,r14; mov r8,rbx; call`), then function_start_of and slot_of ContainerManagerController.vtable. Recipe 1 is the prologue of that subclass override (`push rsi; push rdi; push rbx; sub rsp,..; mov rsi,r8; mov rdi,rdx; mov rbx,rcx; mov rax,[rip+..]` (the stack cookie) `; xor rax,rsp; mov [rsp+..],rax; mov byte ptr [rsp+..],..; lea rcx,[rip+..]; lea rdx,[rsp+..]; lea r8,[rsp+..]; call`), then function_start_of, the target of its direct call 4 (calls 0 to 3 are the two name lookups and the two string compares; call 4 passes rcx, rdx and r8 on unchanged), then slot_of. It depends on the order of the direct calls in the override; a change there makes it fail or give another function, which slot_of then refuses or the other recipes contradict. The override's tail call cannot be a pattern on its own: the handlePlaceOne override of the same class has the same compare-and-call bytes. Recipe 2 is a second part of the body, after the first test (`test al,al; je; xor esi,esi; jmp; lea rdx,[rbp+..]; mov rcx,rsi; mov r8,rbx; call`), then function_start_of and slot_of. Recipes 0 and 2 share no pattern bytes; recipe 1 is in another function. No virtual call site of slot 21 was found (`call [rax+0A8h]` after a load of the screen controller's manager at +12C0h matches nothing), so no recipe reads a displacement. Every displacement, immediate and branch byte is a wildcard. The check is a range window 17..25 (about 4 slots either side, inside the 37-slot interface). Value 21 (15h) on 6aa482fd-2658118eba0dbac5 (26.51), 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63); the base table has the same 37-slot layout on all four, so no build gate is needed."

  recipe {
    step "pattern" { bytes = "4C 89 C7 48 89 D3 48 89 CE 48 83 B9 ?? ?? ?? ?? ?? 74 ?? 4C 8D 75 ?? 48 89 F1 4C 89 F2 49 89 D8 E8 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ContainerManagerController.vtable" }
  }

  recipe {
    step "pattern" { bytes = "56 57 53 48 83 EC ?? 4C 89 C6 48 89 D7 48 89 CB 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? C6 44 24 ?? ?? 48 8D 0D ?? ?? ?? ?? 48 8D 54 24 ?? 4C 8D 44 24 ?? E8 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "call" { nth = 4 }
    step "slot_of" { vtable = "ContainerManagerController.vtable" }
  }

  recipe {
    step "pattern" { bytes = "84 C0 74 ?? 31 F6 E9 ?? ?? ?? ?? 48 8D 55 ?? 48 89 F1 49 89 D8 E8 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ContainerManagerController.vtable" }
  }

  check "range" {
    min = 17
    max = 25
  }
}
