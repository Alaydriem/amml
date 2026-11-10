entry "ClientInstance.update" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$update@ClientInstance@@QEAA_N_N@Z"
  notes     = "bool ClientInstance::update(bool isInitFinished), the IClientInstance::update override: slot 24 of the ClientInstance.vtable global in 26.51 and slot 23 in 26.60 (one IClientInstance virtual before slot 13 went away; update is 9 slots after requestLeaveGame and 10 after requestLeaveGameAsync in both builds). In 26.60 the function takes a third argument, a bool in r8 (kept in r14d, tested as r14b): update(bool, bool). The msvc_name is the 26.51 signature `(bool)`; the 26.60 name would be `?$update@ClientInstance@@QEAA_N_N_N@Z`. The exe has no RTTI for game classes and no string names this function; it was found through the ClientInstance vtable (string_xref on 'virtual ClientInstance::requestLeaveGameAsync' and 'virtual void __cdecl ClientInstance::requestLeaveGame(bool, bool)' gives the pair of adjacent slots). Recipe 0 anchors on the first virtual call (`mov ebx,edx; mov rsi,rcx; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]; mov byte ptr [rbp+..],imm; xorps xmm0,xmm0`). Recipe 1 shares no bytes with recipe 0: `mov byte ptr [rsi+member],imm; cmp byte ptr [rsi+member],imm; jne rel32; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]` (0x15D bytes in on 26.51, 0xFC on 26.60). Both end with `function_start_of`, so the distance from the anchor to the start does not matter. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60), except two kept bytes in recipe 1: the high bytes 00 00 of the jne, because the same block in another function jumps backwards and these bytes keep the match on update. The checks are `function_start` and `in_vtable` of ClientInstance.vtable, slots 19 to 28: this ties the result to the vtable that holds the requestLeaveGame pair, with a window that covers both builds (24, 23) and about 4 slots either side. What changed in 26.60 (a real code change): the third argument and an xmm6 save make the prologue 10 bytes longer, the NonOwnerPointer asserts after the first call are gone, members moved (2C9h to 2E1h, 29Ch to 2BCh) and indirect calls are followed by a `nop`; the old fixed `vtable_slot 24` gave another function there. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build; real code change): the IClientInstance vtable was reordered and shortened, and update is now slot 10 (4 slots after requestLeaveGame, whose virtual string names it on both builds); its body gained assert calls with source-location strings, its members moved (2E1h to 2F1h, 2BCh to 2CCh, 6F5h to 705h) and the virtual call in recipe 1 uses a disp8 (`mov rax,[rax+40h]`). Recipe 0 still matches once. Recipe 1 is now a pattern_any: alternative 0 is the old block with one more kept byte (byte 3, the 02h of the member displacement: on the Preview another function, with members at B62h/FC4h, has the same block with a forward jne, and this byte keeps alternative 0 off it; the member is 29Ch on 26.51 and 2BCh on 26.60 to 26.63), alternative 1 the Preview block with the disp8 call (the jne bytes wildcarded). The in_vtable window is now 6 to 28 (10 on the Preview, 23 on 26.60 to 26.63, 24 on 26.51, about 4 either side). Evidence for the Preview value: (1) the 26.63 block `mov rcx,[rsi+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov [rbp+..],rsi; mov [rbp+..],rdi; mov rdx,rdi; call; nop; mov byte ptr [rsi+..],..` has exactly two sites on 26.63, both in update, and exactly two on the Preview, both in the candidate; (2) the start of the candidate is the 26.63 start (`mov r14d,r8d; mov ebx,edx; mov rsi,rcx`, the first virtual call with an out pointer, `cmp byte ptr [rsi+..],1`), and the old recipe 1 block (the member store, the member test, the jne and the virtual call on this) is in it with only the disp8 change."

  recipe {
    step "pattern" {
      bytes = "89 D3 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? C6 85 ?? ?? ?? ?? ?? 0F 57 C0"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns   = [
        "C6 86 ?? 02 ?? ?? ?? 80 BE ?? ?? ?? ?? ?? 0F 85 ?? ?? 00 00 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15",
        "C6 86 ?? ?? ?? ?? ?? 80 BE ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15",
      ]
      literal_ok = [
        "0.3: mov byte ptr [rsi+member],imm: the 02h byte of the member displacement; on the Preview another function has the same block with members at B62h/FC4h and a forward jne, and this byte keeps the match on update",
        "0.18: jne rel32 jumps forward to code within 64 KiB (high bytes 0); the same block in another function jumps back, so these bytes keep the match on update",
        "0.19: jne rel32 jumps forward to code within 64 KiB (high bytes 0); the same block in another function jumps back, so these bytes keep the match on update",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ClientInstance.vtable"
    min_slot = 6
    max_slot = 28
  }
}

entry "ClientInstance.getCameraRegistry" {
  kind     = "vslot"
  notes    = "Slot of `Bedrock::NonOwnerPointer<CameraRegistry> ClientInstance::getCameraRegistry()` in the ClientInstance vtable: 190 in 26.51. It returns the 24-byte NonOwnerPointer (the shared_ptr to the control block, then the pointer) through a hidden buffer in rdx. LeviLamina header order: getLevelRendererCameraProxy, getCameraRegistry, getCameraRegistry const, getCameraSystems; MSVC puts the overloads in reverse order, so the const overload is 189 and this one 190; getCameraSystems is 191. The three bodies forward to the game object (slot 168 or 169 of ClientInstance, then 0C20h, 0C18h, 0C28h of that object). One recipe: the CameraPresetsPacket handler (0x131bc60) calls it through the ClientInstance at ClientNetworkHandler+58h (`mov rcx,[rbx+58h]; mov rax,[rcx]; mov rax,[rax+5F0h]; lea rdx,[rbp-18h]; call`); the recipe finds the handler by its unique hand-over to the preset code (`add rbx,158h; mov [rsp+20h],rbx; ...; call`) and then the call in the same function, and reads the displacement as a slot. Other callers of slot 190 use the same call shape as many other slots. Grade: the header order with the overload rule, and the CameraInstructionPacket handler calling getCameraSystems at 191 (5F8h). Every displacement, immediate and branch byte is a wildcard. Recipe 1 is the CameraInstructionPacket handler, which calls getCameraSystems through the same ClientInstance (`mov rdi,[rcx+0C0h]; test rdi,rdi; je; mov rcx,[rcx+58h]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp-..]; call`): it reads that slot and subtracts 1, because the header declares getCameraSystems right after the getCameraRegistry overloads (MSVC order: const, non-const, getCameraSystems). A virtual added between them makes the two recipes disagree, and the entry refuses. What changed in 6ad79ee4-17d51891a1119914: the slot is 188 (two virtuals before it went away; getCameraSystems is 189); the forwarder that v2 first used as recipe 1 has no unique counterpart on 26.60 (the unique match there forwards slot 190, another function). Evidence for 188 on 26.60, from the 26.51 handlers: (1) the CameraPresetsPacket handler calls slot 188 at the place where 26.51 calls 190; (2) the CameraInstructionPacket handler calls slot 189 at the place where 26.51 calls getCameraSystems at 191."

  recipe {
    step "pattern" {
      bytes = "48 81 C3 ?? ?? ?? ?? 48 89 5C 24 ?? 48 89 F1 48 89 C2 4D 89 F8 4D 89 F1 E8"
    }
    step "pattern" {
      bytes  = "48 8B 4B ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B B9 ?? ?? ?? ?? 48 85 FF 0F 84 ?? ?? ?? ?? 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15"
      offset = 23
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" {
      value = -1
    }
  }

  check "range" {
    min = 170
    max = 210
  }
}

entry "ClientInstance.getRegion" {
  kind     = "vslot"
  notes    = "Slot of BlockSource* IClientInstance::getRegion() in the IClientInstance vtable: 30 in 26.51 (LeviLamina header order: the slot before getLocalPlayer, 31). The body locks the weak local-player pointer at this+248h (the same member as getLocalPlayer), finds the Actor through the entity registry, inlines Actor::getDimension (the weak-reference lock loop on Actor::mDimension at 1C8h/1D0h) and then calls getBlockSourceFromMainChunkSource through slot 12 of the dimension (`mov rax,[rsi]; mov rax,[rax+60h]; mov rcx,rsi; call [guard]`); it returns null when there is no player. On 26.51 to 26.63 a former recipe 0 (removed for the Preview, see below) was the end of that body (the two unlock calls of the dimension weak reference, the virtual call on the dimension, `jmp short; xor eax,eax; add rsp; pop rdi; pop rsi; ret`), then function_start_of, then slot_of in ClientInstance.vtable. Recipe 1 is a call site in another class whose member at +8 is the client instance (`mov rbx,r9; mov rdi,r8; mov rsi,rdx; mov rcx,[rcx+8]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test rax,rax; je`): the result is used as a BlockSource (its slot 51, the long clip overload, is called with two Vec3 and a hit-result out pointer); the displacement is turned into a slot. The same class calls slot 194 of that member and reads two floats from the result, which is getViewportInfo in header order: the member is an IClientInstance. The recipes anchor on different functions and share no bytes. Every register-free displacement, immediate and branch byte is a wildcard. The range check keeps the slot in the early part of the vtable, next to getLocalPlayer. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build; real code change): the IClientInstance vtable was reordered and shortened, and getRegion is now slot 12. Its body no longer inlines Actor::getDimension: it locks a weak entity pointer, turns it into the Actor through a helper call, calls getDimension and then the dimension virtual at 68h. Every unique pattern of the new body runs past a call, so the former body recipe 0 was removed (recipes end at a call). The recipes now: recipe 0 is the neighbour route: the getLocalPlayer call site in the function that calls getLocalPlayer twice, then getOptions (const), then a virtual on its result, then tail-jumps with the player (slot 195 on 26.63; on the Preview the same function is reached through the seventh base at this+A0h and loads the ClientInstance vtable from [rcx-A0h]). Its first step finds that function by its end, which has no call in it: `mov rcx,rdi; mov edx,eax; add rsp,..; pop rdi; pop rsi; jmp rel32; add rsp,..; pop rdi; pop rsi; ret` (a pattern_any: alternative 0 for 26.60 to the Preview, alternative 1 for 26.51, which has a `nop` after the jmp). The second step scans that function for the second getLocalPlayer call, which starts right after the first call and ends at its own call (`test rax,rax; je; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`, alternative 0 disp32, alternative 1 disp8), and its displacement is turned into a slot, then `add -1`. It relies on getRegion being the slot right before getLocalPlayer, which holds on every known build (30/31 on 26.51, 29/30 on 26.60 to 26.63, 12/13 on the Preview). Recipe 1 is the call site in another class described above, now ending at its call (`...; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`), as a pattern_any: alternative 0 disp32, alternative 1 disp8 (`mov rax,[rax+60h]`). The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range is now 8 to 45 (12 on the Preview). Evidence for the Preview value: (1) the caller of recipe 1, found on the Preview by a pattern drafted from its 26.63 body after the call (the two Vec3 copies and the BlockSource virtual call on the result; it matches once), reads displacement 60h on its IClientInstance member: slot 12; (2) slot 12 has the old getRegion behaviour (lock the weak entity, resolve the Actor, getDimension, the dimension virtual at 68h as on 26.63, null otherwise), and it is next to slot 13 (getLocalPlayer), which locks the same member, as getRegion and getLocalPlayer did on 26.63 (slots 29 and 30)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 F9 89 C2 48 83 C4 ?? 5F 5E E9 ?? ?? ?? ?? 48 83 C4 ?? 5F 5E C3",
        "48 89 F9 89 C2 48 83 C4 ?? 5F 5E E9 ?? ?? ?? ?? 90 48 83 C4 ?? 5F 5E C3",
      ]
    }
    step "pattern_any" {
      patterns = [
        "48 85 C0 74 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15",
        "48 85 C0 74 ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15",
      ]
      offset   = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" {
      value = -1
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 89 CB 4C 89 C7 48 89 D6 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15",
        "4C 89 CB 4C 89 C7 48 89 D6 48 8B 49 ?? 48 8B 01 48 8B 40 ?? FF 15",
      ]
      offset   = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 8
    max = 45
  }
}

