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
  notes    = "float GameMode::getMaxPickRange(): rcx the game mode, the range in xmm0. The body reads the player (`mov rdi,[rcx+8]`), gets the player's game type through the entity context (the same inline code as Player::getPlayerGameType: when the type is Default, the level's default game type through Level slot 140), and returns 12.0 for creative, else 6.7. GameMode::getPickRange (slot 10) holds the same code inline after its input-mode tests. The function is called three times by the item-use-on-actor transaction handler (the one function that loads the text `, maxPickRange = `), each time on the player's GameMode (`mov rcx,[rsi+0AA0h]`, Player::mGameMode, header sum). Recipe 0 is the start of the body up to the type test (`push rsi; push rdi; sub rsp,..; mov rdi,[rcx+..]; lea rcx,[rdi+..]; call; mov esi,eax; cmp eax,..`); the match is the function start. Recipe 1 is the first call site (`movaps xmm6,xmm0; mov rcx,[rsi+..]; call`), read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 1 now finds the transaction handler by its text first (only that function loads it, on both builds) and wildcards the ModRM byte of the GameMode load. Player::mGameMode is AA0h on 26.51 and AC8h on 26.60 (the load `mov rcx,[rsi+0AA0h]` became `mov rcx,[rdi+0AC8h]`). What changed in 6ad79ee4-17d51891a1119914 (26.60): the handler holds the player in rdi and Player members moved, so the 26.51 pattern did not match; the three calls are the same (3 callers on each build), and recipe 0 agrees. Would have survived with a better recipe. Layout gate: 26.63 only (entry_is LevelChunk.mSubChunks = 456); on 70.x the function is inlined into its three callers, which call the game-type test and then load 12.0 or 6.7; the replacement is Player.isCreative (with GameMode.mPlayer and the constants 12.0 and 6.7)."

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
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "GameMode.startDestroyBlock" {
  kind     = "vslot"
  hooked   = true
  notes    = "Slot of bool GameMode::startDestroyBlock(BlockPos const& pos, uchar face, bool& hasDestroyedBlock) in the GameMode vtable: 1 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the position, r8b the face, r9 the bool& (the base body writes 0 to it after its first virtual call: `mov rbx,r9; ... mov byte ptr [rbx],0`); the bool result is in al. The base body is 37698E0h. SurvivalMode.vtable (the table of the player's game mode in survival) holds a wrapper at slot 1 that tests the trial-mode flag and a global and else jumps to this base body (`add rsp,28h; jmp`), so a hook on the base body sees survival, creative and adventure calls. Recipe 0 is the head of the base body (`mov rbx,r9; mov ebp,r8d; mov rsi,rdx; mov rdi,rcx; mov [rsp+..],r8b; mov rcx,[rcx+..]`, the register moves and the player load, ended before the code that follows them, which changed between builds), then function_start_of and slot_of GameMode.vtable. Recipe 1 is the neighbour: the head of the base GameMode::destroyBlock(BlockPos const&, uchar) body at slot 2, then function_start_of, slot_of GameMode.vtable and add -1; it depends on the header order of slots 1 and 2. It is one pattern_any with one alternative per shape of the code after the player load: alternative 0 (26.62, 26.63) `mov ebx,r8d; mov rdi,rdx; mov rsi,rcx; mov rcx,[rcx+..]; mov rdx,[rcx+..]; mov eax,[rdx+..]` (the inlined component lookup of a shared pointer); alternative 1 (70.26) `mov ebx,r8d; mov rdi,rdx; mov rsi,rcx; mov rcx,[rcx+..]; call` (the lookup became a call). The two recipes are in two functions and share no pattern bytes. No direct call site of this slot was found through Player::mGameMode loads. Every displacement, immediate and branch byte is a wildcard. Range window 0..9. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written. What changed in 6afa4e47-dadc375797a3d31a (70.26): EnTT component type ids are assigned at run time, so the inlined component lookup after the player load became a call; the register moves before it and the slot (1) are the same, while the push list and frame size of the body changed (the patterns never covered them). Slots 0..21 of GameMode.vtable hold the same functions in the same order on 26.63 and 70.26. Evidence on 70.26: a pattern drafted from the 26.63 slot 1 head matches once on 70.26, in the function in slot 1, and that body still writes 0 to the bool& after its first virtual call (`mov byte ptr [rbx],0`); the destroyBlock head, drafted from the 26.63 slot 2 body, matches once in the function in slot 2 on 70.26, and recipe 1 gives slot 2 minus 1. Would have survived with a better recipe. `hooked = true`: the loader hooks the function in this slot for the player-action events; `signature` probes the slot targets of the vtables its slot_of steps name."

  recipe {
    step "pattern" { bytes = "4C 89 CB 44 89 C5 48 89 D6 48 89 CF 44 88 44 24 ?? 48 8B 49 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "44 89 C3 48 89 D7 48 89 CE 48 8B 49 ?? 48 8B 91 ?? ?? ?? ?? 8B 42 ??",
        "44 89 C3 48 89 D7 48 89 CE 48 8B 49 ?? E8 ?? ?? ?? ??",
      ]
    }
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
  hooked   = true
  notes    = "Slot of bool GameMode::continueDestroyBlock(BlockPos const& pos, uchar face, Vec3 const& playerPos, bool& hasDestroyedBlock) in the GameMode vtable: 3 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the position, r8b the face, r9 the Vec3 const&, the bool& at [rsp+28h] on entry (the body loads it with `mov rbx,[rbp+0C0h]` and writes 0 to it after its first virtual call); the bool result is in al. The base body is 376A590h. SurvivalMode.vtable holds the same body at slot 3 (no wrapper), so a hook on the base body sees every game type. Recipe 0 is the head of the base body, then function_start_of and slot_of GameMode.vtable. It is one pattern_any with one alternative per shape of the head: alternative 0 (26.62, 26.63) `mov r15,r9; mov [rbp+..],r8b; mov rdi,rdx; mov rsi,rcx; mov rbx,[rbp+..]; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rbp+..]; call [guard]`, ended at the call; alternative 1 (70.26) `mov r15,r9; mov ebx,r8d; mov rdi,rdx; mov rsi,rcx; mov r13,[rbp+..]` (the bool& argument, loaded from the caller frame). Recipe 1 is the neighbour: the start of the base GameMode::stopDestroyBlock body at slot 4 (`push rsi; sub rsp,..; mov rsi,rcx; mov rcx,[rcx+..]; movss xmm2,[rsi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`, ended at the call), then function_start_of, slot_of GameMode.vtable and add -1; it depends on the header order of slots 3 and 4. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 0..9. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written. What changed in 6afa4e47-dadc375797a3d31a (70.26): the body keeps the face in ebx and reads a flag of the player before the first virtual call, so the 26.63 head did not match; the slot (3) is the same. Evidence on 70.26: recipe 1 (the stopDestroyBlock head, drafted from the 26.63 slot 4 body, unchanged) matches once in the function in slot 4 and gives 3; the head drafted from the 26.63 slot 3 body (alternative 1) matches once in the function in slot 3, whose body takes the bool& from the caller frame (`mov r13,[rbp+0F0h]`) and writes 0 to it, and 1 later (`mov byte ptr [r13],0` and `[r13],1`), as the 26.63 body does through rbx. Would have survived with a better recipe. `hooked = true`: the loader hooks the function in this slot for the player-action events; `signature` probes the slot targets of the vtables its slot_of steps name."

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 89 CF 44 88 45 ?? 48 89 D7 48 89 CE 48 8B 9D ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? FF 15",
        "4D 89 CF 44 89 C3 48 89 D7 48 89 CE 4C 8B AD ?? ?? ?? ??",
      ]
    }
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
  hooked   = true
  notes    = "Slot of bool GameMode::useItem(ItemStack& item, bool) in the GameMode vtable: 11 on 6aea437e-7eff13daabb430da (26.62), the header slot. rcx the game mode, rdx the stack, r8b a bool that the header does not have (both client callers pass false: `xor r8d,r8d`); the bool result is in al. The base body (376E540h) copies the stack, sends the item-use event (it returns false when the event is cancelled), then calls the item's use virtual through the stack's Item (Item vtable +288h, with the stack, the player and the bool) and returns true when the stack changed. A hook must pass r8 on. SurvivalMode.vtable holds a wrapper at slot 11 that tests the trial-mode flag and a global and else jumps to this base body, so a hook on the base body sees every game type. The slot after it (12, 376EFB0h) is a second use function without the event (ItemStack&, a pointer, a bool; Item vtable +298h); useItemOn is slot 13 on this build. Recipe 0 is the item use call in the base body, ended at the call, then function_start_of and slot_of GameMode.vtable. It is one pattern_any with one alternative per shape: alternative 0 (26.62, 26.63) `mov rax,[rcx]; mov rax,[rax+..]; mov rdx,rsi; mov r8,r15; movzx r9d,byte ptr [rbp+..]; call [guard]` (a virtual call of the item); alternative 1 (70.26) `mov r8,[rbx+..]; mov rcx,rdi; mov rdx,rsi; movzx r9d,byte ptr [rbp+..]; call` (a direct call). Recipe 1 is a client call site, the player method that uses the held item through Player::mGameMode: `pop rbx; pop rdi; pop rsi; [pop r14; pop rbp;] ret; mov rcx,[player+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[..]; xor r8d,r8d; call [guard]`, read as the displacement of the slot load and turned into a slot. It is one pattern_any with one alternative per shape of the epilogue and of the argument setup (the 70.26 alternative starts with `add rsp,..` so that the slot load sits at the same offset, and it sets r8 to the second argument of the method and r9 to 0). The same call is in a second function on 26.62 and 26.63 whose epilogue before it has no `pop r14`; the epilogue bytes are the only difference, so recipe 1 depends on the push list of that function. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 7..15. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written. What changed in 6afa4e47-dadc375797a3d31a (70.26): the slot is 11 as before and the function sits at the same place in the table (the slot after it, 12, is the same function as on 26.63), but it has one more argument: rcx the game mode, rdx the stack, r8 a pointer (the client call site passes the second argument of its own method), r9b the bool (it was r8b); the item use is a direct call. A hook on this slot must change for the new argument list (mod-code work). Evidence on 70.26: the 26.63 client call site (the player method that tests its virtual at +170h, the ItemStack virtual at +28h and the 10-tick limit, then calls slot 11 of Player::mGameMode with `xor r8d,r8d`) has the same shape on 70.26 and calls the slot at +58h (slot 11) of Player::mGameMode with the stack, the method argument and `xor r9d,r9d`; the function after slot 11 on 26.63 (slot 12, the use function without the event) has the same head as the function in slot 12 on 70.26, so the function in slot 11 is the one before it. Real code change (a new argument). Correction and shape guard (26.70.26 rechecked for the player-action events): on 26.70.26 the function returns an InteractionResult through a hidden pointer, so the arguments are rcx the game mode, rdx the result pointer, r8 the stack and r9b the bool (the client call site passes a local buffer in rdx, `lea rbx,[rsp+..]; mov rdx,rbx`, and its own argument in r8). One name, one layout: this entry means the bool shape only (check entry_is GameMode.useItemResult = 0), and it is refused as not for this build on 26.70.26; GameMode.useItemInteraction is the same slot for the result shape. `hooked = true`: the loader hooks the function in this slot of GameMode.vtable for the player.item_use event; `signature` probes the slot targets of the vtables its slot_of steps name."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 4D 89 F8 44 0F B6 8D ?? ?? ?? ?? FF 15",
        "4C 8B 43 ?? 48 89 F9 48 89 F2 44 0F B6 8D ?? ?? ?? ?? E8",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "5B 5F 5E 41 5E 5D C3 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? 45 31 C0 FF 15",
        "83 C4 ?? 5B 5F 5E C3 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 5C 24 ?? 48 89 DA 49 89 F8 45 31 C9 FF 15",
      ]
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "entry_is" {
    entry = "GameMode.useItemResult"
    value = 0
  }

  check "range" {
    min = 7
    max = 15
  }
}

