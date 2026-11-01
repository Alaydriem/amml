entry "ItemStack.emptyCtor" {
  kind     = "function"
  notes    = "The function the game calls to make an empty ItemStack: rcx the 152-byte stack, returned in rax. On 26.51 it is ItemStack::ItemStack(), the default constructor: it stores the ItemStackBase vtable, zeroes the base members inline (mItem, mUserData, mBlock, the three bool bytes set to 1, 1, 0, the two vectors and mChargedItem), calls the base setNull with an empty std::optional<std::string>, then stores the ItemStack vtable and an empty mNetIdVariant (`mov dword ptr [rax+80h],0; mov byte ptr [rax+90h],0`). On 26.60 ItemStack::ItemStack() is not a function any more: the callers call ItemStackBase::ItemStackBase() (the same base part: the ItemStackBase vtable, the same member init and setNull) and then store the ItemStack vtable and the empty net id variant inline. The entry is the function the callers call on each build. After the call the mod writes the three stores that the 26.60 callers write inline (at the setItemWithForceBalance site: `lea rsi,[rip+ItemStack vtable]; mov [rbp+2A8h],rsi; mov dword ptr [rbp+328h],0; mov byte ptr [rbp+338h],0`, the stack at rbp+2A8h, so +0, +80h and +90h); the 26.51 constructor ends with the same three stores (`mov [rax],rcx` of the ItemStack vtable; `mov dword ptr [rax+80h],0; mov byte ptr [rax+90h],0`), so on 26.51 they write the bytes that are already there. Both recipes are call sites that match on both builds, read as the call target: recipe 0 is the call in Inventory::setItemWithForceBalance (the function that holds the lambda named `setItemWithForceBalance@Inventory`): `movaps [rbp+..],xmm0; mov dword ptr [rbp+..],..; lea rcx,[rbp+..]; call`; recipe 1 is a call on a stack member of an object in r15: `mov dword ptr [r15+..],..; mov rcx,r15; add rcx,..; mov [rbp+..],r15; call`. Recipe 1 was found by drafting each 26.60 call of the base constructor and keeping one whose pattern matches once on each build and calls the 26.51 default constructor there. Every displacement, immediate and branch byte is a wildcard. The check is function_start. This entry replaces ItemStack.ctor: on 6ad79ee4-17d51891a1119914 (26.60) the cold derive resolved ItemStack.ctor from its call-site recipe alone to ItemStackBase::ItemStackBase() (its body recipe found no match), a function that is not ItemStack::ItemStack(); the mod would have built a stack with the ItemStackBase vtable. Real code change (the constructor is inlined)."

  recipe {
    step "pattern" {
      bytes  = "0F 29 85 ?? ?? ?? ?? C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8"
      offset = 24
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "41 C7 47 ?? ?? ?? ?? ?? 4C 89 F9 48 83 C1 ?? 4C 89 BD ?? ?? ?? ?? E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ItemStack.copyCtor" {
  kind     = "function"
  notes    = "ItemStack::ItemStack(ItemStack const&), the copy constructor: rcx the new stack, rdx the source, returned in rax. It calls the ItemStackBase copy constructor, stores the ItemStack vtable, clears mNetIdVariant and copies the source variant through the variant copy helper (`movsx rcx,byte ptr [rsi+90h]; sub rsi,-80h; inc rcx; ...; call`). Inlined copies of the same code appear in FillingContainer::swapSlots; a second out-of-line function with the source and destination registers swapped is a different constructor (the move form) with other callers. Recipe 0 is the head of the body: `mov rsi,rdx; mov rdi,rcx; call; lea rax,[rip+..]; mov [rdi],rax; lea rax,[rdi+..]; mov dword ptr [rdi+..],0; mov byte ptr [rdi+..],0`, then function_start_of. Recipe 1 is a caller that copies a stack into a local when a flag is clear: `test r12b,r12b; jne; lea rcx,[rbp-..]; mov rdx,rbx; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. A second route existed, so the ctor-plus-assignment fallback of the plan is not needed. Recipe 1 is now another call site: a guard call returning a bool, then `test al,al; je; lea rcx,[rbp+..]; mov rdx,r12; call` (it runs past the indirect call; no nop on either build). What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site of recipe 1 changed shape; the new site was found by drafting each 26.60 call of this function and keeping one whose pattern matches once on each build and calls the 26.51 value there. Recipe 0 agrees; the body still stores the ItemStack vtable after the base copy on 26.60. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 D6 48 89 CF E8 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 07 48 8D 87 ?? ?? ?? ?? C7 87 ?? ?? ?? ?? ?? ?? ?? ?? C6 87 ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8D 4D ?? 4C 89 E2 E8"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ItemStack.vtable" {
  kind     = "global"
  notes    = "The vtable of ItemStack (no RTTI), a helper for ItemStack.reinit. 7 slots: the deleting destructor, the three reinit overloads, setNull, toString and toDebugString (the ItemStack header overrides). Recipe 0 is the store in ItemStack.ctor (`lea rcx,[rip+..]; mov rax,[rbp-..]; mov [rax],rcx`). Recipe 1 is the store in ItemStack.copyCtor (`lea rax,[rip+..]; mov [rdi],rax`). Each reads the RIP target of the lea; the two recipes are in two functions. The RIP offsets are wildcards. Recipe 0 is now the reinit(Item const&, int, int) body (the same pattern_any as ItemStack.reinit recipe 0): after the temporary is built it stores this table (`lea rbx,[rip+..]; mov [rbp+..],rbx`, offset 24), read as the RIP target. What changed in 6ad79ee4-17d51891a1119914 (26.60): the default constructor that recipe 0 read is not a function any more (the callers call the ItemStackBase constructor and store this table inline); recipe 1 (the copy constructor) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 CE 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 7D ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 1D ?? ?? ?? ?? 48 89 5D ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 89 F1 48 89 FA E8 ?? ?? ?? ?? 48 8D 7D ?? 48 83 EE ??",
        "48 89 CE 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 7D ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 1D ?? ?? ?? ?? 48 89 5D ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 89 F1 48 89 FA E8 ?? ?? ?? ?? 90 48 8D 7D ?? 48 83 EE ??",
      ]
      offset = 24
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "ref" { entry = "ItemStack.copyCtor" }
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 07" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "ItemStack.reinit" {
  kind     = "vslot"
  notes    = "Slot of void ItemStack::reinit(Item const&, int count, int auxValue) in the ItemStack vtable: 3 on 26.51. Each reinit overload builds a temporary ItemStack with the matching constructor, assigns it to this and destroys it: slot 1 copies a 16-byte string_view argument first, slot 2 passes a null user-data pointer in r9 (the Block overload, two arguments), slot 3 passes a null user-data pointer on the stack (`mov qword ptr [rsp+20h],0`) and the item, count and aux in rdx, r8d and r9d (this overload). The ItemInstance reinit overloads have the same head, so recipe 0 runs past the assignment to the net id copy that only ItemStack has: `mov rsi,rcx; mov qword ptr [rsp+..],0; lea rdi,[rbp-..]; mov rcx,rdi; call; lea rbx,[rip+..]; mov [rbp-..],rbx; mov dword ptr [rbp+..],0; mov byte ptr [rbp+..],0; mov rcx,rsi; mov rdx,rdi; call; lea rdi,[rbp+..]; sub rsi,..`, then function_start_of and slot_of ItemStack.vtable. Recipe 1 is the neighbour route: ItemStack::setNull (slot 4, its head `push rbp; push rsi; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov rsi,rcx; lea rcx,[rbp+..]; mov [rbp+..],rdx`), slot_of ItemStack.vtable, minus 1 (the header puts setNull right after the reinit overloads). Every displacement, immediate and branch byte is a wildcard. Range window 0..8 (the table has 7 slots). Both recipes are now a pattern_any, one alternative per shape: alternative 1 has the nops that 26.60 adds after the calls. Recipe 1 also runs on past the two calls of setNull (the optional copy and the base setNull) to the empty net id store and `sub rsi,-80h`, because the 26.51 head alone matches 11 functions on 26.60. Value 3 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): nops after calls; the slot bodies are the same (the ItemStack vtable on 26.60 holds the same 7 slots). Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 CE 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 7D ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 1D ?? ?? ?? ?? 48 89 5D ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 89 F1 48 89 FA E8 ?? ?? ?? ?? 48 8D 7D ?? 48 83 EE ??",
        "48 89 CE 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 7D ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 1D ?? ?? ?? ?? 48 89 5D ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 89 F1 48 89 FA E8 ?? ?? ?? ?? 90 48 8D 7D ?? 48 83 EE ??",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ItemStack.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "55 56 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE 48 8D 4D ?? 48 89 55 ?? E8 ?? ?? ?? ?? 48 8D 55 ?? 48 89 F1 E8 ?? ?? ?? ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 83 EE ??",
        "55 56 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE 48 8D 4D ?? 48 89 55 ?? E8 ?? ?? ?? ?? 90 48 8D 55 ?? 48 89 F1 E8 ?? ?? ?? ?? 90 C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 83 EE ??",
      ]
    }
    step "slot_of" { vtable = "ItemStack.vtable" }
    step "add" { value = -1 }
  }

  check "range" {
    min = 0
    max = 8
  }
}