entry "ClientInstance.getFontHandle" {
  kind     = "vslot"
  notes    = "Slot of FontHandle ClientInstance::getFontHandle() const in the ClientInstance vtable: 89 in 26.51 (header order; getRegion is at 30 by the same count, which matches the existing ClientInstance.getRegion entry). The FontHandle (0x50 bytes, a class with a destructor) is returned through a hidden pointer: rcx = this, rdx = the result buffer. The call sites show it: `mov rcx,[rsi+98h]; mov rax,[rcx]; mov rax,[rax+2C8h]; lea rbx,[rbp-58h]; mov rdx,rbx; call [guard]; mov rcx,rbx; call FontHandle::getFont`. Recipe 0 is that call site in a function that measures a text width (0x15632c0). Recipe 1 is the call site in extractNameTags (`mov rcx,[rsi+1168h]; mov rax,[rcx]; mov rax,[rax+2C8h]; lea rdx,[rbp-30h]; call [guard]; mov [rbp+..],rsi; lea rcx,[rbp-30h]; call getFont`). Both read the displacement of the vtable call and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60): the slot is 87 (0x2B8), two lower than on 26.51: two ClientInstance virtuals before it were removed. The `FontHandle::getFont` call after the virtual call is inlined there (the mod does its reads through the FontHandle mirror), so both old patterns, which ran on into `mov rcx,rbx; call getFont`, matched nothing. Both recipes are now a pattern_any with one alternative per build; the first starts at the `mov byte ptr [r14+rbx],0` that ends the string copy before the call, so one offset serves both. Class: real code change (inlined callee, moved slot). Cold run result: failed (both recipes no match)."

  recipe {
    step "pattern_any" {
      patterns = [
        "41 C6 04 1E ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 5D ?? 48 89 DA FF 15",
        "41 C6 04 1E ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15",
      ]
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 8D 4D ?? E8",
        "48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ?? 90 48 8B 45 ?? 48 85 C0 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 0F B6 10 84 D2 75 ?? C7 04 25",
      ]
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 50
    max = 140
  }

}

