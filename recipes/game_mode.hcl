entry "GameMode.buildBlock" {
  kind     = "vslot"
  notes    = "Slot of bool GameMode::buildBlock(BlockPos const&, uchar face, HandSlot, bool isSimTick) in the GameMode vtable: 6 on 26.51 (LeviLamina header order: dtor, startDestroyBlock, destroyBlock, continueDestroyBlock, stopDestroyBlock, startBuildBlock 5, buildBlock 6; the GameMode table has the 20 header virtuals). The GameMode constructor stores the table and the player at +8; the base body at slot 6 takes the position in rdx, the face in r8b, the hand slot in r9b and isSimTick as the fifth argument. The client's game mode is a derived class (its constructor calls the GameMode constructor, size 110h); its slot 6 is a leaf that tests a flag and a global and then jumps to the base body. Recipe 0 is GameMode::startBuildBlock (slot 5), which calls slot 6 with isSimTick false: `sub rsp,..; mov dword ptr [rcx+..],0; mov rax,[rcx]; mov rax,[rax+disp]; mov byte ptr [rsp+20h],0; call [guard]`. Recipe 1 is GameMode::continueBuildBlock (slot 7), which calls slot 6 with isSimTick true: `mov r14d,[rsi+..]; mov r13d,[rsi+..]; movzx r8d,byte ptr [rbp+..]; mov rax,[rsi]; mov rax,[rax+disp]; mov r10,[guard]; mov byte ptr [rsp+20h],1`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 0..12. Recipe 1 now starts at `movzx r8d,byte ptr [rbp+..]` (offset 8 to the slot load). What changed in 6ad79ee4-17d51891a1119914 (26.60): the two loads before it became `mov r15d,[rsi+..]; movq xmm6,[rsi+..]`, so the 26.51 pattern did not match; the slot is 6 on both builds and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? C7 41 ?? ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? C6 44 24 ?? ?? FF 15"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "44 0F B6 45 ?? 48 8B 06 48 8B 40 ?? 4C 8B 15 ?? ?? ?? ?? C6 44 24 ?? ??"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 12
  }
}

