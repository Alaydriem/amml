entry "Player.getPlayerGameType" {
  kind     = "function"
  notes    = "GameType Player::getPlayerGameType() const: rcx the player, the type in eax (Survival 0, Creative 1, Adventure 2, Default 5, Spectator 6; -1 when the component is missing). The body looks up the game type component through the entity context at this+8 (type hash 88D3EDDFh), and when the type is Default it tail-calls the level's default game type (`mov rcx,[rcx+1D8h]; mov rax,[rcx]; mov rax,[rax+460h]; ... jmp rdx`, Actor::mLevel and Level slot 140). Many callers hold the same code inline; Player::isCreative and the other tests next to it inline it too. Recipe 0 is the end of the body: `mov eax,[rdx+r9*4]; cmp eax,..; jne; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,[guard]`, then function_start_of. Recipe 1 is the SetPlayerGameTypePacket handler (NetEventCallback slot 224): `cmp qword ptr [rbx+..],..; je; mov ebx,[rsi+..]; mov rcx,rdi; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The ModRM bytes of the compare, the load and the mov in recipe 1 are wildcards. What changed in 6ad79ee4-17d51891a1119914 (26.60): the SetPlayerGameTypePacket handler holds the player in rbx and the new type in edi (rdi and ebx on 26.51); recipe 0 agrees. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): the entity component lookup no longer inlines the type hash probe loop (it calls a lookup function with the hash), so the body is a different shape; the end of the body is now `mov eax,[rdx+rcx*4]; cmp eax,5; jne rel32; mov rcx,[rsi+..]; test rcx,rcx; jne` (the Level pointer is checked, with an assert block before the level call). Recipe 0 is now a pattern_any with one alternative per shape. Recipe 1 gained a `test reg,reg; je` before the compare (the player pointer test) because the old pattern matched twice on 26.70.26 (the other match is a large function); the longer pattern still matches once on every build. The SetPlayerGameTypePacket handler keeps the same call order on 26.70.26 (get type, set type, a level call, get type again), so recipe 1 agrees with recipe 0. "

  recipe {
    step "pattern_any" {
      patterns = [
        "42 8B 04 8A 83 F8 ?? 75 ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15",
        "8B 04 8A 83 F8 ?? 0F 85 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 85 C9 75 ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 85 ?? 74 ?? 48 83 ?? ?? ?? ?? ?? ?? 74 ?? 8B ?? ?? 48 89 ?? E8"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Player.playerInventory" {
  kind     = "field"
  notes    = "Offset of std::unique_ptr<PlayerInventory> Player::mInventory: 5B8h on 26.51, the anchor of the inventory chain (Player, then PlayerInventory, then its Inventory). LeviLamina header sum: 5B8h. Both recipes are the inline Player::setSelectedItem: load the PlayerInventory (`mov rax,[rdi+5B8h]`), then its Inventory (`mov rcx,[rax+0B8h]`) and the selected slot (`mov edx,[rax+10h]`), then call FillingContainer::setItem (slot 12). The low bytes of those two PlayerInventory displacements are kept, so the anchor fails when the PlayerInventory mirror moves (mSelected 10h, mInventory 0B8h; the class has two 8-byte listener bases). Recipe 0 also tests the selected container id (`cmp byte ptr [rax+0B0h],0; jne`), whose low byte is kept too. Recipe 1 is a second function that calls setItem with the slot from a register. Both read the displacement of the first load. Range window 580h..600h."

  recipe {
    step "pattern" {
      bytes      = "48 8B 87 ?? ?? ?? ?? 80 B8 B0 ?? ?? ?? ?? 75 ?? 48 8B 88 B8 ?? ?? ?? 8B 50 10 48 8B 01 48 8B 40 ?? 49 89 F0 FF 15"
      literal_ok = [
        "9: cmp byte ptr [rax+0B0h]: PlayerInventory::mSelectedContainerId, part of the mirror this entry anchors",
        "19: mov rcx,[rax+0B8h]: PlayerInventory::mInventory, part of the mirror this entry anchors",
        "25: mov edx,[rax+10h]: PlayerInventory::mSelected, part of the mirror this entry anchors",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 8B 87 ?? ?? ?? ?? 48 8B 88 B8 ?? ?? ?? 48 8B 01 48 8B 40 ?? 89 DA 4D 89 F0 FF 15"
      literal_ok = ["10: mov rcx,[rax+0B8h]: PlayerInventory::mInventory, part of the mirror this entry anchors"]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1408
    max = 1536
  }
}

entry "Player.getPlayerPermissionLevel" {
  kind     = "function"
  notes    = "PlayerPermissionLevel Player::getPlayerPermissionLevel() const: rcx the player, the level in al (Visitor 0, Member 1, Operator 2, Custom 3). It is a plain function, not a virtual: on 26.63 it has three direct callers and no vtable holds it. Found from the isOperator wrapper (`mov rcx,[rcx+8]; call; cmp al,2; setge al`) and the server function that does `call; cmp al,2; jl`. The body looks up the AbilitiesComponent through the entity context at this+8 and returns byte +1 of it (LayeredAbilities::mPermissions: byte 0 the command permission \"permissionsLevel\", byte 1 the player permission \"playerPermissionsLevel\"; the LayeredAbilities load stores the two tags at [rsi] and [rsi+1]). When the player has no AbilitiesComponent the body calls a no-return error path, so call it only with the live local player of a loaded world. Recipe 0 is the end of the body: `imul rcx,rdx,<pool element size>; movzx eax,byte [rax+rcx+1]; add rsp,..; ret`, then function_start_of. The disp8 +1 is kept: Player::getCommandPermissionLevel has the same end with byte +0 (`movzx eax,byte [rax+rcx]`). The REX and ModRM bytes of the imul and the SIB byte of the movzx are wildcards; the movzx ModRM 44h is kept, because it fixes the form with a disp8, so that byte 11 is the +1 displacement (getCommandPermissionLevel uses ModRM 04h with no displacement). `add rsp` is the epilogue and stays. Recipe 1 is the call in the isOperator wrapper, read as the call target; the compare with 2 (Operator) is kept because it tells this call from other bool wrappers. The REX, ModRM and disp8 of the `mov rcx,[rcx+8]` and the ModRM of the setge are wildcards; `3C` is the al-only form of cmp and stays, because the call result is in al. The two recipes are in two functions and share no bytes. The check is function_start. Layout gate: 26.63 only (entry_is LevelChunk.mSubChunks = 456); on 70.x the function is inlined into its callers, which call the AbilitiesComponent getter and read byte +1 of the component (the isOperator wrapper is `mov rcx,[rcx+8]; lea rdx,[rsp+..]; call getter; mov rax,[rsp+..]; cmp byte ptr [rax+1],2; setge al`); the replacement is Player.getAbilitiesComponent (70.x) plus AbilitiesComponent.mPlayerPermissions (both builds)."

  recipe {
    step "pattern" {
      bytes      = "?? 69 ?? ?? ?? ?? ?? 0F B6 44 ?? 01 48 83 C4 ?? C3"
      literal_ok = ["11: movzx eax,byte [rax+rcx+1]: byte +1 is the player permission; getCommandPermissionLevel has the same end with byte +0 (26.62, 26.63)"]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "?? 8B ?? ?? E8 ?? ?? ?? ?? 3C 02 0F 9D ?? 48 83 C4 ?? C3"
      offset     = 4
      literal_ok = ["10: cmp al,2: PlayerPermissionLevel::Operator, tells the isOperator wrapper from other bool wrappers (26.62, 26.63)"]
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

entry "Player.getAbilitiesComponent" {
  kind     = "function"
  notes    = "gsl::not_null<AbilitiesComponent*>* getAbilitiesComponent(Player const* this, gsl::not_null<AbilitiesComponent*>* out) on 70.x: rcx = the player, rdx = an 8-byte out slot; the function writes the component pointer to *out and returns out in rax. A non-virtual function (an instantiation of the Actor component getter for AbilitiesComponent) with about 190 direct callers. The body reads the EnTT registry at [this+10h] and the entity id at [this+18h] (the entity context at this+8), calls the storage lookup with the AbilitiesComponent type hash 68B75D48h in edx, checks the entity in the sparse set, and stores `page + (index & 7Fh) * 5B8h` to *out. When the player has no AbilitiesComponent it asserts and calls a no-return error path, so call it only with the live local player of a loaded world. It replaces Player.getPlayerPermissionLevel (26.63), which is inlined on 70.x: the player permission level is the byte at AbilitiesComponent.mPlayerPermissions in the returned component. Recipe 0 is the head of the body: `mov rsi,rdx; mov rax,[rcx+10h]; mov ebx,[rcx+18h]; mov rcx,rax; mov edx,68B75D48h; call`, then function_start_of; the hash is kept, because 33 functions share this head with other component hashes; every REX, ModRM and displacement byte is a wildcard. Recipe 1 is the call in the isOperator wrapper (`mov rcx,[rcx+8]; lea rdx,[rsp+..]; call; mov rax,[rsp+..]; cmp byte ptr [rax+1],2; setge al`), read as the call target; the compare with 2 (Operator) is kept, the permission displacement and every register and displacement byte are wildcards. The two recipes are in two functions. The check is function_start (recipe 1 can fail it). Evidence from the 26.63 function: (1) its isOperator wrapper caller (`mov rcx,[rcx+8]; call getPlayerPermissionLevel; cmp al,2; setge al`) is found once on 70.26 with the inlined body, and it calls this function and compares byte +1 of the result with 2; (2) its server caller (`mov rsi,r8; mov rdi,rcx; add rcx,-10h; movzx r8d,byte ptr [r8+10h]; virtual call; test; je; cmp qword ptr [rdi+disp],0; je; call getPlayerPermissionLevel; cmp al,2; jl`) is found on 70.26 with the same head, and it calls this function and does `cmp byte ptr [rax+1],2; jl`; the 26.63 body used the same type hash 68B75D48h, the same entity masks (3FFFFh, shift 0Bh, 7FFh, FFFC0000h) and the same pool element size 5B8h. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552); on 26.63 no out-of-line getter for this component exists (every user inlines the lookup). What changed in 6afa4e47-dadc375797a3d31a: EnTT component lookups call a storage lookup with the hash instead of inlining it, and getPlayerPermissionLevel is inlined into its callers. Class: real code change."

  recipe {
    step "pattern" {
      bytes      = "?? 89 D6 ?? 8B ?? ?? 8B ?? ?? ?? 89 ?? BA 48 5D B7 68 E8"
      literal_ok = [
        "14: AbilitiesComponent type hash 68B75D48h: 33 functions share this head with other component hashes (70.26)",
        "15: as 14",
        "16: as 14",
        "17: as 14",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "?? 8B ?? ?? ?? 8D ?? 24 ?? E8 ?? ?? ?? ?? ?? 8B ?? 24 ?? 80 ?? ?? 02 0F 9D"
      offset     = 9
      literal_ok = ["22: cmp byte ptr [rax+1],2: PlayerPermissionLevel::Operator, tells the isOperator wrapper from other getter-and-compare wrappers (70.26)"]
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "Player.isCreative" {
  kind     = "function"
  notes    = "bool isCreative(Player const* this): rcx = the player, the result in al. A non-virtual function with about 40 direct callers on 26.63 and about 175 on 70.26. The body looks up the player's game type component through the entity context at this+8 (type hash 88D3EDDFh; inlined as `and reg,88D3EDDFh` and a probe loop on 26.63, a call of the storage lookup with the hash in edx on 70.26) and returns true when the type is Creative (1), or when it is Default (5) and the level's default game type, from a virtual call on Actor::mLevel ([this+200h]), is Creative; false when the component is missing. It is the test that GameMode::getMaxPickRange made inline on 26.63 before it chose 12.0 (creative) or 6.7: on 70.26 GameMode.getMaxPickRange is inlined and its callers call this function and then load 12.0 or 6.7 (`call; test al,al; jne; movss xmm0,[6.7]; jmp; movss xmm0,[12.0]`). A mod that called getMaxPickRange(gameMode) calls this with the game mode's player (GameMode.mPlayer) and uses 12.0 when it returns true, else 6.7 (the two constants are the same on 26.63 and 70.26). Recipe 0 is the end of the body after the level virtual call: `mov ecx,eax; mov al,1; cmp reg,1; je; xor reg,5; xor ecx,1; or; sete al`, then function_start_of; the Creative id 1 of `cmp reg,1` is kept, because sibling game-type tests have the same tail with another id (one tests 6); the other immediates, the branch and every register byte are wildcards. Recipe 1 is a call site that is the same on both builds: in a function that opens with `push rsi; push rdi; push rbp; push rbx; sub rsp,..; mov rsi,rdx; mov rdi,rcx; mov rcx,[rcx+disp]; test rcx,rcx; je`, the call right after, read as the call target; the frame size, the displacement and the branch are wildcards. The two recipes are in two functions and share no bytes. The check is function_start (recipe 1 can fail it). No build gate: the function has the same meaning on 26.63 and 70.26. Evidence on 70.26, from the 26.63 GameMode::getMaxPickRange: (1) its three call sites in the item-use-on-actor transaction handler (the one function that loads `, maxPickRange = ` on both builds) are, on 70.26, calls of this function followed by the inline select of 12.0 or 6.7 and the same `ucomiss xmm6,xmm0; jbe` compare; the handler's fourth call (`call; test al,al; mov eax,7.0f; mov ecx,3.0f; cmovne`) calls the 26.63 out-of-line test on 26.63 and this function on 70.26; (2) the 26.63 getMaxPickRange body tests the game type with the same rule (type 1, or type 5 and the level default 1, through the level virtual at 460h, now 468h, the one-slot move that Actor.mLevel saw) and the 26.63 out-of-line test with this body is found by recipe 0 on 26.63. What changed in 6afa4e47-dadc375797a3d31a: the component lookup calls the storage lookup instead of inlining the hash probe; the tail is unchanged. Class: new entry (replacement for GameMode.getMaxPickRange)."

  recipe {
    step "pattern" {
      bytes      = "89 C1 B0 ?? 83 ?? 01 74 ?? 83 ?? ?? 83 ?? ?? 09 ?? 0F 94 C0"
      literal_ok = ["6: cmp reg,1: GameType::Creative; sibling game-type tests have the same tail with another id (26.63, 70.26)"]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "56 57 55 53 48 83 EC ?? 48 89 D6 48 89 CF 48 8B 89 ?? ?? ?? ?? 48 85 C9 74 ?? E8"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