entry "ClientInstance.getGameConnectionInfo" {
  kind     = "vslot"
  notes    = "Slot of std::optional<Social::GameConnectionInfo> IClientInstance::getGameConnectionInfo() in the IClientInstance vtable: 11 in 26.51 (LeviLamina header order: the slot after setGameConnectionInfo, 10). The optional is returned through a hidden out pointer in rdx. The slot 11 body clears the engaged flag of the out value (`mov byte ptr [rdx+310h],0`), copies the member optional at this+6D8h with the GameConnectionInfo copy constructor when its flag (this+9E8h) is set, and sets the out flag. Three other classes have the same body with other member offsets, so no body anchor is unique. Both recipes are call sites in one screen-controller function whose member at +58h is the client instance (the same function calls slot 79 of that member, getMinecraftGame_DEPRECATED in header order, before the first call). Recipe 0 is the second call: it reads the connection type (`movzx edi,word ptr [rbp+X]`) and tests the engaged flag at X+310h right after the call. Recipe 1 is the first call, after a `test al,al; je` and followed by `cmp byte ptr [rbp+X+310h],0; je`. Both read the displacement and turn it into a slot. The two call sites are in the same function, so they are not fully independent: a rewrite of that function breaks both. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Slot 11 (0Bh) on 26.51, 10 (0Ah) on 26.60: ClientInstance lost a virtual before it (the other ClientInstance slots moved down too). Recipe 0 is now a pattern_any with the nop alternative; recipe 1 now ends at the `call [guard]`. Both are in the same server-identity function (Ruling R17), so recipe 2 was added as an independent route: the getGameConnectionInfo body (`mov rax,rdx; mov byte ptr [rdx+size],0; cmp byte ptr [rcx+9E8h],1; jne; mov rdx,rcx; add rdx,..; ...; call`), then function_start_of and slot_of ClientInstance.vtable. Three other optional getters have the same body with other members, so the two bytes of the member offset 9E8h (the engaged flag of the ClientInstance optional GameConnectionInfo, the same on both builds) are kept with literal_ok. Evidence on 26.60: the call sites, drafted from the old function, read 50h; the body of the old slot 11 (drafted from the old body with 9E8h kept) matches once on 26.60 and sits in slot 10. Class: real code change (slot moved)."

  recipe {
    step "pattern_any" {
      patterns = ["48 8B 4E ?? 48 8B 01 48 8B 40 ?? 48 8D 95 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 0F B7 BD ?? ?? ?? ?? 80 BD ?? ?? ?? ?? ?? 75", "48 8B 4E ?? 48 8B 01 48 8B 40 ?? 48 8D 95 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 0F B7 BD ?? ?? ?? ?? 80 BD ?? ?? ?? ?? ?? 75"]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "84 C0 0F 84 ?? ?? ?? ?? 48 8B 4E ?? 48 8B 01 48 8B 40 ?? 48 8D 95 ?? ?? ?? ?? FF 15"
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes      = "48 89 D0 C6 82 ?? ?? ?? ?? ?? 80 B9 E8 09 ?? ?? ?? 75"
      literal_ok = [
        "12: cmp byte ptr [rcx+9E8h],1, the engaged flag of ClientInstance's optional GameConnectionInfo: three other optional getters have the same body with other members (150h, 1D90h, 0CE8h) (26.51 and 26.60)",
        "13: the high byte of the same member offset 9E8h (26.51 and 26.60)",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  check "range" {
    min = 3
    max = 19
  }
}

entry "ClientInstance.onPlayerLoaded" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onPlayerLoaded@ClientInstance@@QEAAXAEAVPlayer@@@Z"
  notes     = "void ClientInstance::onPlayerLoaded(Player& player), the IClientInstance::onPlayerLoaded override: slot 327 of the ClientInstance.vtable global in 26.51 and 323 in 26.60. LeviLamina raises ClientJoinLevelEvent from a hook on this function. It is a 33-byte forwarder, `mov r8,rdx; mov rdx,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+470h]; mov r9,[guard]; jmp r9`, that is minecraftGame->onPlayerLoaded(*this, player) (member 1A8h in 26.51, 1C8h in 26.60). It is a leaf with no stack frame, so it has no .pdata entry: `function_start` and `function_start_of` cannot be used. Recipe 0 is the whole body with the member, the vtable offset and the guard wildcarded; the lint decodes it from its first byte, because the recipe's value is the match start. On 26.51 to 26.63 a recipe 1 (removed for the Preview, see below) shared no bytes with recipe 0: it starts in the .pdata function just before the neighbours and runs through them, the last bytes of a funclet (`pop rsi; pop rbp; ret`), the fadeOutMusic forwarder, five int3, the 8-byte setter `mov byte ptr [rcx+member],imm; ret` (onFullVanillaPackOnStack) and eight int3, the 8-byte getter `movzx eax,byte ptr [rcx+member]; ret` (isFullVanillaPackOnStack) and eight int3; it gives the address 67 bytes after its start, the first byte after that padding. It relies on this link order and padding, which is the same in both builds. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both were unique in both exes and agreed. The checks are `section .text` and `in_vtable`: the result must be in slots 319 to 331 of ClientInstance.vtable (both builds plus about 4 slots either side). What changed in 26.60: the slot moved from 327 to 323 and the members moved (1A8h to 1C8h, 1B8h to 1D8h); the code shape did not change. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build): the IClientInstance vtable was reordered and shortened, and onPlayerLoaded is now slot 296; its member is 1D0h. Recipe 0 still matches once. The former recipe 1 (the link-order run that started in the funclet before the fadeOutMusic forwarder) was removed: on the Preview the funclet ends 15 int3 earlier, so the run must start in the padding, where the lint cannot decode it, and the only way to give both shapes the same offset would run the old alternative past a call. The entry now has one recipe; its in_vtable check is strong, so the deriver needs no second recipe. The in_vtable window is now 292 to 331 (296 on the Preview, 323 on 26.60 to 26.63, 327 on 26.51, about 4 either side). Evidence for the Preview value: (1) the whole 26.63 body (recipe 0) matches once, in slot 296; (2) the link-order neighbours of 26.63 (the fadeOutMusic forwarder through vtable offset 468h, the 8-byte setter, the 8-byte getter and their int3 padding) are in the same order right before it, and slots 295 and 296 line up with 26.63 slots 322 and 323. The window was widened on that evidence."

  recipe {
    step "pattern" {
      bytes = "49 89 D0 48 89 CA 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 8B 0D ?? ?? ?? ?? 49 FF E1"
    }
  }

  check "section" {
    name = ".text"
  }

  check "in_vtable" {
    vtable   = "ClientInstance.vtable"
    min_slot = 292
    max_slot = 331
  }
}

entry "ClientInstance.getLocalPlayer" {
  kind     = "vslot"
  notes    = "Slot of LocalPlayer* IClientInstance::getLocalPlayer() const in the IClientInstance vtable: 31 in 26.51 (the same as LeviLamina's header order; header order, not a reference) and 30 in 26.60 (IClientInstance lost one slot before 13). The function locks the weak player pointer at this+248h and returns the player or null; the slot before it (getRegion) reads the same member. On 26.51 to 26.63 a former recipe 0 was a call site in another ClientInstance virtual (slot 197 in 26.51): `push rdi; sub rsp,..; mov rsi,rcx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test rax,rax`, a call on `this`; the displacement is turned into a slot. The former recipe 1 (removed for the Preview, see below) was the body (stack cookie, `lea rdx,[rcx+member]`, the weak-pointer lock call, `cmp byte ptr [rsp+..],imm`, the second call) and `slot_of` in the ClientInstance.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both were unique in both exes and agreed. The range check keeps the slot in the early part of the vtable. The entry did not break in the 26.60 cold derive. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build; real code change): the IClientInstance vtable was reordered and shortened, and getLocalPlayer is now slot 13. Its body changed: it locks a weak entity pointer (the same member as getRegion, slot 12), tests that the entity has one component (type hash ACB47B38h) and only then returns the Actor. Slot 22 on the Preview has the old getLocalPlayer body byte for byte (only its member differs), so the former body recipe matched slot 22 there and gave a wrong value; it was removed. The recipes now (each pattern ends at a call or has none): recipe 0 is the call site in the function that calls getLocalPlayer twice, then getOptions (const), then a virtual on its result, then tail-jumps with the player (slot 195 on 26.63; on the Preview the same function is reached through the seventh base at this+A0h and loads the ClientInstance vtable from [rcx-A0h]). Its first step finds that function by its end, which has no call in it: `mov rcx,rdi; mov edx,eax; add rsp,..; pop rdi; pop rsi; jmp rel32; add rsp,..; pop rdi; pop rsi; ret` (a pattern_any: alternative 0 for 26.60 to the Preview, alternative 1 for 26.51, which has a `nop` after the jmp). The second step scans that function for the second getLocalPlayer call, which starts right after the first call and ends at its own call (`test rax,rax; je; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`, alternative 0 disp32, alternative 1 disp8), and its displacement is turned into a slot. Recipe 1 is the first call in a large function that calls getLocalPlayer, isShowingMenu and getOptions (const) on rsi in a row. Its first step finds that function by its prologue body, which has no call (`mov r14,r9; mov rsi,r8; mov rbx,rdx; mov rdi,rcx; mov rax,[rcx+..]; test rax,rax; je; cmp qword ptr [rax+..],imm; jne rel32`, offset 26 at the jne); `operand` reads the jne target, which is the start of the getLocalPlayer call (`mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`), `add 3` steps over the 3-byte `mov rax,[rsi]`, and the displacement of the next instruction is turned into a slot. The two recipes anchor in two functions. Every displacement, immediate and branch byte is a wildcard. The range is now 9 to 45 (13 on the Preview). Evidence for the Preview value: (1) the 26.63 function of recipe 0 (two getLocalPlayer calls, getOptions at 173, a virtual on its result, a tail jump with the player) is found on the Preview with the same calls, and it calls slot 13 twice and getOptions at 149 (the Preview getOptions pair is 148/149); (2) the large function of recipe 1, drafted from 26.63, calls slot 13, then slot 125 (isShowingMenu on the Preview) and slot 149, in the same order as 30, 148 and 173 on 26.63; and slot 13 sits next to getRegion (slot 12) and reads the same member, as on 26.63."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 F9 89 C2 48 83 C4 ?? 5F 5E E9 ?? ?? ?? ?? 48 83 C4 ?? 5F 5E C3",
        "48 89 F9 89 C2 48 83 C4 ?? 5F 5E E9 ?? ?? ?? ?? 90 48 83 C4 ?? 5F 5E C3",
      ]
    }
    step "pattern_any" {
      patterns = [
        "48 85 C0 74 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15",
        "48 85 C0 74 ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15",
      ]
      offset   = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "4D 89 CE 4C 89 C6 48 89 D3 48 89 CF 48 8B 41 ?? 48 85 C0 74 ?? 48 83 78 ?? ?? 0F 85 ?? ?? ?? ??"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "add" {
      value = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 9
    max = 45
  }
}

entry "ClientInstance.getTopScreenName" {
  kind     = "vslot"
  notes    = "Slot of std::string IClientInstance::getTopScreenName() const in the IClientInstance vtable: 136 in 26.51 (the same as the header order; header order, not a reference) and 134 in 26.60. The string comes back through a hidden return pointer in rdx; the function keeps rdx (`mov rsi,rdx`) and reads the scene stack through another virtual. Recipe 0 is the call site in the lambda of ClientInstance::_handlePossibleControllerDisconnect(bool) (its source-location string names it; '(anonymous class)' in 26.51, '(lambda)' in 26.60): the lambda locks a weak ClientInstance pointer, tests a flag, calls `client->getTopScreenName()` into a stack string and compares the result with \"progress_screen\". The pattern is the `je` after the flag test and that call (`je rel32; mov rax,[rbx]; mov rax,[rax+disp]; lea rdi,[rbp+..]; mov rcx,rbx; mov rdx,rdi; call [guard]`); the displacement is turned into a slot. It is the only call through this displacement that passes a return slot to a ClientInstance. Recipe 1 is the body (prologue with xmm6 save, `mov rsi,rdx; mov rdi,rcx`, the first virtual call on this with an out pointer) and `slot_of` in the ClientInstance.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. What changed in 26.60: the slot moved, and the flag test changed form (`test byte ptr [rax+member],1` became `movzx` + `test al,1`); recipe 0 now starts after it. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build; real code change): the IClientInstance vtable was reordered and shortened, and getTopScreenName is now slot 113. The lambda keeps the client in rcx (`mov rcx,[rbp+..]; test al,1; je; mov rax,[rcx]; mov rax,[rax+disp]; mov rdi,rbp; mov rdx,rdi; call [guard]`), and the old recipe 0 pattern matched another function on the Preview (after `test rbx,rbx`, a call at slot 277 that returns a string from another object). Recipe 0 is now a pattern_any, all alternatives with the vtable load at byte 11: alternative 0 (26.60 to 26.63) starts at the flag test `test al,imm`; alternative 1 (26.51) starts two bytes before the je, at the ModRM and displacement of `mov rbx,[rbp+..]` (5Dh, which the other function does not have: it has `test rbx,rbx` there); alternative 2 is the Preview form. Recipe 1 is a pattern_any: alternative 0 the old body, now ending at the first call, alternative 1 the Preview body without the xmm6 save. The range is now 109 to 145 (113 on the Preview). Evidence for the Preview value: (1) the lambda, found on the Preview by a pattern drafted from its 26.63 body after the call (the compare of the result with a 15-character string, matches once), calls displacement 388h: slot 113; (2) the 26.63 body (the two virtual calls on this with out pointers at [rbp-28h] and [rbp-10h]) is slot 113 on the Preview, and its vtable neighbours line up with 26.63 slots 133 to 137 (a forwarder after it, onClientCreatedLevel two slots after, which names itself on the Preview)."

  recipe {
    step "pattern_any" {
      patterns = [
        "A8 ?? 0F 84 ?? ?? ?? ?? 48 8B 03 48 8B 80 ?? ?? ?? ?? 48 8D 7D ?? 48 89 D9 48 89 FA FF 15",
        "5D ?? 0F 84 ?? ?? ?? ?? 48 8B 03 48 8B 80 ?? ?? ?? ?? 48 8D 7D ?? 48 89 D9 48 89 FA FF 15",
        "A8 ?? 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 EF 48 89 FA FF 15",
      ]
      offset   = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 D6 48 89 CF 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15",
        "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 D6 48 89 CF 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15",
      ]
    }
    step "slot_of" {
      vtable = "ClientInstance.vtable"
    }
  }

  check "range" {
    min = 109
    max = 145
  }
}