entry "ItemStack.fromTag" {
  kind     = "function"
  notes    = "static ItemStack ItemStack::fromTag(CompoundTag const& tag): rcx the hidden result pointer (the 98h-byte ItemStack the caller owns), rdx the tag; returns the result pointer in rax. It builds a local ItemStack (ItemStackBase constructor, the ItemStack vtable, an empty net id variant), loads it from the tag with the shared stack loader (which reads Name, Count, Damage, WasPickedUp, CanPlaceOn and the rest), runs an Item virtual fix-up, moves the local into the result and destroys the local with ItemStackBase.dtor. An unknown item name gives an empty stack. The overload that also takes an ILevel& is the function 0x1A7D7B0 on 26.60 and is not this entry. Recipe 0 is a loop call site in a function that loads stacks from a list (`jge; lea rcx,[rbp+..]; mov rdx,r12; call`), read as the call target. Recipe 1 is a call site that follows a sign test (`js; mov rcx,r14; mov rdx,r13; call`), read the same way. Recipe 2 is the prologue of the function itself (`push rbp; push r14; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov rdi,rdx; mov rsi,rcx; lea rbx,[rbp+..]`). The three recipes are in three functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 1A7D560h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "0F 8D ?? ?? ?? ?? 48 8D 4D ?? 4C 89 E2 E8 ?? ?? ?? ??"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 88 ?? ?? ?? ?? 4C 89 F1 4C 89 EA E8 ?? ?? ?? ??"
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "55 41 56 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 D7 48 89 CE 48 8D 5D ??" }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
