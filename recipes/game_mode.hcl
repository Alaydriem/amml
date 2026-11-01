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