entry "ClientInstance.isShowingMenu" {
  kind     = "vslot"
  notes    = "Slot of bool IClientInstance::isShowingMenu() const in the IClientInstance vtable: 150 in 26.51 (the same as the header order; header order, not a reference) and 148 in 26.60. The slot after it (isShowingPauseScreen) tests the top screen name against \"pause_screen\", and isInWorldAndNotShowingAnyMenuScreens (158 in 26.51, 156 in 26.60) returns `!this->isShowingMenu()` when a world is loaded. Recipe 0 is that call: `test al,al; je; xor esi,esi; jmp; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`; the displacement is turned into a slot. Recipe 1 is the body (prologue, `mov rdi,rcx; mov byte ptr [rbp+..],imm`, the scene-stack call with the out pointer `lea rsi,[rbp-20h]`) and `slot_of` in the ClientInstance.vtable global. One byte is kept: the out-pointer slot 20h, because 26.60 has a twin of this function outside the ClientInstance vtable with its locals at [rbp-38h]; without the byte the pattern matches both. Recipe 2 is a call site in a large function that calls three IClientInstance virtuals on rsi in a row: getLocalPlayer (`...; call [guard]; mov r13,rax`), then isShowingMenu (`test al,al; jne`), then getOptions; the middle displacement is turned into a slot. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60) apart from the kept byte. All three are unique in both exes and agree. What changed in 26.60: the slot moved, a `nop` follows the call in recipe 0's site, and the twin appeared. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build; real code change): the IClientInstance vtable was reordered and shortened, and isShowingMenu is now slot 125. Recipe 0 still matches once. The body now pushes rbp, rsi, rbx and keeps this in rsi, and its out pointer is [rbp+10h]; a twin outside the ClientInstance vtable (two direct callers) has the same code with [rbp-40h]. Recipe 1 is now a pattern_any: alternative 0 the old body, alternative 1 the Preview body, with byte 46 kept (the 10h out-pointer slot) to leave out that twin. Recipe 2 now ends at its call: its first step finds the large function by its prologue body, which has no call (`mov r14,r9; mov rsi,r8; mov rbx,rdx; mov rdi,rcx; mov rax,[rcx+..]; test rax,rax; je; cmp qword ptr [rax+..],imm; jne rel32`, unique on every known build); the second step scans that function for `mov r13,rax; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]` (it starts right after the getLocalPlayer call and ends at the isShowingMenu call; one match in the function on every known build), and the displacement at offset 6 is turned into a slot. The same pattern serves every build, so it needs no alternatives. The range is now 121 to 165 (125 on the Preview). Evidence for the Preview value: (1) the large function of recipe 2, drafted from 26.63 (getLocalPlayer, then isShowingMenu, then getOptions on rsi), is found once on the Preview and reads displacement 3E8h for the middle call: slot 125; (2) the 26.63 body (the scene-stack call with an out pointer, then the assert) is the Preview slot 125, and the slots around it line up with 26.63 (148 maps to 125; 149 to 154, the five stack-cookie getters, map to 126 to 131)."

  recipe {
    step "pattern" {
      bytes  = "84 C0 74 ?? 31 F6 EB ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns   = [
        "55 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CF C6 45 ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 75 E0 48 89 F2 FF 15",
        "55 56 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE C6 45 ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 10 FF 15",
      ]
      literal_ok = [
        "0.41: lea rsi,[rbp-20h]: the out-pointer slot; a twin of this function outside the ClientInstance vtable (26.60) uses [rbp-38h], so this byte keeps the match on isShowingMenu",
        "1.46: lea rdx,[rbp+10h]: the out-pointer slot; a twin of this function outside the ClientInstance vtable (Preview 1.26.70.23) uses [rbp-40h], so this byte keeps the match on isShowingMenu",
      ]
    }
    step "slot_of" {
      vtable = "ClientInstance.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "4D 89 CE 4C 89 C6 48 89 D3 48 89 CF 48 8B 41 ?? 48 85 C0 74 ?? 48 83 78 ?? ?? 0F 85 ?? ?? ?? ??"
    }
    step "pattern" {
      bytes  = "49 89 C5 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 121
    max = 165
  }
}

entry "ClientInstance.isWorldActive" {
  kind     = "vslot"
  notes    = "Slot of bool IClientInstance::isWorldActive() const in the IClientInstance vtable: 159 in 26.51 (the same as the header order; header order, not a reference) and 157 in 26.60. The function is `return this->getLevel() != nullptr` (the const getLevel overload; MSVC puts overloads in the vtable in reverse declaration order), and the next slot (isInRealm) forwards to the MinecraftGame. Recipe 0 is a call on `this` at the start of another ClientInstance virtual (slot 128 in 26.51): `mov qword ptr [rbp+..],-2; mov rdi,rdx; mov rbx,rcx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test al,al`; the displacement is turned into a slot. Recipe 1 is the whole 31-byte body and the start of the forwarder after it (`int3`, `mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,[guard]; jmp rdx`); the body alone has three matches. `slot_of` reads the ClientInstance.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. The entry did not break in the 26.60 cold derive. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build): the IClientInstance vtable was reordered and shortened, and isWorldActive is now slot 134. Both recipes still match once and agree; only the range moved, to 130 to 170 (134 on the Preview). Evidence for the Preview value: (1) the call site of recipe 0, drafted from the 26.63 caller, and (2) the body of recipe 1 with the isInRealm forwarder after it, drafted from 26.63, both give 134; the slots around it line up with 26.63 (157 and 158 map to 134 and 135, and 161 to 163 to 137 to 139, the same leaf and forwarder pattern). The range was widened on that evidence."

  recipe {
    step "pattern" {
      bytes  = "48 C7 45 ?? ?? ?? ?? ?? 48 89 D7 48 89 CB 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes = "48 83 EC ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 85 C0 0F 95 C0 48 83 C4 ?? C3 CC 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 48 FF E2"
    }
    step "slot_of" {
      vtable = "ClientInstance.vtable"
    }
  }

  check "range" {
    min = 130
    max = 170
  }
}

entry "ClientInstance.getOptions" {
  kind     = "vslot"
  notes    = "Slot of IOptionRegistry& IClientInstance::getOptions() in the IClientInstance vtable: 174 in 26.51 and 172 in 26.60. The header declares the non-const getOptions at 174 and the const one at 175 (header order, not a reference). Both slots hold the same function, `mov rax,[rcx+member]; ret` (0E10h in 26.51, 0E00h in 26.60), so a call through either returns the same object; `slot_of` gives the first slot that holds it, and recipe 0's call site uses that one. Recipe 0 is a call on `this` in a ClientInstance virtual (slot 411 in 26.51): `sub rsp,..; cmp byte ptr [rcx+member],imm; jne; mov rax,[rcx]; mov rax,[rax+disp]; mov rsi,rcx; call [guard]`; the displacement is turned into a slot. Recipe 1 is the 8-byte body `mov rax,[rcx+member]; ret`, its eight int3 and the prologue of the next function (`push rsi; push rdi; sub rsp,..; mov rax,[rip+..]`), and `slot_of` in the ClientInstance.vtable global. The body is a leaf without .pdata; the lint decodes it from its first byte because `slot_of` finds that byte in the vtable. It relies on the next function following the body, as in both builds. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. The entry did not break in the 26.60 cold derive. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23) (an assert-enabled build): the IClientInstance vtable was reordered and shortened, and getOptions is now slot 148 (the const overload 149 holds the same body). Both recipes still match once and agree; only the range moved, to 144 to 185 (148 on the Preview). Evidence for the Preview value: (1) the call site of recipe 0 and (2) the body of recipe 1, both drafted from 26.63, give 148; the slots around it line up with 26.63 (166 to 177 map to 142 to 153, with the same pairs of equal slots at 166/167 and 172/173, and getSettingsRegistry, which names itself on the Preview, at 154); and two callers of getOptions (const) on 26.63 call 149 on the Preview. The range was widened on that evidence."

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 80 B9 ?? ?? ?? ?? ?? 75 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 CE FF 15 ?? ?? ?? ??"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? C3 CC CC CC CC CC CC CC CC 56 57 48 83 EC ?? 48 8B 05"
    }
    step "slot_of" {
      vtable = "ClientInstance.vtable"
    }
  }

  check "range" {
    min = 144
    max = 185
  }
}

entry "ClientInstance.getLevel" {
  kind     = "vslot"
  notes    = "Slot of ClientLevel* ClientInstance::getLevel() in the ClientInstance vtable: 168 on 26.51 (168 and 169 hold one body: the non-const and const overloads). Found by header order: in the 26.51 LeviLamina header the known slots getRegion 30, isShowingMenu 150, isWorldActive 159 and getOptions 174 are at their header positions, and getLevel is header 168/169. The body is a leaf (no .pdata): `mov rax,[rcx+1B0h]; mov rax,[rax+0C8h]; test; je; cmp byte ptr [rax+28h],1; jne; mov rcx,[rax+30h]; test; je; cmp byte ptr [rcx],1; jne; mov rax,[rax+40h]; ret`. Recipe 0 is that body at its first byte, then slot_of in ClientInstance.vtable (the first slot that holds it). Recipe 1 is the neighbour route: ClientInstance.isWorldActive (159, its own recipes) plus 9, the header distance (isInRealm 160, then 7 virtuals, then getLevel). Every displacement and branch byte is a wildcard. Range window 160..176."

  recipe {
    step "pattern" { bytes = "48 8B 81 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 80 78 ?? ?? 75 ?? 48 8B 48 ?? 48 85 C9 74 ?? 80 39 ?? 75 ?? 48 8B 40 ?? C3" }
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "ref" { entry = "ClientInstance.isWorldActive" }
    step "add" { value = 9 }
  }

  check "range" {
    min = 160
    max = 176
  }
}