entry "GameMode.useItemInteraction" {
  kind     = "vslot"
  hooked   = true
  notes    = "Slot of InteractionResult GameMode::useItem(ItemStack&, bool) in the GameMode vtable: 11 on 6afa4e47-dadc375797a3d31a (26.70.26), the same slot and function as GameMode.useItem on the builds where useItem returns a bool. rcx the game mode, rdx the hidden InteractionResult pointer (returned in rax), r8 the stack, r9b the bool. One name, one layout: this entry means the result shape only (check entry_is GameMode.useItemResult = 1), so it is refused as not for this build on 26.63, where GameMode.useItem holds the slot. The recipes are those of GameMode.useItem (see its notes for the anchors and the 26.70.26 evidence): recipe 0 the item use call in the base body, then function_start_of and slot_of GameMode.vtable; recipe 1 the client call site read as a slot. A refused use writes {3, 0} into the result, as the base body does when the game's own item-use event is cancelled and as the SurvivalMode trial-mode wrapper of slot 11 does. `hooked = true`: the loader hooks the function in this slot for the player.item_use event. Range window 7..15. Class: new entry (the signature change of 26.70.26)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 4D 89 F8 44 0F B6 8D ?? ?? ?? ?? FF 15",
        "4C 8B 43 ?? 48 89 F9 48 89 F2 44 0F B6 8D ?? ?? ?? ?? E8",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "GameMode.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "5B 5F 5E 41 5E 5D C3 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? 45 31 C0 FF 15",
        "83 C4 ?? 5B 5F 5E C3 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 5C 24 ?? 48 89 DA 49 89 F8 45 31 C9 FF 15",
      ]
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "entry_is" {
    entry = "GameMode.useItemResult"
    value = 1
  }

  check "range" {
    min = 7
    max = 15
  }
}

