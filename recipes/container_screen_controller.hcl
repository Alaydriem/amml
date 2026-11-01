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