entry "ClientInstance.isInRealm" {
  kind     = "vslot"
  notes    = "Slot of bool ClientInstance::isInRealm() in the ClientInstance vtable: 160 on 26.51 (the 26.51 header position, the slot after isWorldActive 159). The body is a leaf that forwards to the IMinecraftGame at this+1A8h, slot 102 (330h): `mov rcx,[rcx+1A8h]; mov rax,[rcx]; mov rax,[rax+330h]; mov rdx,[guard]; jmp rdx`. IMinecraftGame::isInRealm is header 108; the known IMinecraftGame slots (grabMouse 135 = header 141) put it at 102, so the forward target proves the slot. Recipe 0 is that body, then slot_of in ClientInstance.vtable. The 330h displacement is kept: 50 leaf forwarders have the same shape with other slots. Recipe 1 is ClientInstance.isWorldActive plus 1. Range window 152..168."

  recipe {
    step "pattern" {
      bytes      = "48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 30 03 00 00 48 8B 15 ?? ?? ?? ?? 48 FF E2"
      literal_ok = [
        "13: IMinecraftGame slot 102 (330h) isInRealm: other ClientInstance forwarders call other slots",
        "14: IMinecraftGame slot 102 (330h), second byte",
        "15: upper bytes of the 330h displacement",
        "16: upper bytes of the 330h displacement",
      ]
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "ref" { entry = "ClientInstance.isWorldActive" }
    step "add" { value = 1 }
  }

  check "range" {
    min = 152
    max = 168
  }
}

entry "ClientInstance.getItemInHandRenderer" {
  kind     = "vslot"
  notes    = "Slot of ItemInHandRenderer* ClientInstance::getItemInHandRenderer() in the ClientInstance vtable: 353 on 26.51. The getters 349..357 line up with the header run getBlockTessellator .. getDevConsoleMessageHistory: 349 returns this+610h (the BlockTessellator that the ClientInstance init stores there), 351 copies a shared_ptr (getEntityRenderDispatcher), 352..355 are `mov rax,[rcx+disp]; ret` leaves and 356/357 return `lea` of a deque (the two message histories). 353 returns this+638h. Recipe 0 starts at getBlockTessellator and covers the run of bodies in memory: two member getters, the shared_ptr getter (getEntityRenderDispatcher, `mov rax,rdx; xorps; movups [rdx],xmm0; mov rdx,[rcx+..]; ..; ret`), then two member getters, each padded with int3; then slot_of in ClientInstance.vtable (getBlockTessellator) and add 4: this slot is the second getter after the shared_ptr getter on both builds. Recipe 1 is the render context helper that LevelRenderer.renderLevel calls first (the first direct call in it): it fills the context from ClientInstance getters, and this getter fills +50h (`mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; mov [r64+50h],rax`); one pattern_any alternative per shape (alternative 1 has the nop that 26.60 adds after the call); the store displacement 50h is the one kept byte; the displacement is read as a slot. Range window 345..361. Value 353 (161h) on 26.51, 349 (15Dh) on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): ClientInstance lost 4 virtuals before this run, and the member getter of slot 350 is folded with an identical body elsewhere, so the 26.51 run (three member getters and two lea getters from this slot) now starts one slot early and recipe 0 gave 348: a wrong value, which the weak-check rule refused. The 26.51 call site of recipe 1 (in ClientInputCallbacks.handleBuildAction) has another shape. The slot was accepted on two facts from 26.51: the vtable neighbours (the shared_ptr getter, then two member getters with the same 8-byte steps: 26.51 351, 352, 353 return +628h, +630h, +638h; 26.60 347, 348, 349 return +640h, +648h, +650h), and the render context helper that calls it right after the slot before it and stores the result at +50h on both builds. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes = "48 8B 81 ?? ?? ?? ?? C3 CC CC CC CC CC CC CC CC 48 8B 81 ?? ?? ?? ?? C3 CC CC CC CC CC CC CC CC 48 89 D0 0F 57 C0 0F 11 02 48 8B 91 ?? ?? ?? ?? 48 85 D2 74 ?? F0 FF 42 ?? 48 8B 91 ?? ?? ?? ?? EB ?? 31 D2 48 8B 89 ?? ?? ?? ?? 48 89 08 48 89 50 ?? C3 CC CC CC CC CC CC CC CC CC CC CC CC CC 48 8B 81 ?? ?? ?? ?? C3 CC CC CC CC CC CC CC CC 48 8B 81 ?? ?? ?? ?? C3"
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
    step "add" { value = 4 }
  }

  recipe {
    step "ref" { entry = "LevelRenderer.renderLevel" }
    step "call" { nth = 0 }
    step "pattern_any" {
      patterns   = [
        "48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 49 89 46 50",
        "48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 90 49 89 47 50",
      ]
      literal_ok = [
        "0.22: the render context member +50h that this getter fills: the getters before and after it fill +48h and +58h",
        "1.23: the same member in the 26.60 shape",
      ]
      offset     = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 345
    max = 361
  }
}

entry "ClientInstance.getPacketSender" {
  kind     = "vslot"
  notes    = "Slot of PacketSender& IClientInstance::getPacketSender() in the ClientInstance vtable: 293 on 26.51 (LeviLamina header order; the neighbours line up: 288 and 289 set and get one int member, 292 getInProgressBAI returns this+290h, 294 and 295 hold one body, the getClientNetworkSystem const pair). The body is a leaf that returns a member (`mov rax,[rcx+1D0h]; ret`). The callers then call slot 2 of the result with a packet in rdx (PacketSender::send). Recipe 0 is the ClientInstance function that opens the inventory: it builds an InteractPacket on the stack (action 6, OpenInventory; the runtime id from Actor::getRuntimeID; serialization mode 5), then `mov byte ptr [rbp-..],0; mov dword ptr [rbp],5; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]` and sends it. Recipe 1 is a send in another client function: `mov byte ptr [rdi+rbx],..; mov rdi,[rbp+..]; mov rax,[rdi]; mov rax,[rax+disp]; mov rcx,rdi; call [guard]`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 285..301. Recipe 0 is now a pattern_any that runs on from the send to the code after it, one alternative per shape: alternative 0 (26.51) `..; call [guard]` (the send) `; nop; add rsp,..`; alternative 1 (26.60) the first of two identical send blocks in the function, picked by what follows it (`call; mov rdi,rax; call; cmp rdi,imm32; je near`; the second block has a short je). Recipe 1 is now a send in another client function: `mov [rsp+..],eax; mov dword ptr [rsp+..],imm; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`, then `mov rcx,[rax]; mov r8,[rcx+10h]` (slot 2 of the result, PacketSender::send) on both builds. Value 293 on 26.51, 290 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): ClientInstance lost 3 virtuals before this slot; the open-inventory function has the send twice, and the 26.51 site of recipe 1 changed shape. The slot was accepted on two facts from 26.51: the vtable neighbours (26.51 288..295 and 26.60 285..292 hold the same bodies: the int setter and getter, two forwarders, getInProgressBAI, this leaf, the const pair; the members moved by +18h to +20h), and the InteractPacket send of the open-inventory function (action 6, mode 5) calls it on both builds. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? 90 48 81 C4",
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C7 E8 ?? ?? ?? ?? 48 81 FF ?? ?? ?? ?? 0F 84",
      ]
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
      bytes  = "89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 285
    max = 301
  }
}

entry "ClientInstance.vtable" {
  kind     = "global"
  notes    = "The vtable that holds the IClientInstance virtuals (slot 0 is the destructor; requestLeaveGameAsync and requestLeaveGame are 14 and 15 in 26.51, 13 and 14 in 26.60; update is 24 and 23). The ClientInstance constructor stores five vtable pointers (`lea rax,[rip+..]; mov [r13+0/18h/20h/88h/90h],rax`); the `lea` of the store to this+0 is this vtable. The recipe is one pattern_any: alternative 0 is that run with r13 as the object register (26.51 to 26.63), alternative 1 the same run with r12 (`lea rax,[rip+..]; mov [r12],rax; ... mov [r12+90h],rax`, each store with a SIB byte); the displacements are wildcarded and the operand is the RIP target of the first `lea`. Each alternative matches once on its own builds and never on the others. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). It is unique in both exes. The section check keeps the address inside .rdata. It exists so `slot_of` and `in_vtable` have a vtable to read. Values: 0xe9731b0 in 26.51, 0xf2b9330 in 26.60. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the constructor keeps the object in r12 instead of r13, so every store gained a SIB byte and the old pattern found nothing; the run now stores seven vtables (this+0, 18h, 20h, 88h, 90h, 98h, A0h). Alternative 1 was added for that shape (would have survived with a better recipe). Evidence for the Preview value: (1) the constructor drafted from the 26.63 one (prologue, the 18h-byte allocation with the 100000001h ref counts, the first run of three stores, `mov edx,2; call`, then the long run) is found again with only the register changed, and the first `lea` of its long run names this table; (2) the destructor, the only other code that loads the table, has the same shape as on 26.63 (`mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; ... mov [rcx+88h],rax`), and both builds have exactly these two references. The slot contents line up with 26.63 (requestLeaveGame names itself in both tables, and the neighbours of every slot below line up)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 05 ?? ?? ?? ?? 49 89 45 ?? 48 8D 05 ?? ?? ?? ?? 49 89 45 ?? 48 8D 05 ?? ?? ?? ?? 49 89 45 ?? 48 8D 05 ?? ?? ?? ?? 49 89 85 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 85 ?? ?? ?? ??",
        "48 8D 05 ?? ?? ?? ?? 49 89 04 24 48 8D 05 ?? ?? ?? ?? 49 89 44 24 ?? 48 8D 05 ?? ?? ?? ?? 49 89 44 24 ?? 48 8D 05 ?? ?? ?? ?? 49 89 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 84 24 ?? ?? ?? ??",
      ]
      offset = 0
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

entry "ClientInstance.getMouseGrabbed" {
  kind     = "vslot"
  notes    = "Slot of bool IClientInstance::getMouseGrabbed() const in the IClientInstance vtable: 147 in 26.51 (LeviLamina's IClientInstance header order) and 145 in 26.60. The function returns `isPrimaryClient() && minecraftGame->getMouseGrabbed()`; the slots before it are useController and useTouchscreen, the one after it currentInputModeIsMouseAndKeyboard. Recipe 0 is in a GuiData function that tests getMouseGrabbed and then useTouchscreen on the client held at [rdi+98h]: the end of the null-pointer trap before it and the call (`mov dword ptr [0],imm; mov rdi,[rbp+..]; mov rcx,[rdi+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`). Recipe 1 is a second GuiData function with the same test at its start (`mov rsi,rcx; mov rcx,[rcx+member]; ...; call [guard]; test al,al; je`, then useTouchscreen on the same member). Both displacements are turned into slots. Recipe 2 is the body of ClientInstance::getMouseGrabbed (a 64-byte function that calls isPrimaryClient and tail-calls the MinecraftGame) and `slot_of` in the ClientInstance.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). All three are unique in both exes and agree. The range check keeps the slot near the input-mode queries. What changed in 26.60: the slot moved, a `nop` follows the call in recipe 0's site, and the old recipe 1 site was rewritten."

  recipe {
    step "pattern" {
      bytes  = "C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 7D ?? 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 25
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes = "56 48 83 EC ?? 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 48 83 C4 ?? 5E 48 FF E2 31 C0 48 83 C4 ?? 5E C3"
    }
    step "slot_of" {
      vtable = "ClientInstance.vtable"
    }
  }

  check "range" {
    min = 130
    max = 170
  }
}

