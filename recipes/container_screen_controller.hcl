entry "ContainerScreenController.vtable" {
  kind     = "global"
  notes    = "The primary vtable of ContainerScreenController (no RTTI, 77 slots, 0..76; the secondary base table sits at +0BC8h of the object), a helper for the in_vtable windows of the hover callbacks and for the slot of the visual item stack. The class is abstract on 26.60: slots 72 and 73 are the pure-virtual stub. Found from the constructor, the one function that stores the std::function tables of the lambdas whose type names say `ContainerScreenController::ContainerScreenController(std::shared_ptr<ClientInstanceScreenModel>, InteractionModel)`. After the base constructor call it stores this table at [rsi] and the second base table at [rsi+0BC8h], then clears the members at +0D60h..+0D7Ch. Recipe 0 is that run: `call <base ctor>; nop; lea rax,[rip+..]; mov [rsi],rax; lea rax,[rip+..]; mov [rsi+..],rax; mov qword ptr [rsi+..],0; mov dword ptr [rsi+..],0; mov qword ptr [rsi+..],-1`, read as the RIP target of the first lea. Recipe 1 is the head of the destructor: `mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; lea rax,[rip+..]; mov [rcx+..],rax; mov rax,[rcx+..]; mov rcx,[rax+..]; test rcx,rcx; je; cmp byte ptr [rcx],0; jne; mov dword ptr [0],..; mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`, read as the RIP target of the first lea. The destructor of a sibling controller has the same head and reads the virtual at +568h where this one reads +0F0h, so that one byte is kept (literal_ok). The two recipes are in two functions. Every other displacement, immediate and branch byte is a wildcard. The check is section .rdata. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value F221EF0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 90 48 8D 05 ?? ?? ?? ?? 48 89 06 48 8D 05 ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ??"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 05 ?? ?? ?? ?? 48 89 81 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 0F 84 ?? ?? ?? ?? 80 39 ?? 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 48 ?? 48 8B 01 48 8B 80 F0 ?? ?? ?? FF 15"
      literal_ok = [
        "73: low byte of the virtual at +0F0h: the destructor of a sibling controller has the same head and calls the virtual at +568h",
      ]
      offset     = 3
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ContainerScreenController.onContainerSlotHovered" {
  kind     = "function"
  hooked   = true
  notes    = "ui::ViewRequest ContainerScreenController::_onContainerSlotHovered(std::string const& collectionName, int slot): rcx the controller, rdx the std::string pointer, r8d the slot; the return value (an enum of unsigned int) is in eax, with no hidden pointer. Slot 60 of ContainerScreenController.vtable and of the CraftingScreenController table. It stores the hover (the string into the controller at +1098h, the slot at +10B8h, as its sibling _onContainerSlotUnhovered clears them). CraftingScreenController overrides the slot with a function that ends in a direct call to this one, as do the other screen controllers that have a hover override, so a hook on this function sees the inventory, chest and crafting hovers. Recipe 0 is the prologue and the first tests of the body: `push r15; push r14; push rsi; push rdi; push rbx; sub rsp,..; mov esi,r8d; mov rdi,rdx; mov rbx,rcx; cmp byte ptr [rcx+..],1; jne; mov byte ptr [rbx+..],0; jmp; cmp dword ptr [rbx+..],3; jne; cmp byte ptr [rbx+..],0`. Recipe 1 is the call site in the CraftingScreenController override: `call <free>; mov rcx,rbx; mov rdx,rdi; mov r8d,esi; call <this>`, read as the call target. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and in_vtable on ContainerScreenController.vtable, window 56..64 (the neighbours _handleTakePlace at 56 and _onContainerSlotPressed at 63 are inside it, the slot is 60). The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 500D460h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "41 57 41 56 56 57 53 48 83 EC ?? 44 89 C6 48 89 D7 48 89 CB 80 B9 ?? ?? ?? ?? ?? 75 ?? C6 83 ?? ?? ?? ?? ?? EB ?? 83 BB ?? ?? ?? ?? ?? 75 ?? 80 BB ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 48 89 D9 48 89 FA 41 89 F0 E8 ?? ?? ?? ??"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ContainerScreenController.vtable"
    min_slot = 56
    max_slot = 64
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ContainerScreenController.onContainerSlotUnhovered" {
  kind     = "function"
  hooked   = true
  notes    = "ui::ViewRequest ContainerScreenController::_onContainerSlotUnhovered(std::string const& collectionName, int slot): rcx the controller, rdx the std::string pointer, r8d the slot; the enum return is in eax (no hidden pointer), as for _onContainerSlotHovered. Slot 61 of ContainerScreenController.vtable (CraftingScreenController does not override it). It builds a temporary std::string, compares the controller's stored hover (+1098h, slot at +10B8h) with the arguments, and when they match and the interaction model at +1230h is 2 it clears the stored hover (`mov qword ptr [rbx+..],0; mov dword ptr [rbx+..],-1`, the same for the other stored strings) and sets +118Ah. It has no direct caller: it is only reached through the table. Recipe 0 is the prologue and the first calls: `push rbp; push r15; push r14; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov esi,r8d; mov rdi,rdx; mov rbx,rcx; lea r14,[rcx+..]; lea r15,[rbp+..]; mov rcx,r15; call`. Recipe 1 is the block that clears the stored hover: `cmp dword ptr [rbx+..],2; jne; mov qword ptr [rbx+..],0; cmp qword ptr [rbx+..],10h; jb; mov r14,[rbx+..]; mov byte ptr [r14],0; mov dword ptr [rbx+..],-1`, then function_start_of. The two anchors are in different parts of the body and share no pattern bytes; the function has no caller, so no call-site recipe exists, and the in_vtable window is the independent fact. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and in_vtable on ContainerScreenController.vtable, window 57..65. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 500D6B0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 44 89 C6 48 89 D7 48 89 CB 4C 8D B1 ?? ?? ?? ?? 4C 8D 7D ?? 4C 89 F9 E8 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "83 BB ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 C7 83 ?? ?? ?? ?? ?? ?? ?? ?? 48 83 BB ?? ?? ?? ?? ?? 72 ?? 4C 8B B3 ?? ?? ?? ?? 41 C6 06 ?? C7 83 ?? ?? ?? ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ContainerScreenController.vtable"
    min_slot = 57
    max_slot = 65
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ContainerScreenController.getVisualItemStackImpl" {
  kind     = "vslot"
  notes    = "Slot of ItemStackBase const& ContainerScreenController::_getVisualItemStackImpl(std::string const& collectionName, int slot) in the ContainerScreenController vtable: 71 on 26.60. rcx the controller, rdx the std::string pointer, r8d the slot; the reference comes back in rax and is never null (the static empty stack when nothing is there). The container slot is a 5-byte `jmp` thunk to the function that builds a SlotData, calls _reevaluateSlotData (slot 59) and asks the container manager controller (+12C0h); the thunk has no .pdata, and the CraftingScreenController overrides the slot with its own function. No direct caller is known, and the thunk cannot be matched by its bytes, so both recipes are neighbour routes: the slot is read from the functions on either side of it in the table, and the header order puts the slot between them (function in slot 70, then this slot 71, then the two pure virtuals 72 and 73, then the function in slot 74). Recipe 0 is the body of the function in slot 70 (`push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; cmp qword ptr [rcx+..],..`), then function_start_of, slot_of ContainerScreenController.vtable and add 1. Recipe 1 is the prologue of the function in slot 74 (`push rbp; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; xorps xmm0,xmm0; movaps [rbp+..],xmm0`), then slot_of and add -3. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 71 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 66 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 83 B9 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ContainerScreenController.vtable" }
    step "add" { value = 1 }
  }

  recipe {
    step "pattern" { bytes = "55 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 0F 57 C0 0F 29 45 ?? 48 C7 45 ?? ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ContainerScreenController.vtable" }
    step "add" { value = -3 }
  }

  check "range" {
    min = 40
    max = 76
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ContainerScreenController._handleAutoPlace" {
  kind     = "function"
  notes    = "void ContainerScreenController::_handleAutoPlace(int amount, std::string const& collectionName, int index): rcx the controller, edx the amount (7FFFFFFFh the whole stack, 1 one item), r8 the std::string pointer, r9d the index. Non-virtual. It returns at once when the virtual at +138h (slot 39) is false, builds a SlotData from the name and index, asks the container manager controller (+12C0h) for the stack, takes a separate path on interaction mode 1 (+1230h), and ends in the auto-place call of the manager. Found from the ContainerScreenController constructor: the button hash 7932D52Ah registers the state 11 and state 12 actions, two std::function lambdas that load the controller from [rcx+8] and call this function with edx = 7FFFFFFFh (state 11) and edx = 1 (state 12). The prologue moves the arguments in the int, pointer, int order of the header. Recipe 0 is the head of the body: `mov esi,r9d; mov rdi,r8; mov r14d,edx; mov rbx,rcx; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`, then function_start_of. Recipe 1 is the call site in the state 12 (one item) lambda: `push r14; push rsi; push rdi; push rbx; sub rsp,..; mov rsi,r8; mov rdi,[rcx+..]; mov ebx,[r9]; mov rcx,rdi; mov edx,..; mov r9d,ebx; call <this>`, read as the call target. The state 11 lambda has the same body after a shorter push list (push rsi; push rdi; push rbx), so the leading push r14 makes the match unique without keeping the amount immediate. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The code is the same on all four builds, so the entry has no gate. Values: 4DFB400h on 6aa482fd-2658118eba0dbac5 (26.51), 500AFB0h on 6ad79ee4-17d51891a1119914 (26.60), 500B320h on 6aea437e-7eff13daabb430da (26.62), 500B080h on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" { bytes = "44 89 CE 4C 89 C7 41 89 D6 48 89 CB 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "41 56 56 57 53 48 83 EC ?? 4C 89 C6 48 8B 79 ?? 41 8B 19 48 89 F9 BA ?? ?? ?? ?? 41 89 D9 E8 ?? ?? ?? ??"
      offset = 30
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ContainerScreenController._handleTakeAll" {
  kind     = "function"
  notes    = "void ContainerScreenController::_handleTakeAll(std::string const& collectionName, int index): rcx the controller, rdx the std::string pointer, r8d the index. Non-virtual. It puts the whole stack of the slot on the cursor: it returns at once when the virtual at +138h (slot 39) is false, flushes the pending coalesce list when a selected slot is stored (+1058h string length, +1068h index), clears the stored selection, builds a SlotData and calls the container manager controller (+12C0h) at its slot 10 (+50h). It is the only screen handler that uses that manager slot. Found from that manager call and from its callers: a std::function lambda that loads the controller from [rcx+8] and calls it with the name and index, and a second lambda that calls it and then stores the name into the controller. Recipe 0 is the selected-slot test of the body: `cmp qword ptr [rsi+..],0; setne al; mov r9d,[rsi+..]; cmp r9d,-1; setne cl; and cl,al; cmp cl,1; jne`, then function_start_of. Recipe 1 is the call site in the second lambda: `mov ebp,[rdx]; mov edi,[r9]; mov rbx,[rcx+..]; mov rcx,rbx; mov rdx,r8; mov r8d,edi; call <this>`, read as the call target. Recipe 2 is the call site in the first lambda, a small function: `sub rsp,..; mov rdx,r8; mov rcx,[rcx+..]; mov r8d,[r9]; call <this>`, read as the call target. Its pattern is short and other lambdas have the same shape, so on a new build it is the first to become ambiguous (a refusal of that recipe, not a wrong value). The three recipes are in three functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The code is the same on all four builds, so the entry has no gate. Values: 4DFA750h on 6aa482fd-2658118eba0dbac5 (26.51), 500A240h on 6ad79ee4-17d51891a1119914 (26.60), 500A5B0h on 6aea437e-7eff13daabb430da (26.62), 500A310h on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" { bytes = "48 83 BE ?? ?? ?? ?? ?? 0F 95 C0 44 8B 8E ?? ?? ?? ?? 41 83 F9 ?? 0F 95 C1 20 C1 80 F9 ?? 75 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "8B 2A 41 8B 39 48 8B 59 ?? 48 89 D9 4C 89 C2 41 89 F8 E8 ?? ?? ?? ??"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 4C 89 C2 48 8B 49 ?? 45 8B 01 E8 ?? ?? ?? ??"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ContainerScreenController._handlePlaceAll" {
  kind     = "vslot"
  notes    = "Slot of void ContainerScreenController::_handlePlaceAll(std::string const& collectionName, int index) in the ContainerScreenController vtable: 54 on 26.60, 26.62 and 26.63. rcx the controller, rdx the std::string pointer, r8d the index. It puts the cursor stack into the slot (into an empty slot, onto a matching stack, or a swap): it returns at once when the virtual at +138h (slot 39) is false, flushes the pending coalesce list, reads the selected slot through the virtual at +1D0h (slot 58, _getSelectedSlotInfo) and calls the container manager controller (+12C0h) at its slot 11 (+58h). The header order puts it second among the own virtuals of the class: _handleTakeHalf 53, _handlePlaceAll 54, _handlePlaceOne 55, _handleTakePlace 56. The table on these builds agrees: the slot 56 function is _handleTakePlace (it makes the two calls of recipe 0) and slot 60 is _onContainerSlotHovered. Recipe 0 is the call site in _handleTakePlace (slot 56): it tests its bool allItems and calls [vt+1B0h] when it is true, [vt+1B8h] (_handlePlaceOne) when it is false: `mov rax,[rdi]; test r14b,r14b; je; mov rax,[rax+..]; mov rcx,rdi; mov rdx,rbx; mov r8d,esi; call [guard]`, read as the displacement, then displacement_to_slot. The je before the load picks the allItems branch, so the pattern cannot match the _handlePlaceOne call. Recipe 1 is the body of the function: `cmp dword ptr [rsi+..],2; jne; mov r14,[rbp+..]; test r14,r14; je; lea r15,[rip+..]; mov [rbp+..],r15; mov qword ptr [rbp+..],13h`, then function_start_of and slot_of ContainerScreenController.vtable. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range window 50..58 around the known slot. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the layout of ContainerScreenController.vtable, which recipe 1 needs: it refuses as not for this build on 26.51. On 26.51 the call site of recipe 0 also reads +1B0h (slot 54), but the vtable entry does not resolve there, so no second recipe could agree. Values: 54 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 07 45 84 F6 74 ?? 48 8B 80 ?? ?? ?? ?? 48 89 F9 48 89 DA 41 89 F0 FF 15"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" { bytes = "83 BE ?? ?? ?? ?? ?? 75 ?? 4C 8B 75 ?? 4D 85 F6 74 ?? 4C 8D 3D ?? ?? ?? ?? 4C 89 7D ?? 48 C7 45 ?? ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ContainerScreenController.vtable" }
  }

  check "range" {
    min = 50
    max = 58
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ContainerScreenController._handleDropItem" {
  kind     = "function"
  notes    = "void ContainerScreenController::_handleDropItem(std::string const& collectionName, int collectionIndex, ItemTransferAmount transferAmount): rcx the controller, rdx the std::string pointer, r8d the index, r9 a pointer to a caller-owned copy of the 12-byte ItemTransferAmount (MSVC passes a by-value struct over 8 bytes by address; the function copies [r9] and [r9+8] to its frame). No return value. Not virtual. Found from the two input actions of the container state machine that drop an item (on 26.63 the state 9 action 505A0B0h and the state 10 action 505A1B0h, the only two direct callers): each asks the selected slot, then calls _handleDropSelectedItem(controller, &amount) on one branch and this function (controller, name, index, &amount) on the other, the arities of the two header declarations. The body builds a SlotData from the name and the index, loads the container manager controller from [this+12C0h] and calls its virtual at +0B0h (slot 22, handleDrop) with the SlotData and the copied amount, then clears the stored selection strings. Recipe 0 is that block: `mov r15,[rsi+..]; mov eax,[r14+..]; mov [rbp+..],eax; mov rax,[r14]; mov [rbp+..],rax; lea r14,[rbp+..]; mov rcx,r14; mov rdx,rbx; mov r8d,edi; call`, then function_start_of. Recipe 1 is the call site in the state 9 action: `mov qword ptr [rsp+..],imm32; mov byte ptr [rsp+..],0; lea r9,[rsp+..]; mov rcx,rdi; mov rdx,rsi; mov r8d,ebx; call <this>`, read as the call target; the state 10 action builds the amount with `mov rax,imm64; mov [rsp+..],rax` and does not match, so the match is unique. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end in function_start_of, so the check counts). The code is the same on all four builds, so no entry_is gate. Value 4DFC030h on 6aa482fd-2658118eba0dbac5 (26.51), 500BC40h on 6ad79ee4-17d51891a1119914 (26.60), 500BFB0h on 6aea437e-7eff13daabb430da (26.62), 500BD10h on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" { bytes = "4C 8B BE ?? ?? ?? ?? 41 8B 46 ?? 89 45 ?? 49 8B 06 48 89 45 ?? 4C 8D 75 ?? 4C 89 F1 48 89 DA 41 89 F8 E8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 C7 44 24 ?? ?? ?? ?? ?? C6 44 24 ?? ?? 4C 8D 4C 24 ?? 48 89 F9 48 89 F2 41 89 D8 E8 ?? ?? ?? ??"
      offset = 28
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ContainerScreenController._handleDropSelectedItem" {
  kind     = "function"
  notes    = "void ContainerScreenController::_handleDropSelectedItem(ItemTransferAmount transferAmount): rcx the controller, rdx a pointer to a caller-owned copy of the 12-byte ItemTransferAmount (MSVC passes a by-value struct over 8 bytes by address). No return value. Not virtual. It drops the stack of the selected slot (the cursor). Found from the same two drop actions as _handleDropItem: the branch with one argument calls this function. The body gets the selected slot with the virtual at +1D0h (slot 58, _getSelectedSlotInfo, hidden return pointer), loads the container manager controller from [this+12C0h], builds the amount (the selected count when it is above 0, else a copy of [rdx]) and calls the manager virtual at +0B0h (slot 22, handleDrop), then clears the stored selection. Recipe 0 is the block after the selected-slot call up to the manager call: `mov eax,[rsp+..]; mov rcx,[rsi+..]; test eax,eax; jle; mov dword ptr [rsp+..],0; mov byte ptr [rsp+..],0; mov [rsp+..],eax; lea r8,[rsp+..]; jmp; mov eax,[rdi+..]; mov [rsp+..],eax; mov rax,[rdi]; mov [rsp+..],rax; lea r8,[rsp+..]; mov rdx,[rsp+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`, then function_start_of. Recipe 1 is the call site in the state 9 action: `je; mov qword ptr [rsp+..],imm32; mov byte ptr [rsp+..],0; lea rdx,[rsp+..]; mov rcx,rdi; call <this>`, read as the call target (26.51 has an inlined test before the je, so the pattern starts at the je; the state 10 action uses `mov rax,imm64` and the third caller uses rbp frame slots, so neither matches). The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end in function_start_of, so the check counts). The code is the same on all four builds, so no entry_is gate. Value 4DFBC50h on 6aa482fd-2658118eba0dbac5 (26.51), 500B9C0h on 6ad79ee4-17d51891a1119914 (26.60), 500BD30h on 6aea437e-7eff13daabb430da (26.62), 500BA90h on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" { bytes = "8B 44 24 ?? 48 8B 8E ?? ?? ?? ?? 85 C0 7E ?? C7 44 24 ?? ?? ?? ?? ?? C6 44 24 ?? ?? 89 44 24 ?? 4C 8D 44 24 ?? EB ?? 8B 47 ?? 89 44 24 ?? 48 8B 07 48 89 44 24 ?? 4C 8D 44 24 ?? 48 8B 54 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "74 ?? 48 C7 44 24 ?? ?? ?? ?? ?? C6 44 24 ?? ?? 48 8D 54 24 ?? 48 89 F9 E8 ?? ?? ?? ??"
      offset = 24
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ContainerScreenController.mContainerManagerController" {
  kind     = "field"
  notes    = "Offset of mContainerManagerController in ContainerScreenController: a std::shared_ptr<ContainerManagerController> (16 bytes); the object pointer is at the offset, the control block 8 bytes later. 12C0h on all four builds. The code loads the ContainerManagerController* from [this+offset] and makes virtual calls on it (+0B0h, handleDrop, with a SlotData and an ItemTransferAmount); the 26.60 notes of ContainerScreenController.getVisualItemStackImpl name the same +12C0h load in the visual item stack function. The LeviLamina header puts the member after mInteractionModel. Recipe 0 is in _handleDropItem: `mov r15,[rsi+..]; mov eax,[r14+..]; ...; mov r8d,edi; call`, the displacement of operand 1 of the first instruction. Recipe 1 is in _handleDropSelectedItem: `mov eax,[rsp+..]; mov rcx,[rsi+..]; test eax,eax; jle`, the displacement of operand 1 of the second instruction. Each recipe starts with a ref to its function entry and scans only that function, so the field depends on both function entries (when one function entry fails, its recipe fails and the field rests on the other recipe and the range check). The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is range 1000h..1600h. Value 12C0h on 6aa482fd-2658118eba0dbac5 (26.51), 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "ref" { entry = "ContainerScreenController._handleDropItem" }
    step "pattern" { bytes = "4C 8B BE ?? ?? ?? ?? 41 8B 46 ?? 89 45 ?? 49 8B 06 48 89 45 ?? 4C 8D 75 ?? 4C 89 F1 48 89 DA 41 89 F8 E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ContainerScreenController._handleDropSelectedItem" }
    step "pattern" {
      bytes  = "8B 44 24 ?? 48 8B 8E ?? ?? ?? ?? 85 C0 7E ??"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4096
    max = 5632
  }
}
