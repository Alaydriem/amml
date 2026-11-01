entry "HudScreenController.vtable" {
  kind     = "global"
  notes    = "The primary vtable of HudScreenController (no RTTI), a helper for HudScreenController.tick and the table that the tick hook needs. Found from the HudScreenController constructor: it is the one function that stores the std::function tables of the lambdas whose type names say `HudScreenController::HudScreenController(std::shared_ptr<ClientInstanceScreenModel>)`. After the base constructor call it stores this table at [rsi] and a second base table at +0BC8h. The HudScreenController destructor stores the same pair; the deleting destructor in slot 0 calls it and frees 1158h bytes. Slot 2 (tick) of the table calls MinecraftScreenController::tick, the base tick that the named `EDUPlayScreenController::tick` also calls. Recipe 0 is the constructor: `lea rdx,[rbp+..]; mov rcx,rsi; call; lea rax,[rip+..]; mov [rsi],rax; lea rax,[rip+..]; mov [rsi+..],rax; lea rax,[rsi+..]; mov [rbp+..],rax; lea rax,[rsi+..]` (a sibling screen controller has the same head and then zeroes its members); it reads the RIP target of the first lea after the call. Recipe 1 is the head of the destructor: `push rbp; push rsi; push rdi; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp-8],-2; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; lea rax,[rip+..]; mov [rcx+..],rax; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]`; it reads the RIP target of the first lea. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and RIP offset is a wildcard. Recipe 0 is now a pattern_any, one alternative per shape of the constructor, both with the first lea at offset 14: alternative 1 has the nop that 26.60 adds after the base constructor call. The two alternatives start inside the `lea rdx,[rbp+..]` before the call (at its ModRM byte on 26.51, at its displacement on 26.60), so that the lea after the call sits at the same offset; the lint decodes the match from the start of its .pdata range. Recipe 1 now starts at `mov rsi,rcx` in the destructor and keeps the two low bytes of the screen model offset 0E18h, because on 26.60 the destructor is a deleting destructor with a longer prologue and three sibling destructors have the same head (they read the model at 12D0h). What changed in 6ad79ee4-17d51891a1119914 (26.60): the nop after the call in the constructor, and the destructor became the deleting one (`mov edi,edx` before `mov rsi,rcx`). The table, the second base at +0BC8h and the model at +0E18h are the same; both recipes agree. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "8D 95 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 06 48 8D 05 ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 86",
        "95 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 90 48 8D 05 ?? ?? ?? ?? 48 89 06 48 8D 05 ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 86",
      ]
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 05 ?? ?? ?? ?? 48 89 81 ?? ?? ?? ?? 48 8B 89 18 0E ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      literal_ok = [
        "30: low byte of the screen model offset 0E18h: the three sibling destructors with the same head read the model at 12D0h",
        "31: second byte of the screen model offset",
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
}

entry "HudScreenController.tick" {
  kind     = "vslot"
  notes    = "Slot of ui::DirtyFlag HudScreenController::tick() in the screen controller vtable: 2 on 26.51 (IScreenController header order: dtor 0, preFrameTick 1, tick 2). The function named by the text `virtual ui::DirtyFlag __cdecl EDUPlayScreenController::tick(void)` sits in slot 2 of its table, after the shared empty preFrameTick. The HudScreenController override first ticks a member of the screen model (`mov rcx,[rcx+0BF0h]; call`), then calls MinecraftScreenController::tick. Recipe 0 is that head (`mov rsi,rcx; mov rcx,[rcx+..]; call; mov rcx,rsi; call; mov ecx,[rsi+..]`), then function_start_of and slot_of HudScreenController.vtable. Recipe 1 is a member test later in the body (`cmp dword ptr [rcx+..],..; je; xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov eax,[rcx+..]; test eax,eax`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 0..8 (the interface has 6 virtuals). The ModRM bytes of the compare and the load in recipe 1 are wildcards. Value 2 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the weak_ptr lock in the body uses rdx for the control block (rcx on 26.51), so recipe 1 did not match; the vtable is HudScreenController.vtable on both builds and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 CE 48 8B 89 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 8B 8E ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "HudScreenController.vtable" }
  }

  recipe {
    step "pattern" { bytes = "83 ?? ?? ?? 0F 84 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 8B ?? ?? 85 C0" }
    step "function_start_of" {}
    step "slot_of" { vtable = "HudScreenController.vtable" }
  }

  check "range" {
    min = 0
    max = 8
  }
}

entry "HudScreenController.handleSlotSelection" {
  kind     = "function"
  notes    = "void HudScreenController::_handleSlotSelection(int slot, ContainerID containerId): rcx the controller, edx the slot, r8d the container id (Inventory is 0). The body saves the two arguments (`mov ebx,r8d; mov edi,edx; mov rsi,rcx`), asserts the screen model at this+0D48h (the ClientInstanceScreenModel), and calls the model function that ends in PlayerInventory::selectSlot (the function that rejects a slot above 8, stores the slot at +10h, mSelected, and the container id at +0B0h). Its callers are four HUD input handlers that pass the next, the previous or a numbered hotbar slot, each with container id 0. Recipe 0 is the handler that selects the previous slot: `mov edx,[rdi+10h]` (PlayerInventory::mSelected) `; dec edx; mov rcx,rsi; xor r8d,r8d; call`, read as the call target. Recipe 1 is the handler that selects a slot that a helper returns: `mov rcx,rdi; xor edx,edx; call; mov ebx,eax; mov rcx,rsi; mov edx,ebx; xor r8d,r8d; call`, read as the call target. The two recipes are in two functions. Every displacement and branch byte is a wildcard. The check is function_start. Recipe 1 now starts at the helper call and wildcards the ModRM bytes of the two movs of its result (`call; mov r32,eax; mov rcx,rsi; mov edx,r32; xor r8d,r8d; call`). What changed in 6ad79ee4-17d51891a1119914 (26.60): the handler keeps the slot in edi (ebx on 26.51) and its first argument setup changed; 4 callers on each build, and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "8B 57 ?? FF CA 48 89 F1 45 31 C0 E8"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 89 ?? 48 89 F1 89 ?? 45 31 C0 E8"
      offset = 15
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