entry "ClientInstance.GameCallbacks.vtable" {
  kind     = "global"
  notes    = "The GameCallbacks vtable of ClientInstance, at this+88h (destructor, onLevelCorrupt, onGameModeChanged, onBeforeSimTick, onTick, onInternetUpdate, onGameSessionReset, onLevelExit (7), onRequestResourceReload, updateScreens). It is the fourth vtable that the ClientInstance constructor stores (the run that also stores ClientInstance.vtable at this+0; five stores in 26.51, six in 26.60). The pattern starts before the run: `mov dword ptr [reg+..],imm; mov edx,imm; call; movups [reg+..],xmm`, then the first five `lea rax,[rip+..]; mov [reg+..],rax` stores, with every REX, ModRM, displacement and immediate byte wildcarded, so the registers are free; it is unique in both exes. Offset 59 gives the `lea` of the fourth store, whose RIP target is the operand. The section check keeps the address inside .rdata. It exists for the `in_vtable` check of ClientExitLevel.hook; it was added in Plan 5b. Values: 0xe973f20 in 26.51, 0xf2ba080 in 26.60, 0xf2bb060 in 26.63, 0x12fbbaa0 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed in 6ae4eeee-4dd4fa7acacb4c41: the constructor holds `this` in r12 instead of r13, so every store through it has a SIB byte (`mov [r12],rax` 4 bytes, `mov [r12+18h],rax` 5 bytes, `mov [r12+88h],rax` 8 bytes, against 4, 4 and 7 on 26.63), and the run has a seventh store (+0A0h); the fixed-length pattern found no match. The recipe is now one `pattern_any`: alternative 0 is the old pattern (26.51 to 26.63); alternative 1 is the same sequence for the SIB encoding (`mov edx,imm; call; movups [r12+..],xmm6`, then the `lea rax,[rip+..]; mov [r12+..],rax` stores), and it starts inside the instruction before `mov edx,imm` (the wildcarded tail of `mov dword ptr [r12+38h],0`) so that the `lea` of the fourth store is at offset 59 in both alternatives. Alternative 0 matches once on 26.51, 26.60, 26.62 and 26.63 and not on the Preview; alternative 1 matches once on the Preview and on no other build. Evidence on the Preview, both from 26.63: (1) the 26.63 constructor run (base vtable at this+0, +18h, +20h after the `mov edx,2; call`, then the ClientInstance stores, the fourth at this+88h) is found once on the Preview, with the fourth store at this+88h; (2) the table it stores has the 26.63 shape: slots 0 and 10 hold the same kind of non-start thunks, slots 8 and 11 the shared empty stub, and slot 7 holds the onLevelExit body that both recipes of ClientExitLevel.hook (drafted from 26.63) find on the Preview; the ClientInstance destructor loads the same table, as on 26.63. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? C7 ?? ?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ??",
        "?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ??",
      ]
      offset = 59
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

entry "ClientInstance.onTick" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onTick@ClientInstance@@QEAAXHH@Z"
  notes     = "void ClientInstance::onTick(int nTick, int maxTick), the GameCallbacks::onTick override: slot 4 of the ClientInstance.GameCallbacks.vtable global (destructor, onLevelCorrupt, onGameModeChanged, onBeforeSimTick, onTick; slot 7 is onLevelExit, ClientExitLevel.hook). `this` in rcx is the GameCallbacks subobject (ClientInstance + ClientInstance.gameCallbacksOffset, 88h), as for ClientExitLevel.hook; edx = nTick, r8d = maxTick. It runs on the main thread once per client tick (Minecraft::mRealTimer, 20 per second, more than once in a frame when the game catches up), in menus too. The first call is getLocalPlayer on this-88h; the in-world work is skipped when it is null. No direct caller: it is only reached through the vtable. Recipe 0 is the prologue up to that call and its null test. Recipe 1 shares no bytes with it: the end (`mov rcx,[rsi+member]; call; movaps xmm6,[rbp-..]; add rsp; pop rbx/rdi/rsi/r14/r15/rbp; ret`) followed by the first bytes of the cold path (`mov dword ptr [0],0DEADC0DEh`), then function_start_of. Every displacement, immediate and branch byte is a wildcard. The check in_vtable keeps the value to slots 2 to 6 of the GameCallbacks vtable (a window around slot 4; the neighbours onBeforeSimTick and onInternetUpdate are other functions). Values: 0x65a0c90 on 6aea437e-7eff13daabb430da (26.62), 0x65a09f0 on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" {
      bytes = "55 41 57 41 56 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE 44 89 45 ?? 89 55 ?? 48 8D B9 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ??"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 8E ?? ?? ?? ?? E8 ?? ?? ?? ?? 0F 28 75 ?? 48 83 C4 ?? 5B 5F 5E 41 5E 41 5F 5D C3 C7 04 25"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ClientInstance.GameCallbacks.vtable"
    min_slot = 2
    max_slot = 6
  }
}

entry "ClientInstance.gameCallbacksOffset" {
  kind  = "field"
  notes = "Offset of the GameCallbacks subobject in ClientInstance: 88h in 26.62 and 26.63. It is the displacement of the store that puts the ClientInstance.GameCallbacks.vtable pointer into the object: the `this` that a GameCallbacks override such as ClientInstance.onTick or ClientExitLevel.hook receives in rcx is ClientInstance + this value, and the ClientInstance is rcx minus it. Recipe 0 is the constructor run of ClientInstance.GameCallbacks.vtable (see that entry): the fourth `lea rax,[rip+..]; mov [reg+disp32],rax` store, read as the displacement of operand 0 of the `mov` (offset 66 from the pattern start, 7 bytes after the `lea` that the vtable entry reads). Every REX, ModRM, displacement and immediate byte of the pattern is a wildcard. A second recipe is not possible: the other uses of the offset are `lea reg,[rcx-88h]` in the overrides, a negative displacement that no step negates. Value 0x88 on 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" {
      bytes  = "?? C7 ?? ?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ??"
      offset = 66
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 512
  }
}