entry "GameMode.attack" {
  kind     = "vslot"
  hooked   = true
  notes    = "Slot of bool GameMode::attack(Actor& target, Vec3 const&) in the GameMode vtable: 15 on 6aea437e-7eff13daabb430da (26.62). The header has 14; this build has one more use slot before useItemOn (see GameMode.useItem), so interact is 14 and attack 15. rcx the game mode, rdx the target, r8 a Vec3 const& (the header has no second argument); the bool result is in al. Proof of the slot: the item-use-on-actor transaction handler (the one function that loads the text `, maxPickRange = `) dispatches in a lambda on the action type at +70h: type 0 (interact) calls slot 14 (+70h) of Player::mGameMode (+0AC8h) with the actor, the click position (+0ECh) and a byte, type 1 (attack) calls slot 15 (+78h) with the actor and the click position. The base slot holds a thunk without .pdata (`mov r9,r8; mov r8b,1; jmp 37692B0h`); the shared attack body takes (Actor&, bool, Vec3 const&). SurvivalMode.vtable slot 15 does not reach this thunk: it is its own leaf (`mov r9,r8; <trial-mode tests>; mov r8b,1; jmp 37692B0h`), so a survival attack never runs the base slot. A hook must hook slot 15 of SurvivalMode.vtable for survival, and slot 15 of GameMode.vtable for the other game types. Recipe 0 is the base thunk and the head of the shared body after it (`mov r9,r8; mov r8b,..; jmp; int3 x5; push rbp; push r15; push r14; push r12; push rsi; push rdi; push rbx`), offset 0, then slot_of GameMode.vtable; it depends on the link order (the thunk sits right before its target); five other thunks have the same three instructions. Recipe 1 is the SurvivalMode leaf (`mov r9,r8; cmp byte ptr [rcx+..],..; jne; cmp byte ptr [rip+..],..; je; xor eax,eax; ret; mov r8b,..; jmp`), offset 0, then slot_of SurvivalMode.vtable. The two recipes are in two functions and two tables and share only the first instruction. The handler lambda is a leaf without .pdata, so its call site is evidence in these notes and not a recipe (the lint cannot decode a match there). Every displacement, immediate and branch byte is a wildcard. Range window 11..19. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written. `hooked = true`: the loader hooks the function in this slot for the player-action events; `signature` probes the slot targets of the vtables its slot_of steps name."

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


entry "GameMode.useItemResult" {
  kind     = "field"
  notes    = "Which result GameMode::useItem (the function in slot GameMode.useItem / GameMode.useItemInteraction of GameMode.vtable) has: 0 when it returns a bool in al and takes (ItemStack&, bool) in rdx and r8 (26.63, 6aed1906-b127418e74d2f2f9), 1 when it returns an InteractionResult through a hidden pointer and takes (InteractionResult* out, ItemStack&, bool) in rdx, r8 and r9 (26.70.26, 6afa4e47-dadc375797a3d31a). GameMode.useItem resolves only on 0 and GameMode.useItemInteraction only on 1 (entry_is), so a hook never gets the wrong argument list. Both recipes are a pattern_any over the executable whose alternative 0 is the bool shape and alternative 1 the result shape, then the `alternative` step; each alternative matches once on its own build, in the base useItem body, and zero times on the other. Recipe 0 is the head of the base body after the frame setup: alternative 0 `mov qword ptr [rbp+..],imm; mov r15b,r8b; mov rsi,rdx; mov rdi,rcx; lea rbx,[rbp+..]; mov rcx,rbx; call` (the bool in r8, the stack in rdx, copied into a local), alternative 1 `mov qword ptr [rbp+..],imm; mov [rbp+..],r9b; mov rdi,r8; mov rsi,rdx; mov rbx,rcx; lea rcx,[rbp+..]; mov rdx,r8; call` (the bool in r9, the stack in r8 copied, the result pointer kept in rsi). Recipe 1 is the end of the body: alternative 0 `mov esi,eax; jmp; xor esi,esi; jmp; xor esi,esi; xor sil,1; mov rcx,[rbp+..]` (a bool result that is returned in eax), alternative 1 `mov dword ptr [rsi],imm; mov byte ptr [rsi+..],imm; jmp; xor eax,eax; xor al,1; mov [rsi+..],al` (the result written through the hidden pointer, which is returned in rax). Every displacement and immediate byte is a wildcard. Evidence for the result shape on 26.70.26: the SurvivalMode wrapper of slot 11 keeps rdx (`mov rsi,rdx`), writes `mov dword ptr [rsi],3; mov byte ptr [rsi+4],0` and returns rsi on its trial-mode refusal, and else calls the base body with rdx unchanged; the base body writes the same {3, 0} when the game's own item-use event is cancelled and passes rsi to the item's use call as its result. On 26.63 the wrapper returns `xor eax,eax` and jumps to the base body. The check is range 0..1. Class: real code change (signature)."

  recipe {
    step "pattern_any" {
      patterns = ["48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 45 88 C7 48 89 D6 48 89 CF 48 8D 9D ?? ?? ?? ?? 48 89 D9 E8", "48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 44 88 8D ?? ?? ?? ?? 4C 89 C7 48 89 D6 48 89 CB 48 8D 8D ?? ?? ?? ?? 4C 89 C2 E8"]
    }
    step "alternative" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 C6 EB ?? 31 F6 EB ?? 31 F6 40 80 F6 ?? 48 8B 8D", "C7 06 ?? ?? ?? ?? C6 46 ?? ?? EB ?? 31 C0 34 ?? 88 46 ??"]
    }
    step "alternative" {}
  }

  check "range" {
    min = 0
    max = 1
  }
}