entry "GameMode.vtable" {
  kind     = "global"
  notes    = "The base GameMode vtable (no RTTI), a helper: the buildBlock hook takes the base body from its slot 6 (GameMode.buildBlock), as the C++ hooked GameMode::buildBlock. 20 slots (the header count). The client's game mode is a derived class with its own table, whose slot 6 is a leaf that jumps to this base body. Recipe 0 is the GameMode constructor: `lea rax,[rip+..]; mov [rcx],rax; mov [rcx+8],rdx; mov qword ptr [rcx+10h],-1; mov dword ptr [rcx+18h],-1; mov byte ptr [rcx+1Ch],0` (the table, mPlayer, mDestroyBlockPos, mDestroyBlockFace); it reads the RIP target of the lea. Recipe 1 is the GameMode destructor: `push rsi; sub rsp,..; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rcx,[rcx+..]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax]; mov edx,1; call [guard]; mov rcx,[rsi+..]` (it releases mMessenger, then mTimer); it reads the RIP target of the lea. The pattern runs past the first release call, because the head alone is the same in other destructors. The two recipes are in two functions. Every displacement, immediate and RIP offset is a wildcard."

  recipe {
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 89 51 ?? 48 C7 41 ?? ?? ?? ?? ?? C7 41 ?? ?? ?? ?? ?? C6 41 ?? ??" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "56 48 83 EC ?? 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 89 ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 00 BA ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ??"
      offset = 8
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

entry "GameMode.mPlayer" {
  kind     = "field"
  notes    = "Offset of Player& GameMode::mPlayer: 8 on 26.51, the anchor of the GameMode mirror (LeviLamina header: the first member after the vptr). Recipe 0 is the GameMode constructor: `lea rax,[rip+..]; mov [rcx],rax; mov [rcx+8],rdx; mov qword ptr [rcx+10h],-1; mov dword ptr [rcx+18h],-1; mov byte ptr [rcx+1Ch],0` (the vtable, the player argument, then mDestroyBlockPos and mDestroyBlockFace); it reads the displacement of the player store. Recipe 1 is GameMode::getMaxPickRange, which loads the player at once (`mov rdi,[rcx+8]; lea rcx,[rdi+8]; call`, the entity context of the player); it reads the displacement of that load. The two recipes are in two functions. Range window 0..40h."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 89 51 ?? 48 C7 41 ?? ?? ?? ?? ?? C7 41 ?? ?? ?? ?? ?? C6 41 ?? ??"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "GameMode.getMaxPickRange" }
    step "pattern" {
      bytes  = "48 8B 79 ?? 48 8D 4F ?? E8"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 64
  }
}

entry "GameMode.getMaxPickRange" {
  kind     = "function"
  notes    = "float GameMode::getMaxPickRange(): rcx the game mode, the range in xmm0. The body reads the player (`mov rdi,[rcx+8]`), gets the player's game type through the entity context (the same inline code as Player::getPlayerGameType: when the type is Default, the level's default game type through Level slot 140), and returns 12.0 for creative, else 6.7. GameMode::getPickRange (slot 10) holds the same code inline after its input-mode tests. The function is called three times by the item-use-on-actor transaction handler (the one function that loads the text `, maxPickRange = `), each time on the player's GameMode (`mov rcx,[rsi+0AA0h]`, Player::mGameMode, header sum). Recipe 0 is the start of the body up to the type test (`push rsi; push rdi; sub rsp,..; mov rdi,[rcx+..]; lea rcx,[rdi+..]; call; mov esi,eax; cmp eax,..`); the match is the function start. Recipe 1 is the first call site (`movaps xmm6,xmm0; mov rcx,[rsi+..]; call`), read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 1 now finds the transaction handler by its text first (only that function loads it, on both builds) and wildcards the ModRM byte of the GameMode load. Player::mGameMode is AA0h on 26.51 and AC8h on 26.60 (the load `mov rcx,[rsi+0AA0h]` became `mov rcx,[rdi+0AC8h]`). What changed in 6ad79ee4-17d51891a1119914 (26.60): the handler holds the player in rdi and Player members moved, so the 26.51 pattern did not match; the three calls are the same (3 callers on each build), and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "56 57 48 83 EC ?? 48 8B 79 ?? 48 8D 4F ?? E8 ?? ?? ?? ?? 89 C6 83 F8 ??" }
  }

  recipe {
    step "string_xref" { text = ", maxPickRange = " }
    step "pattern" {
      bytes  = "0F 28 F0 48 8B ?? ?? ?? ?? ?? E8"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "GameMode.startDestroyBlock" {
  kind     = "vslot"
  notes    = "Slot of bool GameMode::startDestroyBlock(BlockPos const& pos, uchar face, bool& hasDestroyedBlock) in the GameMode vtable: 1 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the position, r8b the face, r9 the bool& (the base body writes 0 to it after its first virtual call: `mov rbx,r9; ... mov byte ptr [rbx],0`); the bool result is in al. The base body is 37698E0h. SurvivalMode.vtable (the table of the player's game mode in survival) holds a wrapper at slot 1 that tests the trial-mode flag and a global and else jumps to this base body (`add rsp,28h; jmp`), so a hook on the base body sees survival, creative and adventure calls. Recipe 0 is the head of the base body (`mov rbx,r9; mov ebp,r8d; mov rsi,rdx; mov rdi,rcx; mov [rsp+..],r8b; mov rcx,[rcx+..]; mov rdx,[rcx+..]; mov eax,[rdx+..]`, the player and the add-ref of a shared pointer it holds), then function_start_of and slot_of GameMode.vtable. Recipe 1 is the neighbour: the head of the base GameMode::destroyBlock(BlockPos const&, uchar) body at slot 2 (`mov ebx,r8d; mov rdi,rdx; mov rsi,rcx; mov rcx,[rcx+..]; mov rdx,[rcx+..]; mov eax,[rdx+..]`), then function_start_of, slot_of GameMode.vtable and add -1; it depends on the header order of slots 1 and 2. The two recipes are in two functions and share no pattern bytes. No direct call site of this slot was found through Player::mGameMode loads. Every displacement, immediate and branch byte is a wildcard. Range window 0..9. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" { bytes = "4C 89 CB 44 89 C5 48 89 D6 48 89 CF 44 88 44 24 ?? 48 8B 49 ?? 48 8B 91 ?? ?? ?? ?? 8B 42 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern" { bytes = "44 89 C3 48 89 D7 48 89 CE 48 8B 49 ?? 48 8B 91 ?? ?? ?? ?? 8B 42 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
    step "add" { value = -1 }
  }

  check "range" {
    min = 0
    max = 9
  }
}

entry "GameMode.continueDestroyBlock" {
  kind     = "vslot"
  notes    = "Slot of bool GameMode::continueDestroyBlock(BlockPos const& pos, uchar face, Vec3 const& playerPos, bool& hasDestroyedBlock) in the GameMode vtable: 3 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the position, r8b the face, r9 the Vec3 const&, the bool& at [rsp+28h] on entry (the body loads it with `mov rbx,[rbp+0C0h]` and writes 0 to it after its first virtual call); the bool result is in al. The base body is 376A590h. SurvivalMode.vtable holds the same body at slot 3 (no wrapper), so a hook on the base body sees every game type. Recipe 0 is the head of the base body (`mov r15,r9; mov [rbp+..],r8b; mov rdi,rdx; mov rsi,rcx; mov rbx,[rbp+..]; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rbp+..]; call [guard]`, ended at the call), then function_start_of and slot_of GameMode.vtable. Recipe 1 is the neighbour: the start of the base GameMode::stopDestroyBlock body at slot 4 (`push rsi; sub rsp,..; mov rsi,rcx; mov rcx,[rcx+..]; movss xmm2,[rsi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`, ended at the call), then function_start_of, slot_of GameMode.vtable and add -1; it depends on the header order of slots 3 and 4. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 0..9. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" { bytes = "4D 89 CF 44 88 45 ?? 48 89 D7 48 89 CE 48 8B 9D ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? FF 15" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern" { bytes = "56 48 83 EC ?? 48 89 CE 48 8B 89 ?? ?? ?? ?? F3 0F 10 56 ?? 48 8B 01 48 8B 40 ?? FF 15" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
    step "add" { value = -1 }
  }

  check "range" {
    min = 0
    max = 9
  }
}

entry "GameMode.useItem" {
  kind     = "vslot"
  notes    = "Slot of bool GameMode::useItem(ItemStack& item, bool) in the GameMode vtable: 11 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the stack, r8b a bool that the header does not have (both client callers pass false: `xor r8d,r8d`); the bool result is in al. The base body (376E540h) copies the stack, sends the item-use event (it returns false when the event is cancelled), then calls the item's use virtual through the stack's Item (Item vtable +288h, with the stack, the player and the bool) and returns true when the stack changed. A hook must pass r8 on. SurvivalMode.vtable holds a wrapper at slot 11 that tests the trial-mode flag and a global and else jumps to this base body, so a hook on the base body sees every game type. The slot after it (12, 376EFB0h) is a second use function without the event (ItemStack&, a pointer, a bool; Item vtable +298h); useItemOn is slot 13 on this build. Recipe 0 is the item use call in the base body (`mov rax,[rcx]; mov rax,[rax+..]; mov rdx,rsi; mov r8,r15; movzx r9d,byte ptr [rbp+..]; call [guard]`, ended at the call), then function_start_of and slot_of GameMode.vtable. Recipe 1 is a client call site in the function at 1038530h: `pop rbx; pop rdi; pop rsi; pop r14; pop rbp; ret; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rbp+..]; xor r8d,r8d; call [guard]`, read as the displacement of the slot load and turned into a slot. The same call is in a second function (0DCFDC60h) whose epilogue before it has no `pop r14`; the epilogue bytes are the only difference, so recipe 1 depends on the push list of that function. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 7..15. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" { bytes = "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 4D 89 F8 44 0F B6 8D ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "5B 5F 5E 41 5E 5D C3 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? 45 31 C0 FF 15"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 7
    max = 15
  }
}

entry "GameMode.attack" {
  kind     = "vslot"
  notes    = "Slot of bool GameMode::attack(Actor& target, Vec3 const&) in the GameMode vtable: 15 on 6aea437e-7eff13daabb430da (26.62). The header has 14; this build has one more use slot before useItemOn (see GameMode.useItem), so interact is 14 and attack 15. rcx the game mode, rdx the target, r8 a Vec3 const& (the header has no second argument); the bool result is in al. Proof of the slot: the item-use-on-actor transaction handler (the one function that loads the text `, maxPickRange = `) dispatches in a lambda on the action type at +70h: type 0 (interact) calls slot 14 (+70h) of Player::mGameMode (+0AC8h) with the actor, the click position (+0ECh) and a byte, type 1 (attack) calls slot 15 (+78h) with the actor and the click position. The base slot holds a thunk without .pdata (`mov r9,r8; mov r8b,1; jmp 37692B0h`); the shared attack body takes (Actor&, bool, Vec3 const&). SurvivalMode.vtable slot 15 does not reach this thunk: it is its own leaf (`mov r9,r8; <trial-mode tests>; mov r8b,1; jmp 37692B0h`), so a survival attack never runs the base slot. A hook must hook slot 15 of SurvivalMode.vtable for survival, and slot 15 of GameMode.vtable for the other game types. Recipe 0 is the base thunk and the head of the shared body after it (`mov r9,r8; mov r8b,..; jmp; int3 x5; push rbp; push r15; push r14; push r12; push rsi; push rdi; push rbx`), offset 0, then slot_of GameMode.vtable; it depends on the link order (the thunk sits right before its target); five other thunks have the same three instructions. Recipe 1 is the SurvivalMode leaf (`mov r9,r8; cmp byte ptr [rcx+..],..; jne; cmp byte ptr [rip+..],..; je; xor eax,eax; ret; mov r8b,..; jmp`), offset 0, then slot_of SurvivalMode.vtable. The two recipes are in two functions and two tables and share only the first instruction. The handler lambda is a leaf without .pdata, so its call site is evidence in these notes and not a recipe (the lint cannot decode a match there). Every displacement, immediate and branch byte is a wildcard. Range window 11..19. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" {
      bytes  = "4D 89 C1 41 B0 ?? E9 ?? ?? ?? ?? CC CC CC CC CC 55 41 57 41 56 41 54 56 57 53"
      offset = 0
    }
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 89 C1 80 B9 ?? ?? ?? ?? ?? 75 ?? 80 3D ?? ?? ?? ?? ?? 74 ?? 31 C0 C3 41 B0 ?? E9"
      offset = 0
    }
    step "slot_of" { vtable = "SurvivalMode.vtable" }
  }

  check "range" {
    min = 11
    max = 19
  }
}