entry "ClientInstance.onStartJoinGame" {
  kind      = "vslot"
  msvc_name = "?$onStartJoinGame@ClientInstance@@QEAAX_NAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@11W4NetworkType@@W4MultiplayerServiceIdentifier@Social@@UPlayerJoinWorldContext@@@Z"
  notes     = "Slot of void IClientInstance::onStartJoinGame(bool isLocalServer, std::string const&, std::string const&, std::string const& worldName, NetworkType, MultiplayerServiceIdentifier, PlayerJoinWorldContext) in the ClientInstance.vtable global: 12 in 26.51, 11 in 26.60 (one IClientInstance virtual before slot 13 went away, as for ClientInstance.update). The header order gives 13 and 12, one more than the exe. Arguments: rcx = ClientInstance, dl = isLocalServer (`mov bl,dl`), r8 and r9 = the two string references, then stack arguments at entry+28h (worldName), +30h (NetworkType), +38h (the service identifier) and +40h (a pointer to the by-value PlayerJoinWorldContext). Recipe 0 is the head after the frame set-up: `mov r14,r9; mov rdi,r8; mov bl,dl; mov rsi,rcx; mov byte ptr [rcx+member],imm; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]` (the second member is the IMinecraftGame*, 1A8h in 26.51 and 1C8h in 26.60), then `function_start_of` and `slot_of` ClientInstance.vtable. Recipe 1 is a structural route: the slot of ClientInstance.getGameConnectionInfo plus 1 (that getter is the slot before this one in both builds). The two recipes share no anchor bytes. Every immediate, displacement and branch byte of the pattern is a wildcard. Both recipes agree on both builds. The range check is a window around the two values. Values: slot 12 in 26.51 (0x5da2130), 11 in 26.60 (0x6591b70). What changed in 26.60: the slot moved down by one, the members moved and the frame is 20h bigger; the argument layout did not change."

  recipe {
    step "pattern" {
      bytes = "4D 89 CE 4C 89 C7 88 D3 48 89 CE C6 81 ?? ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "ref" { entry = "ClientInstance.getGameConnectionInfo" }
    step "add" { value = 1 }
  }

  check "range" {
    min = 7
    max = 16
  }
}

entry "ClientInstance.startExternalNetworkWorld" {
  kind      = "vslot"
  msvc_name = "?$startExternalNetworkWorld@ClientInstance@@QEAAXVGameConnectionInfo@Social@@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@UPlayerJoinWorldContext@@@Z"
  notes     = "Slot of void ClientInstance::startExternalNetworkWorld(Social::GameConnectionInfo, std::string const& serverName, PlayerJoinWorldContext) in the ClientInstance.vtable global: 332 in 26.51, 328 in 26.60 (the header forecast is one more). The protected _startExternalNetworkWorld is folded into this virtual. Arguments: rcx = ClientInstance, rdx = a pointer to the by-value GameConnectionInfo, r8 = serverName, r9 = a pointer to the by-value PlayerJoinWorldContext; it returns void. The callee destroys both by-value arguments (GameConnectionInfo.dtor on the saved rdx, and the inlined PlayerJoinWorldContext destructor on the saved r9), so a caller passes pointers to objects that it built and does not destroy them after the call. Evidence: the std RTTI of the `_Func_impl` of its lambda (`<lambda_44>` in 26.51, `<lambda_46>` in 26.60) leads to a vtable that only this function loads. Recipe 0 is the head after the frame set-up: `mov [rbp+..],r9; mov rbx,r8; mov rdi,rcx; mov [rbp+..],rdx; cmp qword ptr [rdx+member],0; je; mov rax,[rbp+..]; cmp qword ptr [rax+member],0; jne; mov rax,[rbp+..]; cmp qword ptr [rax+..]` (the GameConnectionInfo fields 120h, 0E0h and 188h; 168h in 26.60), then `function_start_of` and `slot_of` ClientInstance.vtable. Recipe 1 is a call site in the slot before it (331 in 26.51, 327 in 26.60), which builds a GameConnectionInfo and calls this slot on `this`: `mov rax,[rsi]; mov rax,[rax+disp]; lea r8,[rip+..]; lea rdx,[rbp+..]; lea r9,[rbp+..]; mov rcx,rsi; call [guard]`; it reads the displacement at offset 3 and turns it into a slot. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. The range check is a window around the two values. Values: slot 332 in 26.51 (0x5daf490), 328 in 26.60 (0x659ee60). What changed in 26.60: the slot moved down by four, the frame is smaller and the GameConnectionInfo layout after +8Ch shrank by 20h."

  recipe {
    step "pattern" {
      bytes = "4C 89 8D ?? ?? ?? ?? 4C 89 C3 48 89 CF 48 89 95 ?? ?? ?? ?? 48 83 BA ?? ?? ?? ?? ?? 74 ?? 48 8B 85 ?? ?? ?? ?? 48 83 B8 ?? ?? ?? ?? ?? 75 ?? 48 8B 85 ?? ?? ?? ?? 48 83 B8"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 06 48 8B 80 ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 8D ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 320
    max = 340
  }
}

entry "ClientInstance.requestLeaveGame" {
  kind      = "vslot"
  msvc_name = "?$requestLeaveGame@ClientInstance@@QEAAX_N0@Z"
  notes     = "Slot of void IClientInstance::requestLeaveGame(bool switchScreen, bool sync) in the ClientInstance.vtable global: 15 in 26.51 and 14 in 26.60 (one IClientInstance virtual before slot 13 went away, as for ClientInstance.update; update is 9 slots after it in both builds). Arguments: rcx = ClientInstance, dl = switchScreen, r8b = sync; it returns void. Evidence: the function loads the string 'virtual void __cdecl ClientInstance::requestLeaveGame(bool, bool)' (`inspect xrefs --string` gives only this function: 0x5da3d10 in 26.51, 0x6593960 in 26.60), and both addresses are what the ClientInstance.vtable holds at slot 15 and slot 14. Recipe 0 is the first virtual call after the frame set-up: `test bl,bl; mov [rbp+..],reg; jne rel32; mov rax,[r14]; mov rax,[rax+disp]; mov rcx,r14; call [guard]` (a bool-returning virtual of the same object; displacement 510h in 26.51, 500h in 26.60). Recipe 1 shares no bytes with recipe 0: it is the packing of the two bools further down (`movzx ecx,reg8; movzx eax,bl; shl eax,imm; or eax,ecx; mov byte ptr [r14+member],imm; mov reg,[r14+member]`). Both end with `function_start_of` and `slot_of` ClientInstance.vtable. Every immediate, displacement and branch byte of the patterns is a wildcard; only the register bytes are kept as they are. Both recipes are unique in both exes and agree. The range check is a window around the two values. Values: slot 15 in 26.51 (0x5da3d10), 14 in 26.60 (0x6593960). What changed in 26.60: the slot moved down by one; the frame is 10h bigger; a null check of a member at 178h and a `nop` after the first call were added; the body has the same arguments."

  recipe {
    step "pattern" {
      bytes = "84 DB ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 49 8B 06 48 8B 80 ?? ?? ?? ?? 4C 89 F1 FF 15"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "pattern" {
      bytes = "40 0F B6 ?? 0F B6 C3 C1 E0 ?? 09 C8 41 C6 86 ?? ?? ?? ?? ?? 49 8B ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  check "range" {
    min = 10
    max = 20
  }
}

entry "ClientInstance.getMinecraftInput" {
  kind      = "vslot"
  msvc_name = "?$getMinecraftInput@ClientInstance@@QEBA?AV?$NonOwnerPointer@VMinecraftInputHandler@@@Bedrock@@XZ"
  notes     = "Slot of Bedrock::NonOwnerPointer<MinecraftInputHandler> IClientInstance::getMinecraftInput() const in the ClientInstance.vtable global: 267 (displacement 858h) in 26.51 and 264 (840h) in 26.60 (the header order gives one more than the exe in 26.60, as for the other slots after slot 13). Arguments: rcx = ClientInstance, rdx = a hidden pointer to a 24-byte buffer; it returns that pointer in rax. The buffer holds the NonOwnerPointer: the control block at +0, the reference at +8 and the MinecraftInputHandler pointer at +10h; the caller releases the shared pointer at +0/+8. Evidence: the body (0x5dce200 in 26.51, 0x65bad00 in 26.60) forwards to slot 134 (430h) of the MinecraftGame held at [rcx+1A8h] (1C8h in 26.60) with the same rdx; it is a frame and a call in 26.51 and a frameless tail jump in 26.60. Recipe 0 is a call site: a function that holds the client at [this+member] calls the slot into a stack buffer (`mov qword ptr [rbp+..],imm; mov [rbp+..],r8; mov rsi,rdx; mov rbx,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]`); it reads the displacement at offset 25 and turns it into a slot. Recipe 1 shares no bytes with recipe 0: the body of ClientInstance.setLastPointerLocation (three `movss [rcx+member],xmmN` stores and `ret`, a leaf that is two slots after this one in both builds: 269 and 266), `slot_of` ClientInstance.vtable, minus 2. Every immediate, displacement and branch byte of the patterns is a wildcard. Both recipes are unique in both exes and agree. The range check is a window around the two values. Values: slot 267 in 26.51 (0x5dce200), 264 in 26.60 (0x65bad00). What changed in 26.60: the slot moved down by three; the body lost its frame; the MinecraftGame member moved."

  recipe {
    step "pattern" {
      bytes  = "48 C7 45 ?? ?? ?? ?? ?? 4C 89 45 ?? 48 89 D6 48 89 CB 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ??"
      offset = 25
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes = "F3 0F 11 89 ?? ?? ?? ?? F3 0F 11 91 ?? ?? ?? ?? F3 0F 11 99 ?? ?? ?? ?? C3"
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
    step "add" { value = -2 }
  }

  check "range" {
    min = 260
    max = 272
  }
}

entry "ClientInstance.getTextureGroup" {
  kind     = "vslot"
  notes    = "Slot of `std::shared_ptr<mce::TextureGroup> IClientInstance::getTextureGroup() const` in the IClientInstance vtable: 103 on 26.60, 105 on 26.51. The ClientInstance override calls MinecraftGame slot 10 (`[rax+50h]`) on the IMinecraftGame member and returns its result through the hidden out pointer in rdx (16 bytes: element, control block). On 26.60 it is a leaf thunk without .pdata (0x65ac2c0: `mov rcx,[rcx+1C8h]; mov rax,[rcx]; mov rax,[rax+50h]; mov r8,[guard]; jmp r8`); on 26.51 it has a frame (0x5dbe030: `push rsi; sub rsp,..; mov rsi,rdx; mov rcx,[rcx+1A8h]; mov rax,[rcx]; mov rax,[rax+50h]; call [guard]`). Recipe 0 is the call in HudLocatorBarRenderer.render (`movaps [rbp+..],xmm0; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; lea rdx,[rbp+..]; call [guard]`); the displacement is turned into a slot. render then adds 18h to the group before it calls TextureGroup.getTexture. Recipe 1 is the override itself, from its first byte, then slot_of the ClientInstance.vtable: a pattern_any with one alternative per build (alternative 0 the 26.60 thunk, alternative 1 the 26.51 function); each matches once on its own build and never on the other. The sibling overrides (slots 102 and 104 on 26.60) have the same shape and forward other MinecraftGame slots, and a thunk of another class forwards slot 10 from another member (+0A0h), so the member offset (2 bytes) and the slot displacement (1 byte) are kept in each alternative. The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. The range check is a window around the slots of both builds."

  recipe {
    step "pattern" {
      bytes  = "0F 29 85 ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 48 8D 95 ?? ?? ?? ?? FF 15"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 89 C8 01 ?? ?? 48 8B 01 48 8B 40 50 4C 8B 05 ?? ?? ?? ?? 49 FF E0",
        "56 48 83 EC ?? 48 89 D6 48 8B 89 A8 01 ?? ?? 48 8B 01 48 8B 40 50 FF 15",
      ]
      offset     = 0
      literal_ok = [
        "0.3: the IMinecraftGame member 1C8h of ClientInstance (26.60): a thunk of another class forwards the same slot from +0A0h",
        "0.4: the IMinecraftGame member 1C8h of ClientInstance (26.60): a thunk of another class forwards the same slot from +0A0h",
        "0.13: MinecraftGame slot 10 (50h): the sibling ClientInstance thunks forward other slots of the same member",
        "1.11: the IMinecraftGame member 1A8h of ClientInstance (26.51): keeps the pattern on ClientInstance overrides",
        "1.12: the IMinecraftGame member 1A8h of ClientInstance (26.51): keeps the pattern on ClientInstance overrides",
        "1.21: MinecraftGame slot 10 (50h): the sibling ClientInstance overrides call other slots of the same member",
      ]
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  check "range" {
    min = 95
    max = 113
  }
}

entry "ClientInstance.getMinecraftGame" {
  kind     = "vslot"
  notes    = "Slot of IMinecraftGame& IClientInstance::getMinecraftGame_DEPRECATED() in the ClientInstance vtable: 78 on 26.60 (offset 270h). rcx the client instance; the IMinecraftGame* in rax. The game passes it as the third argument of the BaseActorRenderContext constructor everywhere it draws an inventory item. Recipe 0 is that call in the helper that draws a flying item (`mov rax,[r14]; mov rax,[rax+..]; mov rcx,r14; call [guard]; mov rdx,[rsi+..]; lea r15,[rbp+..]; mov rcx,r15; mov r8,r14; mov r9,rax; call`), read as the displacement of the load and turned into a slot. Recipe 1 is the same call in the inventory item renderer (`mov rax,[r8]; mov rax,[rax+..]; mov rcx,r8; mov rbx,r8; call [guard]; lea r15,[rbp+..]`). The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window like the neighbouring ClientInstance slots. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 78 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "49 8B 06 48 8B 80 ?? ?? ?? ?? 4C 89 F1 FF 15 ?? ?? ?? ?? 48 8B 56 ?? 4C 8D 7D ?? 4C 89 F9 4D 89 F0 49 89 C1 E8"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 C1 4C 89 C3 FF 15 ?? ?? ?? ?? 4C 8D 7D ?? 4C 89 F9"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 50
    max = 140
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ClientInstance.mMinecraft" {
  kind  = "field"
  notes = "Offset of `mMinecraft` in ClientInstance: 1D0h in 26.60. The member is a std::unique_ptr<Minecraft> (8 bytes): the client Minecraft object, which the old LeviLamina API returned from ClientInstance::getServerData(). 26.60 has no ClientInstance virtual that only returns it: the one `mov rax,[rcx+1D0h]; ret` body in the exe (0xec69890) is in five other vtables and not in ClientInstance.vtable, so a mod reads the field. ClientInstance::init (slot 3, 0x658be40) calls a std::function factory with the pair {&this+E58h, &this+ED8h} (the two client timers), moves the returned pointer into [this+1D0h] and deletes the old one through its virtual destructor (`mov rax,[rcx]; mov rax,[rax]; mov edx,1; call`). ClientInstance::update (0x65977b0) passes [this+1D0h] to Minecraft::update (0xb82190) right after it advances the two timers; the server update lambda (0x1048dc0) passes [ServerInstance+B8h] to the same function. The ClientInstance slot-26 leaf (0x659eba0) tests [this+1D0h] for null before it reads [[this+1D0h]+E8h] (Minecraft.mSimTimer), so the pointer can be null. Recipe 0 is the store in ClientInstance::init, after the factory call: `mov rdi,[rbp+disp]; mov qword ptr [rbp+disp],imm; mov rcx,[rsi+member]; mov [rsi+member],rdi; test rcx,rcx; je rel8; mov rax,[rcx]; mov rax,[rax]; mov edx,imm; call [guard]`; it reads the displacement of operand 1 of the third instruction (offset 18). Recipe 1 is in ClientInstance::update (ref ClientInstance.update): the end of the real-timer advance and the Minecraft::update call, `xor r8d,r8d; call; nop; mov rcx,[rsi+member]; call`; it reads the displacement of operand 1 of the `mov` (offset 9). It runs past one call, because 26.60 puts a `nop` there; a build without the nop breaks it, and it fails then rather than give a wrong value. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range 100h..400h. The gate LevelChunk.mSubChunks = 456 ties it to 26.60 (the 26.51 exe was not available; widen it when a later build is added). Value 1D0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8B BD ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 89 BE ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 00 BA ?? ?? ?? ?? FF 15"
      offset = 18
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ClientInstance.update" }
    step "pattern" {
      bytes  = "45 31 C0 E8 ?? ?? ?? ?? 90 48 8B 8E ?? ?? ?? ?? E8"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 1024
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ClientInstance.mSimTimer" {
  kind  = "field"
  notes = "Offset of the client sim Timer in ClientInstance: E58h in 26.60. The member is an inline Timer (80h bytes), not a pointer: the code passes `lea rcx,[this+E58h]` as the Timer's `this`. The name is inferred: Minecraft.mSimTimer (a Timer&) of the client Minecraft points at it. Evidence: ClientInstance::update (0x65977b0) advances [this+E58h] and then [this+ED8h] with Timer::advanceTime(rcx, 1.0f, false) when its third argument (r14b) is set, then calls Minecraft::update with ClientInstance.mMinecraft; when the argument is clear it only recomputes mAlpha (+8) of both from mLastMs (+30h), mTimeScale (+0Ch), mTicksPerSecond (+0) and mPassedTime (+10h), and checks the std::function at +78h of each Timer (E58h+78h = ED0h, ED8h+78h = F50h). ClientInstance::init (slot 3, 0x658be40) passes {&this+E58h, &this+ED8h} to the Minecraft factory, whose lambda (0x8d3120) puts them in the Minecraft construction arguments at +40h/+48h; the Minecraft constructor (0xb80b30) stores them at Minecraft+E8h/+F0h. So [ClientInstance.mMinecraft + E8h] == this+E58h. Recipe 0 is that pair in ClientInstance::init: `lea rax,[rsi+sim]; mov [rbp+disp],rax; lea rax,[rsi+real]; mov [rbp+disp],rax; mov rax,[r12+disp]; mov rdx,[r12+disp]; lea rcx,[rbp+disp]; lea r8,[rbp+disp]; call [guard]`; it reads the displacement of operand 1 of the first `lea`. Recipe 1 is the first advance in ClientInstance::update (ref ClientInstance.update): `test r14b,r14b; je rel32; lea rcx,[rsi+sim]; movss xmm1,[rip+..]; xor r8d,r8d; call`; it reads the displacement of operand 1 of the `lea` (offset 9). The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range C00h..1400h. The gate LevelChunk.mSubChunks = 456 ties it to 26.60 (the 26.51 exe was not available; widen it when a later build is added). Value E58h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes = "48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 49 8B 44 24 ?? 49 8B 54 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? FF 15"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ClientInstance.update" }
    step "pattern" {
      bytes  = "45 84 F6 0F 84 ?? ?? ?? ?? 48 8D 8E ?? ?? ?? ?? F3 0F 10 0D ?? ?? ?? ?? 45 31 C0 E8"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 3072
    max = 5120
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ClientInstance.mRealTimer" {
  kind  = "field"
  notes = "Offset of the client real-time Timer in ClientInstance: ED8h in 26.60, 80h (one Timer) after ClientInstance.mSimTimer. The member is an inline Timer (80h bytes), not a pointer. The name is inferred: Minecraft.mRealTimer (a Timer&) of the client Minecraft points at it. Evidence: ClientInstance::update (0x65977b0) advances it second, right after [this+E58h], with Timer::advanceTime(rcx, 1.0f, false); ClientInstance::init (slot 3, 0x658be40) passes it as the second element of the timer pair to the Minecraft factory, and the Minecraft constructor (0xb80b30) stores it at Minecraft+F0h. Recipe 0 is the pair in ClientInstance::init (see ClientInstance.mSimTimer recipe 0); it reads the displacement of operand 1 of the second `lea` (offset 14). Recipe 1 is the second advance in ClientInstance::update (ref ClientInstance.update): `xor r8d,r8d; call; nop; lea rcx,[rsi+real]; movss xmm1,[rip+..]; xor r8d,r8d; call`; it reads the displacement of operand 1 of the `lea` (offset 9). It runs past one call, because 26.60 puts a `nop` there; a build without the nop breaks it, and it fails then rather than give a wrong value. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range C00h..1400h. The gate LevelChunk.mSubChunks = 456 ties it to 26.60 (the 26.51 exe was not available; widen it when a later build is added). Value ED8h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 86 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 49 8B 44 24 ?? 49 8B 54 24 ?? 48 8D 8D ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? FF 15"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ClientInstance.update" }
    step "pattern" {
      bytes  = "45 31 C0 E8 ?? ?? ?? ?? 90 48 8D 8E ?? ?? ?? ?? F3 0F 10 0D ?? ?? ?? ?? 45 31 C0 E8"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 3072
    max = 5120
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ClientInstance.getCurrentUIRouter" {
  kind     = "vslot"
  notes    = "Slot of OreUI::Router& IClientInstance::getCurrentUIRouter() in the ClientInstance.vtable global: 239 (displacement 778h) on 26.60, 26.62 and 26.63. The header (LeviLamina IClientInstance.h) declares the non-const and the const overload one after the other; the exe folds both into one function, so slots 238 and 239 hold the same body (26.63 0x65b4fa0, 26.62 0x65b5240, 26.60 0x65b4640); the game calls 239. rcx = the ClientInstance; the Router pointer is returned in rax (a reference; the game does not test it for null at the call sites read). The body returns the member router at +5F0h when the client shows no world or an overlay flag is set, else tail-calls MinecraftGame slot 123 (3D8h) on the IMinecraftGame at +1C8h; the mod must call the slot, not read the member, to get the router the game uses in every state. Evidence that the result is the OreUI::Router: in both call sites below the returned rax goes as rcx straight into OreUIRouter.pushRoute (0x4d428c0 on 26.63: `mov rcx,rax; xor r8d,r8d; call 0xf3c360`, the route `/play`) or, through rdi, into OreUIRouter.replaceRoute (0x28c0e90: the Play tab helper). Recipe 0 is the call site in the Play tab helper (0x28c0e90 on 26.63, 0x28c1130 on 26.62, 0x28c11d0 on 26.60): `mov rcx,[rax+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; mov rdi,rax; lea r9,[rip+..]; lea rbx,[rbp+..]; mov rcx,rbx; mov edx,esi; xor r8d,r8d; call` (the call builds the `/play/<tab>` route). Recipe 1 is the call site in the function that pushes `/play` (0x4d428c0 on 26.63, 0x4d42b60 on 26.62, 0x4d42990 on 26.60): `mov rcx,[rsi+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`, the inline std::string set-up (size, capacity, four zero stores, the 4-byte and 1-byte rip loads of `/play`), `lea rdx,[rbp+..]; mov rcx,rax; xor r8d,r8d; call` (the call is OreUIRouter.pushRoute). Both read the displacement of the vtable load at offset 7 and turn it into a slot. They are in two functions and share no anchor bytes past the common vtable load. Every displacement, immediate and branch byte is a wildcard. Both are unique on 26.60, 26.62 and 26.63 and agree. The range check is a window around the slot. On 26.51 recipe 0 has no match and recipe 1 alone gives slot 242 (790h, also a folded pair 241/242); a range check alone needs two recipes, so the entry refuses on 26.51 (out of scope). Values: 239 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 48 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 89 C7 4C 8D 0D ?? ?? ?? ?? 48 8D 5D ?? 48 89 D9 89 F2 45 31 C0 E8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? C7 45 ?? ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 89 4D ?? 0F B6 0D ?? ?? ?? ?? 88 4D ?? 48 8D 55 ?? 48 89 C1 45 31 C0 E8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 232
    max = 248
  }
}
