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
  notes    = "Slot of `Bedrock::NonOwnerPointer<CameraRegistry> ClientInstance::getCameraRegistry()` in the ClientInstance vtable: 190 in 26.51. It returns the 24-byte NonOwnerPointer (the shared_ptr to the control block, then the pointer) through a hidden buffer in rdx. LeviLamina header order: getLevelRendererCameraProxy, getCameraRegistry, getCameraRegistry const, getCameraSystems; MSVC puts the overloads in reverse order, so the const overload is 189 and this one 190; getCameraSystems is 191. The three bodies forward to the game object (slot 168 or 169 of ClientInstance, then 0C20h, 0C18h, 0C28h of that object). One recipe: the CameraPresetsPacket handler (0x131bc60) calls it through the ClientInstance at ClientNetworkHandler+58h (`mov rcx,[rbx+58h]; mov rax,[rcx]; mov rax,[rax+5F0h]; lea rdx,[rbp-18h]; call`); the recipe finds the handler by its unique hand-over to the preset code (`add rbx,158h; mov [rsp+20h],rbx; ...; call`) and then the call in the same function, and reads the displacement as a slot. Other callers of slot 190 use the same call shape as many other slots. Grade: the header order with the overload rule, and the CameraInstructionPacket handler calling getCameraSystems at 191 (5F8h). Every displacement, immediate and branch byte is a wildcard. Recipe 1 is the CameraInstructionPacket handler, which calls getCameraSystems through the same ClientInstance (`mov rdi,[rcx+0C0h]; test rdi,rdi; je; mov rcx,[rcx+58h]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp-..]; call`): it reads that slot and subtracts 1, because the header declares getCameraSystems right after the getCameraRegistry overloads (MSVC order: const, non-const, getCameraSystems). A virtual added between them makes the two recipes disagree, and the entry refuses. What changed in 6ad79ee4-17d51891a1119914: the slot is 188 (two virtuals before it went away; getCameraSystems is 189); the forwarder that v2 first used as recipe 1 has no unique counterpart on 26.60 (the unique match there forwards slot 190, another function). Evidence for 188 on 26.60, from the 26.51 handlers: (1) the CameraPresetsPacket handler calls slot 188 at the place where 26.51 calls 190; (2) the CameraInstructionPacket handler calls slot 189 at the place where 26.51 calls getCameraSystems at 191. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 164 (0A4h), 24 lower than on 26.63 (the ClientInstance vtable lost virtuals before it). Both recipes still match, in two functions (the CameraPresetsPacket handler and the CameraInstructionPacket handler), and agree on 164 (recipe 1 reads getCameraSystems at 165 and subtracts 1); only the range window was too narrow, and it is now 160 to 210. Evidence on the Preview, both from 26.63: (1) the two recipes, drafted from two 26.63 functions, each found once on the Preview, read the same slot, 164, from different call sites; (2) the move from 188 to 164 is the 24 slots by which the resolved ClientInstance.getOptions moved (172 to 148) a few slots before this one, and the header order getCameraRegistry then getCameraSystems holds (164, 165). Would have survived with a wider window."

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
    min = 160
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
  notes    = "Slot of FontHandle ClientInstance::getFontHandle() const in the ClientInstance vtable: 89 in 26.51 (header order; getRegion is at 30 by the same count, which matches the existing ClientInstance.getRegion entry). The FontHandle (0x50 bytes, a class with a destructor) is returned through a hidden pointer: rcx = this, rdx = the result buffer. The call sites show it: `mov rcx,[rsi+98h]; mov rax,[rcx]; mov rax,[rax+2C8h]; lea rbx,[rbp-58h]; mov rdx,rbx; call [guard]; mov rcx,rbx; call FontHandle::getFont`. Recipe 0 is that call site in a function that measures a text width (0x15632c0). Recipe 1 is the call site in extractNameTags (`mov rcx,[rsi+1168h]; mov rax,[rcx]; mov rax,[rax+2C8h]; lea rdx,[rbp-30h]; call [guard]; mov [rbp+..],rsi; lea rcx,[rbp-30h]; call getFont`). Both read the displacement of the vtable call and turn it into a slot. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60): the slot is 87 (0x2B8), two lower than on 26.51: two ClientInstance virtuals before it were removed. The `FontHandle::getFont` call after the virtual call is inlined there (the mod does its reads through the FontHandle mirror), so both old patterns, which ran on into `mov rcx,rbx; call getFont`, matched nothing. Both recipes are now a pattern_any with one alternative per build; the first starts at the `mov byte ptr [r14+rbx],0` that ends the string copy before the call, so one offset serves both. Class: real code change (inlined callee, moved slot). Cold run result: failed (both recipes no match). What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 66 (42h), 21 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; getTopScreenName went from 134 to 113). The window 50 to 140 already holds it. Recipe 0 still matched, in the text measure function. Recipe 1 (extractNameTags) found nothing: after the call the code no longer tests the string pointer inline; it stores the client (`mov [rbp+..],rdi`) and calls getFont, the shape the 26.51 pattern had, with a nop after the call and other registers. Recipe 1 has a third alternative for it: `mov rcx,[reg+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]; nop; mov [rbp+..],reg; lea rcx,[rbp+..]; call`, found once on the Preview and not on 26.63. Evidence on the Preview, both from 26.63: (1) the two recipes, in two functions drafted from 26.63 (the text measure function and extractNameTags), read the same slot, 66; (2) the 26.63 extractNameTags and the Preview function are the best match of the 26.63 body by the function similarity list (first candidate) and both call this slot through the client member at the same place in the function; the move from 87 to 66 is the 21 slots of getTopScreenName. Would have survived with a better recipe."

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
        "48 8B ?? ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ?? 90 48 89 ?? ?? ?? ?? ?? 48 8D 4D ?? E8",
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
  notes    = "Slot of std::optional<Social::GameConnectionInfo> IClientInstance::getGameConnectionInfo() in the IClientInstance vtable: 11 in 26.51 (LeviLamina header order: the slot after setGameConnectionInfo, 10). The optional is returned through a hidden out pointer in rdx. The slot 11 body clears the engaged flag of the out value (`mov byte ptr [rdx+310h],0`), copies the member optional at this+6D8h with the GameConnectionInfo copy constructor when its flag (this+9E8h) is set, and sets the out flag. Three other classes have the same body with other member offsets, so no body anchor is unique. Both recipes are call sites in one screen-controller function whose member at +58h is the client instance (the same function calls slot 79 of that member, getMinecraftGame_DEPRECATED in header order, before the first call). Recipe 0 is the second call: it reads the connection type (`movzx edi,word ptr [rbp+X]`) and tests the engaged flag at X+310h right after the call. Recipe 1 is the first call, after a `test al,al; je` and followed by `cmp byte ptr [rbp+X+310h],0; je`. Both read the displacement and turn it into a slot. The two call sites are in the same function, so they are not fully independent: a rewrite of that function breaks both. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Slot 11 (0Bh) on 26.51, 10 (0Ah) on 26.60: ClientInstance lost a virtual before it (the other ClientInstance slots moved down too). Recipe 0 is now a pattern_any with the nop alternative; recipe 1 now ends at the `call [guard]`. Both are in the same server-identity function (Ruling R17), so recipe 2 was added as an independent route: the getGameConnectionInfo body (`mov rax,rdx; mov byte ptr [rdx+size],0; cmp byte ptr [rcx+9E8h],1; jne; mov rdx,rcx; add rdx,..; ...; call`), then function_start_of and slot_of ClientInstance.vtable. Three other optional getters have the same body with other members, so the two bytes of the member offset 9E8h (the engaged flag of the ClientInstance optional GameConnectionInfo, the same on both builds) are kept with literal_ok. Evidence on 26.60: the call sites, drafted from the old function, read 50h; the body of the old slot 11 (drafted from the old body with 9E8h kept) matches once on 26.60 and sits in slot 10. Class: real code change (slot moved). What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 2, eight lower than on 26.63 (the ClientInstance vtable lost virtuals before it; onStartJoinGame went from 11 to 3 and requestLeaveGame from 14 to 6). Recipes 0 and 1 still matched and agreed on slot 2; only the range window was too narrow, and it is now 0 to 19. The engaged flag of the optional moved from 9E8h to 0A10h, so recipe 2 is now a pattern_any with one alternative per offset (the offset is the only thing that tells the body from the three other optional getters; each alternative matches once on its own builds and not on the other). Evidence on the Preview, both from 26.63: (1) recipes 0 and 1, call sites in two places drafted from the 26.63 function, read slot 2 and agree; the body of 26.63 slot 10 (recipe 2, with the flag offset kept) is found once on the Preview, in the function of slot 2; (2) that function sits 80h before the onStartJoinGame slot function, as the 26.63 slot 10 function sits 80h before its slot 11. Would have survived with a wider window and a pattern_any."

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
    step "pattern_any" {
      patterns   = ["48 89 D0 C6 82 ?? ?? ?? ?? ?? 80 B9 E8 09 ?? ?? ?? 75", "48 89 D0 C6 82 ?? ?? ?? ?? ?? 80 B9 10 0A ?? ?? ?? 75"]
      literal_ok = [
        "0.12: cmp byte ptr [rcx+9E8h],1, the engaged flag of ClientInstance's optional GameConnectionInfo: three other optional getters have the same body with other members (150h, 1D90h, 0CE8h) (26.51 and 26.60)",
        "0.13: the high byte of the same member offset 9E8h (26.51 and 26.60)",
        "1.12: cmp byte ptr [rcx+0A10h],1, the engaged flag of ClientInstance's optional GameConnectionInfo on the Preview: the same three other optional getters have the same body, so the member offset tells them apart (alternative 1, Preview 1.26.70.26)",
        "1.13: the high byte of the same member offset 0A10h (alternative 1, Preview 1.26.70.26)",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  check "range" {
    min = 0
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
  notes    = "Slot of ClientLevel* ClientInstance::getLevel() in the ClientInstance vtable: 168 on 26.51 (168 and 169 hold one body: the non-const and const overloads). Found by header order: in the 26.51 LeviLamina header the known slots getRegion 30, isShowingMenu 150, isWorldActive 159 and getOptions 174 are at their header positions, and getLevel is header 168/169. The body is a leaf (no .pdata): `mov rax,[rcx+1B0h]; mov rax,[rax+0C8h]; test; je; cmp byte ptr [rax+28h],1; jne; mov rcx,[rax+30h]; test; je; cmp byte ptr [rcx],1; jne; mov rax,[rax+40h]; ret`. Recipe 0 is that body at its first byte, then slot_of in ClientInstance.vtable (the first slot that holds it). Recipe 1 is the neighbour route: ClientInstance.isWorldActive (159, its own recipes) plus 9, the header distance (isInRealm 160, then 7 virtuals, then getLevel). Every displacement and branch byte is a wildcard. Range window 160..176. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 142 (8Eh), 24 lower than on 26.63, and 8 slots after isWorldActive, not 9: a virtual between isInRealm and getLevel went away. Recipe 1 added 9 to isWorldActive, a fixed header distance that is wrong on the Preview (it gave 8Fh against 8Eh from the body in recipe 0, so the entry refused). Recipe 1 is now a call site: a small function that calls getLevel on this and then a virtual on the result (`push rsi; sub rsp,..; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; mov rsi,rax; test rax,rax; je; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`), read as the displacement of the first call and turned into a slot (166 on 26.63, 142 on the Preview). It shares no bytes with recipe 0. The window is now 138 to 176. Evidence on the Preview, both from 26.63: (1) the body of the 26.63 getLevel (recipe 0, drafted from that leaf) is found once on the Preview and is held by slots 142 and 143; (2) the call-site function drafted from 26.63 is found once on the Preview and calls slot 142 in the same place; ClientInstance.isWorldActive, which on both builds is getLevel() != nullptr, calls the same slot (displacement 530h on 26.63, 470h on the Preview). Class: would have survived with a better recipe (the fixed distance of 9 was the cause)."

  recipe {
    step "pattern" { bytes = "48 8B 81 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 80 78 ?? ?? 75 ?? 48 8B 48 ?? 48 85 C9 74 ?? 80 39 ?? 75 ?? 48 8B 40 ?? C3" }
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "56 48 83 EC ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 89 C6 48 85 C0 74 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 138
    max = 176
  }
}

entry "ClientInstance.isInRealm" {
  kind     = "vslot"
  notes    = "Slot of bool ClientInstance::isInRealm() in the ClientInstance vtable: 160 on 26.51 (the 26.51 header position, the slot after isWorldActive 159). The body is a leaf that forwards to the IMinecraftGame at this+1A8h, slot 102 (330h): `mov rcx,[rcx+1A8h]; mov rax,[rcx]; mov rax,[rax+330h]; mov rdx,[guard]; jmp rdx`. IMinecraftGame::isInRealm is header 108; the known IMinecraftGame slots (grabMouse 135 = header 141) put it at 102, so the forward target proves the slot. Recipe 0 is that body, then slot_of in ClientInstance.vtable. The 330h displacement is kept: 50 leaf forwarders have the same shape with other slots. Recipe 1 is ClientInstance.isWorldActive plus 1. Range window 152..168. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 135 (87h), 23 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; isWorldActive went from 157 to 134). Both recipes give 135 and agree; only the range window was too narrow, and it is now 131 to 168. Evidence on the Preview, both from 26.63: (1) the body of recipe 0, drafted from the 26.63 leaf with the 330h forward kept, is found once on the Preview, in a leaf that forwards through the same IMinecraftGame slot 102, and slot_of gives 135; (2) that leaf sits 20h bytes after the isWorldActive function, as the 26.63 leaf does, and the Preview slot is the one after the resolved ClientInstance.isWorldActive (134). Would have survived with a wider window."

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
    min = 131
    max = 168
  }
}

entry "ClientInstance.getItemInHandRenderer" {
  kind     = "vslot"
  notes    = "Slot of ItemInHandRenderer* ClientInstance::getItemInHandRenderer() in the ClientInstance vtable: 353 on 26.51. The getters 349..357 line up with the header run getBlockTessellator .. getDevConsoleMessageHistory: 349 returns this+610h (the BlockTessellator that the ClientInstance init stores there), 351 copies a shared_ptr (getEntityRenderDispatcher), 352..355 are `mov rax,[rcx+disp]; ret` leaves and 356/357 return `lea` of a deque (the two message histories). 353 returns this+638h. Recipe 0 starts at getBlockTessellator and covers the run of bodies in memory: two member getters, the shared_ptr getter (getEntityRenderDispatcher, `mov rax,rdx; xorps; movups [rdx],xmm0; mov rdx,[rcx+..]; ..; ret`), then two member getters, each padded with int3; then slot_of in ClientInstance.vtable (getBlockTessellator) and add 4: this slot is the second getter after the shared_ptr getter on both builds. Recipe 1 is the render context helper that LevelRenderer.renderLevel calls first (the first direct call in it): it fills the context from ClientInstance getters, and this getter fills +50h (`mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; mov [r64+50h],rax`); one pattern_any alternative per shape (alternative 1 has the nop that 26.60 adds after the call); the store displacement 50h is the one kept byte; the displacement is read as a slot. Range window 345..361. Value 353 (161h) on 26.51, 349 (15Dh) on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): ClientInstance lost 4 virtuals before this run, and the member getter of slot 350 is folded with an identical body elsewhere, so the 26.51 run (three member getters and two lea getters from this slot) now starts one slot early and recipe 0 gave 348: a wrong value, which the weak-check rule refused. The 26.51 call site of recipe 1 (in ClientInputCallbacks.handleBuildAction) has another shape. The slot was accepted on two facts from 26.51: the vtable neighbours (the shared_ptr getter, then two member getters with the same 8-byte steps: 26.51 351, 352, 353 return +628h, +630h, +638h; 26.60 347, 348, 349 return +640h, +648h, +650h), and the render context helper that calls it right after the slot before it and stores the result at +50h on both builds. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 313 (139h), 36 lower than on 26.63 (the ClientInstance vtable lost virtuals before it). Both recipes still match and agree, so only the range window was too narrow; it is now 309 to 365. Evidence on the Preview, both from 26.63: (1) recipe 0 (the run of getter bodies in memory, getBlockTessellator plus 4) and recipe 1 (the helper that LevelRenderer.renderLevel calls first, the store of the result at +50h) are two places in two functions, drafted from 26.63, each found once on the Preview, and both read slot 313; (2) the helper is reached through the first call of LevelRenderer.renderLevel, a resolved entry on both builds, and it makes the same call in the same position before the same store. Would have survived with a wider window."

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
    min = 309
    max = 365
  }
}

entry "ClientInstance.getPacketSender" {
  kind     = "vslot"
  notes    = "Slot of PacketSender& IClientInstance::getPacketSender() in the ClientInstance vtable: 293 on 26.51 (LeviLamina header order; the neighbours line up: 288 and 289 set and get one int member, 292 getInProgressBAI returns this+290h, 294 and 295 hold one body, the getClientNetworkSystem const pair). The body is a leaf that returns a member (`mov rax,[rcx+1D0h]; ret`). The callers then call slot 2 of the result with a packet in rdx (PacketSender::send). Recipe 0 is the ClientInstance function that opens the inventory: it builds an InteractPacket on the stack (action 6, OpenInventory; the runtime id from Actor::getRuntimeID; serialization mode 5), then `mov byte ptr [rbp-..],0; mov dword ptr [rbp],5; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]` and sends it. Recipe 1 is a send in another client function: `mov byte ptr [rdi+rbx],..; mov rdi,[rbp+..]; mov rax,[rdi]; mov rax,[rax+disp]; mov rcx,rdi; call [guard]`. Both read the displacement as a slot. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 285..301. Recipe 0 is now a pattern_any that runs on from the send to the code after it, one alternative per shape: alternative 0 (26.51) `..; call [guard]` (the send) `; nop; add rsp,..`; alternative 1 (26.60) the first of two identical send blocks in the function, picked by what follows it (`call; mov rdi,rax; call; cmp rdi,imm32; je near`; the second block has a short je). Recipe 1 is now a send in another client function: `mov [rsp+..],eax; mov dword ptr [rsp+..],imm; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`, then `mov rcx,[rax]; mov r8,[rcx+10h]` (slot 2 of the result, PacketSender::send) on both builds. Value 293 on 26.51, 290 on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): ClientInstance lost 3 virtuals before this slot; the open-inventory function has the send twice, and the 26.51 site of recipe 1 changed shape. The slot was accepted on two facts from 26.51: the vtable neighbours (26.51 288..295 and 26.60 285..292 hold the same bodies: the int setter and getter, two forwarders, getInProgressBAI, this leaf, the const pair; the members moved by +18h to +20h), and the InteractPacket send of the open-inventory function (action 6, mode 5) calls it on both builds. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 264 (108h), 26 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; getCurrentUIRouter went from 239 to 213, also 26). Recipe 1 still matched and read 264. Recipe 0 found nothing: in the function that opens the inventory the packet is now built through rsp, not rbp (`mov byte ptr [rsp+..],0; mov dword ptr [rsp+..],5`), and the `je` after the compare is short. Recipe 0 has a third alternative for that shape (the same send, `mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; mov rcx,[rax]; mov r8,[rcx+10h]; lea rdx,[rsp+..]; mov rcx,rax; mov rax,r8; call [guard]; call; mov rdi,rax; call; cmp rdi,imm32; je short`), found once on the Preview and not on 26.63. The window is now 255 to 301. Evidence on the Preview, both from 26.63: (1) the two recipes, in two functions (the inventory-open function and the send in the other client function), drafted from 26.63, read the same slot, 264; (2) the Preview inventory-open function is the same function as the 26.63 one (the InteractPacket action 6 and mode 5 set-up, the send through slot 2 of the result, then the two clock calls and the compares with 16E3600h and 989680h), and the move of 26 equals that of getCurrentUIRouter. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? 90 48 81 C4",
        "C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 55 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C7 E8 ?? ?? ?? ?? 48 81 FF ?? ?? ?? ?? 0F 84",
        "?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 41 ?? 48 8D 54 24 ?? 48 89 C1 4C 89 C0 FF 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C7 E8 ?? ?? ?? ?? 48 81 FF ?? ?? ?? ?? 74",
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
    min = 255
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
  notes    = "Slot of bool IClientInstance::getMouseGrabbed() const in the IClientInstance vtable: 147 in 26.51 (LeviLamina's IClientInstance header order) and 145 in 26.60. The function returns `isPrimaryClient() && minecraftGame->getMouseGrabbed()`; the slots before it are useController and useTouchscreen, the one after it currentInputModeIsMouseAndKeyboard. Recipe 0 is in a GuiData function that tests getMouseGrabbed and then useTouchscreen on the client held at [rdi+98h]: the end of the null-pointer trap before it and the call (`mov dword ptr [0],imm; mov rdi,[rbp+..]; mov rcx,[rdi+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`). Recipe 1 is a second GuiData function with the same test at its start (`mov rsi,rcx; mov rcx,[rcx+member]; ...; call [guard]; test al,al; je`, then useTouchscreen on the same member). Both displacements are turned into slots. Recipe 2 is the body of ClientInstance::getMouseGrabbed (a 64-byte function that calls isPrimaryClient and tail-calls the MinecraftGame) and `slot_of` in the ClientInstance.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). All three are unique in both exes and agree. The range check keeps the slot near the input-mode queries. What changed in 26.60: the slot moved, a `nop` follows the call in recipe 0's site, and the old recipe 1 site was rewritten. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 122 (7Ah), 23 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; isShowingMenu went from 148 to 125). Recipe 1 and recipe 2 gave 122 and agreed; the range window was too narrow (now 118 to 170). Recipe 0 found nothing: the null-pointer trap before the call is now `mov rax,[rip+..]; mov dword ptr [rax],imm` and no longer an absolute store, so its anchor is gone. Recipe 0 now starts at the member load and the call itself and runs on to the second call in the same function (`mov rcx,[rdi+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; nop; test al,al; je; mov rcx,[rdi+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; nop; test al,al; je`: getMouseGrabbed then useTouchscreen), with offset 10 at the displacement of the first call; it matches once on 26.63 and on the Preview. Evidence on the Preview, both from 26.63: (1) the three recipes are in three places (the GuiData function of recipe 0, the one of recipe 1, the ClientInstance body of recipe 2), each drafted from 26.63, each found once on the Preview, and all give slot 122; (2) the slot moved by the same 23 as isShowingMenu (148 to 125) and isWorldActive (157 to 134), which are resolved entries, and the 26.63 order getMouseGrabbed, useTouchscreen is the same in the Preview call site (displacements 3D0h and 3C8h, slots 122 and 121). Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 84 C0 74 ?? 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 84 C0 74 ??"
      offset = 10
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
    min = 118
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
  notes     = "void ClientInstance::onTick(int nTick, int maxTick), the GameCallbacks::onTick override: slot 4 of the ClientInstance.GameCallbacks.vtable global (destructor, onLevelCorrupt, onGameModeChanged, onBeforeSimTick, onTick; slot 7 is onLevelExit, ClientExitLevel.hook). `this` in rcx is the GameCallbacks subobject (ClientInstance + ClientInstance.gameCallbacksOffset, 88h), as for ClientExitLevel.hook; edx = nTick, r8d = maxTick. It runs on the main thread once per client tick (Minecraft::mRealTimer, 20 per second, more than once in a frame when the game catches up), in menus too. The first call is getLocalPlayer on this-88h; the in-world work is skipped when it is null. No direct caller: it is only reached through the vtable. Recipe 0 is the code after the getLocalPlayer null test: `mov rbx,[rax+member]; mov rcx,[rsi+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]` (one match in the function on every build), then function_start_of; it does not touch the prologue, whose push list and frame changed between builds. Recipe 1 shares no bytes with it: the end of the function, a pattern_any with one alternative per shape. Alternative 0 (26.62, 26.63) is `mov rcx,[rsi+member]; call; movaps xmm6,[rbp-..]; add rsp; pop rbx/rdi/rsi/r14/r15/rbp; ret` followed by the first bytes of the cold path (`mov dword ptr [0],0DEADC0DEh`). Alternative 1 (Preview 1.26.70.26) is the same end without the xmm6 restore, with `nop; add rsp,imm32` and one pop less, and no cold path behind the ret; it starts at the two calls before it (`xor edx,edx; call [guard]; nop; lea rcx,[rbp+..]; call; mov rcx,[rsi+member]; call; nop`) because the end alone matches a second function. Each alternative matches once on its own builds and not on the others; recipe 1 then takes function_start_of. Every displacement, immediate and branch byte is a wildcard. The check in_vtable keeps the value to slots 2 to 6 of the GameCallbacks vtable (a window around slot 4; the neighbours onBeforeSimTick and onInternetUpdate are other functions). What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the function has one push less (no r15), no xmm6 save, a larger frame, getLocalPlayer is called with a one-byte displacement (slot 0Dh), and the cold path after the ret is gone; both old recipes found nothing. Evidence on the Preview, both from 26.63: (1) the pattern after the null test, drafted from the 26.63 body, matches once on the Preview, in the function that holds slot 4 of the GameCallbacks vtable; (2) the 26.63 body calls getLocalPlayer first (slot 1Eh there) and the Preview body calls slot 0Dh, the resolved ClientInstance.getLocalPlayer value, at the same place, on this-88h, with the same two member loads and the same call through slot 148h after it. Would have survived with a better recipe. Values: 0x65a0c90 on 6aea437e-7eff13daabb430da (26.62), 0x65a09f0 on 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "pattern" {
      bytes = "48 8B 98 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 8E ?? ?? ?? ?? E8 ?? ?? ?? ?? 0F 28 75 ?? 48 83 C4 ?? 5B 5F 5E 41 5E 41 5F 5D C3 C7 04 25",
        "31 D2 FF 15 ?? ?? ?? ?? 90 48 8D 4D ?? E8 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 81 C4 ?? ?? ?? ?? 5B 5F 5E 41 5E 5D C3",
      ]
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
  notes = "Offset of the GameCallbacks subobject in ClientInstance: 88h in 26.62 and 26.63. It is the displacement of the store that puts the ClientInstance.GameCallbacks.vtable pointer into the object: the `this` that a GameCallbacks override such as ClientInstance.onTick or ClientExitLevel.hook receives in rcx is ClientInstance + this value, and the ClientInstance is rcx minus it. Recipe 0 is the constructor run of ClientInstance.GameCallbacks.vtable (see that entry): the fourth `lea rax,[rip+..]; mov [reg+disp32],rax` store, read as the displacement of operand 0 of the `mov` (offset 66 from the pattern start, 7 bytes after the `lea` that the vtable entry reads). Every REX, ModRM, displacement and immediate byte of the pattern is a wildcard. A second recipe is not possible: the other uses of the offset are `lea reg,[rcx-88h]` in the overrides, a negative displacement that no step negates. Recipe 0 is the same `pattern_any` as ClientInstance.GameCallbacks.vtable: alternative 0 is the old pattern (26.51 to 26.63), alternative 1 is the SIB encoding of the stores through r12 (Preview 1.26.70.23 and later). The `lea` of the fourth store is at offset 59 in both alternatives and the `mov` is at offset 66, so the operand is read from the same place. Value 0x88 on 6aea437e-7eff13daabb430da (26.62), 6aed1906-b127418e74d2f2f9 (26.63) and 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26). What changed in 6afa4e47-dadc375797a3d31a: nothing for this entry; its anchor is the one that the Preview 1.26.70.23 repair gave the vtable entry. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? C7 ?? ?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ??",
        "?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ??",
      ]
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
  notes     = "Slot of void IClientInstance::onStartJoinGame(bool isLocalServer, std::string const&, std::string const&, std::string const& worldName, NetworkType, MultiplayerServiceIdentifier, PlayerJoinWorldContext) in the ClientInstance.vtable global. Arguments: rcx = ClientInstance, dl = isLocalServer, r8 and r9 = the two string references, then stack arguments at entry+28h (worldName), +30h (NetworkType), +38h (the service identifier) and +40h (a pointer to the by-value PlayerJoinWorldContext). Recipe 0 is the body after the head: `mov byte ptr [this+member],imm; mov rcx,[this+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; nop; mov r15b,al; mov rax,[this]; mov rax,[rax+disp]; mov rcx,this; call [guard]; nop; lea r12,[rbp+disp]; mov rcx,r12; mov rdx,rax; mov r8d,r15d; call` (the IMinecraftGame* member, then two virtual calls whose results feed one helper call), then function_start_of and slot_of ClientInstance.vtable. It ends at the call. The head is not in the pattern: 26.63 reads the members straight after the argument moves, 70.26 puts a thread-local static init guard (`mov rdx,gs:[58h]`) there. Every immediate, displacement and branch byte is a wildcard, and so are the REX and ModRM bytes of the member accesses (this is rcx in the old shape and rsi in the new one). Recipe 1 is a structural route: the slot of ClientInstance.getGameConnectionInfo plus 1 (that getter is the slot before this one). The two recipes share no anchor bytes. The range check is a window around the known values. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the table was reordered (the getter is slot 2, this entry slot 3; 26.63 has 10 and 11), the frame grew, the members moved and a static init guard came between the head and the member reads. Class: would have survived with a better recipe (the head anchor was too long). Evidence for 70.26: (1) the body pattern of recipe 0, drafted from the 26.63 function, matches once on 26.63 and once on 70.26, in the function whose prologue (eight pushes, `mov qword ptr [rbp+..],-2`, `mov r?,r9; mov rdi,r8; mov ?,edx; mov rsi,rcx`) is the same as the old function; (2) the old function is the slot right after the getGameConnectionInfo slot (10 and 11), and on 70.26 getGameConnectionInfo (three agreeing recipes) is slot 2, the next slot holds that same function. The check range was widened from 7..16 to 2..16 on this evidence. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes = "C6 ?? ?? ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 41 88 C7 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 90 4C 8D A5 ?? ?? ?? ?? 4C 89 E1 48 89 C2 45 89 F8 E8"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  recipe {
    step "ref" { entry = "ClientInstance.getGameConnectionInfo" }
    step "add" { value = 1 }
  }

  check "range" {
    min = 2
    max = 16
  }
}

entry "ClientInstance.startExternalNetworkWorld" {
  kind      = "vslot"
  hooked    = true
  msvc_name = "?$startExternalNetworkWorld@ClientInstance@@QEAAXVGameConnectionInfo@Social@@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@UPlayerJoinWorldContext@@@Z"
  notes     = "Slot of void ClientInstance::startExternalNetworkWorld(Social::GameConnectionInfo, std::string const& serverName, PlayerJoinWorldContext) in the ClientInstance.vtable global: 332 in 26.51, 328 in 26.60 (the header forecast is one more). The protected _startExternalNetworkWorld is folded into this virtual. Arguments: rcx = ClientInstance, rdx = a pointer to the by-value GameConnectionInfo, r8 = serverName, r9 = a pointer to the by-value PlayerJoinWorldContext; it returns void. The callee destroys both by-value arguments (GameConnectionInfo.dtor on the saved rdx, and the inlined PlayerJoinWorldContext destructor on the saved r9), so a caller passes pointers to objects that it built and does not destroy them after the call. Evidence: the std RTTI of the `_Func_impl` of its lambda (`<lambda_44>` in 26.51, `<lambda_46>` in 26.60) leads to a vtable that only this function loads. Recipe 0 is the head after the frame set-up: `mov [rbp+..],r9; mov rbx,r8; mov rdi,rcx; mov [rbp+..],rdx; cmp qword ptr [rdx+member],0; je; mov rax,[rbp+..]; cmp qword ptr [rax+member],0; jne; mov rax,[rbp+..]; cmp qword ptr [rax+..]` (the GameConnectionInfo fields 120h, 0E0h and 188h; 168h in 26.60), then `function_start_of` and `slot_of` ClientInstance.vtable. Recipe 1 is a call site in the slot before it (331 in 26.51, 327 in 26.60), which builds a GameConnectionInfo and calls this slot on `this`: `mov rax,[rsi]; mov rax,[rax+disp]; lea r8,[rip+..]; lea rdx,[rbp+..]; lea r9,[rbp+..]; mov rcx,rsi; call [guard]`; it reads the displacement at offset 3 and turns it into a slot. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. The range check is a window around the two values. Values: slot 332 in 26.51 (0x5daf490), 328 in 26.60 (0x659ee60). What changed in 26.60: the slot moved down by four, the frame is smaller and the GameConnectionInfo layout after +8Ch shrank by 20h. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 300 (12Ch), 28 lower than on 26.63 (the ClientInstance vtable lost virtuals before it). Recipe 1 still matched and read 300. Recipe 0 found nothing: the register allocation changed (the server name is kept in r14 and this in rbx, not rbx and rdi), so the two register moves after the store of r9 are now wildcards (`mov [rbp+..],r9; mov reg,r8; mov reg,rcx; mov [rbp+..],rdx; cmp qword ptr [rdx+member],0; ...`); the pattern matches once on 26.63 and once on the Preview, in the same function. The window is now 296 to 340. Evidence on the Preview, both from 26.63: (1) recipe 0, drafted from the 26.63 head, is found once on the Preview, in a function that begins with the same tests of three GameConnectionInfo members (offsets 138h, 0F8h and 180h) and then calls slot 64 (200h) on this, and slot_of gives 300; (2) recipe 1, the call site of the slot before it in a second function, reads the same slot, 300, and the Preview function takes the same arguments (rcx this, rdx the GameConnectionInfo, r8 the name, r9 the context, saved at the frame head). Would have survived with a better recipe. Hooked (sd-batch exe survey): the slot target is 0x659f7c0 on 6aed1906-b127418e74d2f2f9 (26.63) and 0x8b284c0 on 6afa4e47-dadc375797a3d31a (70.26). Both save r9, r8, rcx and rdx first. rdx is the by-value GameConnectionInfo passed by pointer: the head reads [rdx+120h] on 26.63 and [rdx+138h] on 70.26, 18h further, as GameConnectionInfo.mUnresolvedUrl moves from 28h to 40h. r9 is the by-value PlayerJoinWorldContext passed by pointer: both read the byte [r9+22h], move the 20h-byte string at [r9], read the dword [r9+20h] and test `cmp byte ptr [r9+0A8h],1`, the PlayerJoinWorldContextLayout offsets. Recipe 0 names ClientInstance.vtable in its slot_of step, so the loader patches that table. `hooked = true`: `signature` probes the slot target (rcx, rdx, r8, r9 on both builds: same)."

  recipe {
    step "pattern" {
      bytes = "4C 89 8D ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 89 95 ?? ?? ?? ?? 48 83 BA ?? ?? ?? ?? ?? 74 ?? 48 8B 85 ?? ?? ?? ?? 48 83 B8 ?? ?? ?? ?? ?? 75 ?? 48 8B 85 ?? ?? ?? ?? 48 83 B8"
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
    min = 296
    max = 340
  }
}

entry "ClientInstance.requestLeaveGame" {
  kind      = "vslot"
  msvc_name = "?$requestLeaveGame@ClientInstance@@QEAAX_N0@Z"
  notes     = "Slot of void IClientInstance::requestLeaveGame(bool switchScreen, bool sync) in the ClientInstance.vtable global: 15 in 26.51 and 14 in 26.60 (one IClientInstance virtual before slot 13 went away, as for ClientInstance.update; update is 9 slots after it in both builds). Arguments: rcx = ClientInstance, dl = switchScreen, r8b = sync; it returns void. Evidence: the function loads the string 'virtual void __cdecl ClientInstance::requestLeaveGame(bool, bool)' (`inspect xrefs --string` gives only this function: 0x5da3d10 in 26.51, 0x6593960 in 26.60), and both addresses are what the ClientInstance.vtable holds at slot 15 and slot 14. Recipe 0 is the first virtual call after the frame set-up: `test bl,bl; mov [rbp+..],reg; jne rel32; mov rax,[r14]; mov rax,[rax+disp]; mov rcx,r14; call [guard]` (a bool-returning virtual of the same object; displacement 510h in 26.51, 500h in 26.60). Recipe 1 shares no bytes with recipe 0: it is the packing of the two bools further down (`movzx ecx,reg8; movzx eax,bl; shl eax,imm; or eax,ecx; mov byte ptr [r14+member],imm; mov reg,[r14+member]`). Both end with `function_start_of` and `slot_of` ClientInstance.vtable. Every immediate, displacement and branch byte of the patterns is a wildcard; only the register bytes are kept as they are. Both recipes are unique in both exes and agree. The range check is a window around the two values. Values: slot 15 in 26.51 (0x5da3d10), 14 in 26.60 (0x6593960). What changed in 26.60: the slot moved down by one; the frame is 10h bigger; a null check of a member at 178h and a `nop` after the first call were added; the body has the same arguments. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 6, eight lower than on 26.63 (the IClientInstance vtable lost virtuals before it, as for onStartJoinGame, slot 11 there and 3 now); both recipes still match, in the one function, and agree, so only the range window was too narrow. The window is now 2 to 19 (about four slots either side of 6 and 15). Evidence: (1) both recipes, drafted from the 26.63 body, find the same function on the Preview, and slot_of gives 6; (2) that slot of the Preview vtable loads the text 'virtual void __cdecl ClientInstance::requestLeaveGame(bool, bool)', as the 26.63 slot 14 does. Would have survived with a wider window."

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
    min = 2
    max = 19
  }
}

entry "ClientInstance.getMinecraftInput" {
  kind      = "vslot"
  msvc_name = "?$getMinecraftInput@ClientInstance@@QEBA?AV?$NonOwnerPointer@VMinecraftInputHandler@@@Bedrock@@XZ"
  notes     = "Slot of Bedrock::NonOwnerPointer<MinecraftInputHandler> IClientInstance::getMinecraftInput() const in the ClientInstance.vtable global: 267 (displacement 858h) in 26.51 and 264 (840h) in 26.60 (the header order gives one more than the exe in 26.60, as for the other slots after slot 13). Arguments: rcx = ClientInstance, rdx = a hidden pointer to a 24-byte buffer; it returns that pointer in rax. The buffer holds the NonOwnerPointer: the control block at +0, the reference at +8 and the MinecraftInputHandler pointer at +10h; the caller releases the shared pointer at +0/+8. Evidence: the body (0x5dce200 in 26.51, 0x65bad00 in 26.60) forwards to slot 134 (430h) of the MinecraftGame held at [rcx+1A8h] (1C8h in 26.60) with the same rdx; it is a frame and a call in 26.51 and a frameless tail jump in 26.60. Recipe 0 is a call site: a function that holds the client at [this+member] calls the slot into a stack buffer (`mov qword ptr [rbp+..],imm; mov [rbp+..],r8; mov rsi,rdx; mov rbx,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]`); it reads the displacement at offset 25 and turns it into a slot. Recipe 1 shares no bytes with recipe 0: the body of ClientInstance.setLastPointerLocation (three `movss [rcx+member],xmmN` stores and `ret`, a leaf that is two slots after this one in both builds: 269 and 266), `slot_of` ClientInstance.vtable, minus 2. Every immediate, displacement and branch byte of the patterns is a wildcard. Both recipes are unique in both exes and agree. The range check is a window around the two values. Values: slot 267 in 26.51 (0x5dce200), 264 in 26.60 (0x65bad00). What changed in 26.60: the slot moved down by three; the body lost its frame; the MinecraftGame member moved. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 238 (0EEh), 26 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; getCurrentUIRouter went from 239 to 213, also 26). Recipe 1 still matched (setLastPointerLocation, slot_of, minus 2) and read 238. Recipe 0 found nothing: the call-site function now has four pushes (rbp added) and stores to its frame with 32-bit displacements (`mov qword ptr [rbp+..],0FFFFFFFFFFFFFFFEh` and `mov [rbp+..],r8` in the long form). Recipe 0 is now a pattern_any with one alternative per encoding; alternative 1 starts inside the wildcarded tail of the frame-marker store so that the displacement of the slot call stays at offset 25 in both. Each alternative matches once on its own builds and not on the others. The window is now 234 to 272. Evidence on the Preview, both from 26.63: (1) recipe 0, drafted from the 26.63 call-site function, is found once on the Preview (the same function: `mov r8` stored, `mov rsi,rdx; mov rbx,rcx; mov rcx,[rcx+8]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]; nop`) and reads slot 238; (2) recipe 1, the body of setLastPointerLocation, which is two slots after this one on every build, reads the same slot through slot_of and a different function. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 C7 45 ?? ?? ?? ?? ?? 4C 89 45 ?? 48 89 D6 48 89 CB 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ??",
        "?? ?? ?? ?? ?? 4C 89 85 ?? ?? ?? ?? 48 89 D6 48 89 CB 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ??",
      ]
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
    min = 234
    max = 272
  }
}

entry "ClientInstance.getTextureGroup" {
  kind     = "vslot"
  notes    = "Slot of `std::shared_ptr<mce::TextureGroup> IClientInstance::getTextureGroup() const` in the IClientInstance vtable: 103 on 26.60, 105 on 26.51. The ClientInstance override calls MinecraftGame slot 10 (`[rax+50h]`) on the IMinecraftGame member and returns its result through the hidden out pointer in rdx (16 bytes: element, control block). On 26.60 it is a leaf thunk without .pdata (0x65ac2c0: `mov rcx,[rcx+1C8h]; mov rax,[rcx]; mov rax,[rax+50h]; mov r8,[guard]; jmp r8`); on 26.51 it has a frame (0x5dbe030: `push rsi; sub rsp,..; mov rsi,rdx; mov rcx,[rcx+1A8h]; mov rax,[rcx]; mov rax,[rax+50h]; call [guard]`). Recipe 0 is the call in HudLocatorBarRenderer.render (`movaps [rbp+..],xmm0; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; lea rdx,[rbp+..]; call [guard]`); the displacement is turned into a slot. render then adds 18h to the group before it calls TextureGroup.getTexture. Recipe 1 is the override itself, from its first byte, then slot_of the ClientInstance.vtable: a pattern_any with one alternative per build (alternative 0 the 26.60 thunk, alternative 1 the 26.51 function); each matches once on its own build and never on the other. The sibling overrides (slots 102 and 104 on 26.60) have the same shape and forward other MinecraftGame slots, and a thunk of another class forwards slot 10 from another member (+0A0h), so the member offset (2 bytes) and the slot displacement (1 byte) are kept in each alternative. The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. The range check is a window around the slots of both builds. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 82 (52h), 21 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; getTopScreenName went from 134 to 113). Recipe 0 still matched and read 82. Recipe 1 found nothing: the IMinecraftGame member of ClientInstance moved from 1C8h to 1D0h. Recipe 1 has a third alternative with the member 1D0h, the same thunk (`mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+50h]; mov r8,[rip+..]; jmp r8`); the member bytes stay literal, because a thunk of another class forwards the same slot from another member and the wildcarded form has three matches. It matches once on the Preview, in no function (a leaf without .pdata, found through slot_of), and not on 26.63. The window is now 78 to 113. Evidence on the Preview, both from 26.63: (1) recipe 0, a call site drafted from 26.63, reads slot 82; (2) the body of recipe 1, the thunk of 26.63 (slot 103), is found once on the Preview with the new member offset, and slot_of finds it in slot 82, the same slot. Would have survived with a better recipe."

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
        "48 8B 89 D0 01 ?? ?? 48 8B 01 48 8B 40 50 4C 8B 05 ?? ?? ?? ?? 49 FF E0",
      ]
      offset     = 0
      literal_ok = [
        "0.3: the IMinecraftGame member 1C8h of ClientInstance (26.60): a thunk of another class forwards the same slot from +0A0h",
        "0.4: the IMinecraftGame member 1C8h of ClientInstance (26.60): a thunk of another class forwards the same slot from +0A0h",
        "0.13: MinecraftGame slot 10 (50h): the sibling ClientInstance thunks forward other slots of the same member",
        "1.11: the IMinecraftGame member 1A8h of ClientInstance (26.51): keeps the pattern on ClientInstance overrides",
        "1.12: the IMinecraftGame member 1A8h of ClientInstance (26.51): keeps the pattern on ClientInstance overrides",
        "1.21: MinecraftGame slot 10 (50h): the sibling ClientInstance overrides call other slots of the same member",
        "2.3: the IMinecraftGame member 1D0h of ClientInstance (Preview 1.26.70.26): a thunk of another class forwards the same slot from another member",
        "2.4: the IMinecraftGame member 1D0h of ClientInstance (Preview 1.26.70.26), second byte",
        "2.13: MinecraftGame slot 10 (50h): the sibling ClientInstance thunks forward other slots of the same member",
      ]
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
  }

  check "range" {
    min = 78
    max = 113
  }
}

entry "ClientInstance.getMinecraftGame" {
  kind     = "vslot"
  notes    = "Slot of IMinecraftGame& IClientInstance::getMinecraftGame_DEPRECATED() in the ClientInstance vtable: 78 on 26.60 (offset 270h). rcx the client instance; the IMinecraftGame* in rax. The game passes it as the third argument of the BaseActorRenderContext constructor everywhere it draws an inventory item. Recipe 0 is that call in the helper that draws a flying item (`mov rax,[r14]; mov rax,[rax+..]; mov rcx,r14; call [guard]; mov rdx,[rsi+..]; lea r15,[rbp+..]; mov rcx,r15; mov r8,r14; mov r9,rax; call`), read as the displacement of the load and turned into a slot. Recipe 1 is the same call in the inventory item renderer (`mov rax,[r8]; mov rax,[rax+..]; mov rcx,r8; mov rbx,r8; call [guard]; lea r15,[rbp+..]`). The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window like the neighbouring ClientInstance slots. Value 78 on 6ad79ee4-17d51891a1119914 (26.60). What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the slot is 57 (39h), 21 lower than on 26.63 (the ClientInstance vtable lost virtuals before it; getTopScreenName went from 134 to 113); the window 50 to 140 holds it. Recipe 0 still matched and read 57. Recipe 1 found nothing: the register allocation changed (`mov r15,r8` and `lea rbx,[rbp+..]` where 26.63 has rbx and r15), so the two register moves before the call and the lea and mov after it are wildcards; it matches once on 26.63 and once on the Preview, in the same function. Evidence on the Preview, both from 26.63: (1) recipe 0 and recipe 1 are in two functions, drafted from 26.63, each found once on the Preview, and read the same slot, 57; (2) the recipe 1 function calls the slot through r8 and then passes the result as r9 to the render context helper (the function that LevelRenderer.renderLevel calls first, a resolved entry on both builds), as on 26.63. No build gate: builds before 26.63 are not supported."

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
      bytes  = "49 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 C1 ?? ?? ?? FF 15 ?? ?? ?? ?? ?? 8D ?? ?? ?? 89 ??"
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
}

entry "ClientInstance.mMinecraft" {
  kind  = "field"
  notes = "Offset of `mMinecraft` in ClientInstance: 1D0h in 26.60. The member is a std::unique_ptr<Minecraft> (8 bytes): the client Minecraft object, which the old LeviLamina API returned from ClientInstance::getServerData(). 26.60 has no ClientInstance virtual that only returns it: the one `mov rax,[rcx+1D0h]; ret` body in the exe (0xec69890) is in five other vtables and not in ClientInstance.vtable, so a mod reads the field. ClientInstance::init (slot 3, 0x658be40) calls a std::function factory with the pair {&this+E58h, &this+ED8h} (the two client timers), moves the returned pointer into [this+1D0h] and deletes the old one through its virtual destructor (`mov rax,[rcx]; mov rax,[rax]; mov edx,1; call`). ClientInstance::update (0x65977b0) passes [this+1D0h] to Minecraft::update (0xb82190) right after it advances the two timers; the server update lambda (0x1048dc0) passes [ServerInstance+B8h] to the same function. The ClientInstance slot-26 leaf (0x659eba0) tests [this+1D0h] for null before it reads [[this+1D0h]+E8h] (Minecraft.mSimTimer), so the pointer can be null. Recipe 0 is the release of the member in the ClientInstance destructor (the function that stores ClientInstance.vtable): `mov edx,80h; mov rcx,r14; call` (the free of a 80h-byte Timer, the last member released before it), then `mov rcx,[rsi+member]; mov qword ptr [rsi+member],0; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax]; mov edx,1; call [guard]` (the unique_ptr reset and the deleting destructor); it reads the displacement of operand 1 of the first `mov` (offset 13). Recipe 1 is in ClientInstance::update (ref ClientInstance.update): the end of the real-timer advance and the Minecraft::update call, `xor r8d,r8d; call; nop; mov rcx,[rsi+member]; call`; it reads the displacement of operand 1 of the `mov` (offset 9). It runs past one call, because 26.60 puts a `nop` there; a build without the nop breaks it, and it fails then rather than give a wrong value. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range 100h..400h. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the member is 1E0h, 10h higher than on 26.63. Recipe 0 (ClientInstance::init) found nothing: init is no longer a ClientInstance virtual and no store of the member remains in the ClientInstance code, so recipe 0 now reads the destructor, which has the same release on 26.63 and the Preview (one match on each build). Evidence on the Preview, both from 26.63: (1) recipe 1, drafted from the 26.63 update, is found once on the Preview and reads 1E0h; (2) recipe 0, drafted from the 26.63 destructor, is found once on the Preview in the function that stores ClientInstance.vtable (found with xrefs on the vtable) and reads 1E0h. Would have survived with a better recipe. Value 1D0h on 6ad79ee4-17d51891a1119914 (26.60). No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes  = "BA 80 00 00 00 4C 89 F1 E8 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 00 BA ?? ?? ?? ?? FF 15"
      literal_ok = [
        "1: the size 80h of the Timer free: the destructor frees another member with the same release shape and another size",
        "2: the size 80h, second byte",
        "3: the size 80h, third byte",
        "4: the size 80h, fourth byte",
      ]
      offset = 13
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
}

entry "ClientInstance.mSimTimer" {
  kind  = "field"
  notes = "Offset of the client sim Timer in ClientInstance: E58h in 26.60. The member is an inline Timer (80h bytes), not a pointer: the code passes `lea rcx,[this+E58h]` as the Timer's `this`. The name is inferred: Minecraft.mSimTimer (a Timer&) of the client Minecraft points at it. Evidence: ClientInstance::update (0x65977b0) advances [this+E58h] and then [this+ED8h] with Timer::advanceTime(rcx, 1.0f, false) when its third argument (r14b) is set, then calls Minecraft::update with ClientInstance.mMinecraft; when the argument is clear it only recomputes mAlpha (+8) of both from mLastMs (+30h), mTimeScale (+0Ch), mTicksPerSecond (+0) and mPassedTime (+10h), and checks the std::function at +78h of each Timer (E58h+78h = ED0h, ED8h+78h = F50h). ClientInstance::init (slot 3, 0x658be40) passes {&this+E58h, &this+ED8h} to the Minecraft factory, whose lambda (0x8d3120) puts them in the Minecraft construction arguments at +40h/+48h; the Minecraft constructor (0xb80b30) stores them at Minecraft+E8h/+F0h. So [ClientInstance.mMinecraft + E8h] == this+E58h. Recipe 0 is the construction of the Timer in the ClientInstance constructor: `lea rcx,[reg+sim]; xorps xmm0,xmm0; movups [reg+..],xmm0 (two or three times, and on 26.60 to 26.63 a qword store); lea rsi,[rip+..]; mov [rbp+..],rsi`, then the Timer constructor call; it is a pattern_any with one alternative per shape of the stores in front of the `lea rsi` (alternative 0 26.60 to 26.63, alternative 1 the Preview), and it reads the displacement of operand 1 of the first `lea` (offset 0). Recipe 1 is the first advance in ClientInstance::update (ref ClientInstance.update): `test r14b,r14b; je rel32; lea rcx,[rsi+sim]; movss xmm1,[rip+..]; xor r8d,r8d; call`; it reads the displacement of operand 1 of the `lea` (offset 9). The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range C00h..1400h. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the member is F00h, A8h higher than on 26.63. Recipe 0 (ClientInstance::init) found nothing: init is no longer a virtual of ClientInstance and no function of the Preview other than update and the constructor takes both Timer offsets; recipe 0 now reads the constructor, where 26.63 constructs the same two Timers in the same order (E58h then ED8h). Evidence on the Preview, both from 26.63: (1) recipe 1, drafted from the 26.63 update, is found once on the Preview and reads F00h; (2) recipe 0, drafted from the 26.63 constructor (one match on each build), reads F00h in the constructor on the Preview, where the next Timer, the real one, is built right after it as in 26.63. Would have survived with a better recipe. Value E58h on 6ad79ee4-17d51891a1119914 (26.60). No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 8B ?? ?? ?? ?? 0F 57 C0 0F 11 83 ?? ?? ?? ?? 0F 11 83 ?? ?? ?? ?? 48 C7 83 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 35 ?? ?? ?? ?? 48 89 B5",
        "48 8D 8A ?? ?? ?? ?? 0F 57 C0 0F 11 82 ?? ?? ?? ?? 0F 11 82 ?? ?? ?? ?? 48 8D 35 ?? ?? ?? ?? 48 89 B5",
      ]
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
}

entry "ClientInstance.mRealTimer" {
  kind  = "field"
  notes = "Offset of the client real-time Timer in ClientInstance: ED8h in 26.60, 80h (one Timer) after ClientInstance.mSimTimer. The member is an inline Timer (80h bytes), not a pointer. The name is inferred: Minecraft.mRealTimer (a Timer&) of the client Minecraft points at it. Evidence: ClientInstance::update (0x65977b0) advances it second, right after [this+E58h], with Timer::advanceTime(rcx, 1.0f, false); ClientInstance::init (slot 3, 0x658be40) passes it as the second element of the timer pair to the Minecraft factory, and the Minecraft constructor (0xb80b30) stores it at Minecraft+F0h. Recipe 0 is the construction of the real Timer in the ClientInstance constructor, after the sim Timer: `lea rcx,[reg+real]; mov [rbp+..],rsi; mov [rbp+..],reg; lea r8,[rbp+..]; mov [rbp+..],r8; movss xmm1,[rip+..]; call`; every register, displacement and immediate byte is a wildcard. It reads the displacement of operand 1 of the `lea` (offset 0). Recipe 1 is the second advance in ClientInstance::update (ref ClientInstance.update): `xor r8d,r8d; call; nop; lea rcx,[rsi+real]; movss xmm1,[rip+..]; xor r8d,r8d; call`; it reads the displacement of operand 1 of the `lea` (offset 9). It runs past one call, because 26.60 puts a `nop` there; a build without the nop breaks it, and it fails then rather than give a wrong value. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. Both are unique in 26.60 and agree. Check: range C00h..1400h. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the member is F70h, 98h higher than on 26.63 (70h after the sim Timer, not 80h). Recipe 0 (ClientInstance::init) found nothing, as for ClientInstance.mSimTimer; it now reads the constructor. Evidence on the Preview, both from 26.63: (1) recipe 1, drafted from the 26.63 update, is found once on the Preview and reads F70h; (2) recipe 0, drafted from the 26.63 constructor (one match on each build), reads F70h in the Preview constructor, where it follows the sim Timer construction as in 26.63. Would have survived with a better recipe. Value ED8h on 6ad79ee4-17d51891a1119914 (26.60). No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes = "?? 8D ?? ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 4C 89 85 ?? ?? ?? ?? F3 0F 10 0D ?? ?? ?? ?? E8"
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
}

entry "ClientInstance.getCurrentUIRouter" {
  kind     = "vslot"
  notes     = "Slot of OreUI::Router& IClientInstance::getCurrentUIRouter() in the ClientInstance.vtable global: 239 (displacement 778h) on 26.60, 26.62 and 26.63, 213 (6A8h) on the Preview 1.26.70.26. The header (LeviLamina IClientInstance.h) declares the non-const and the const overload one after the other; MSVC puts the const one first. On 26.60 to 26.63 the exe folds both into one function, so the two slots hold the same body (26.63 0x65b4fa0); the game calls the second. On the Preview each has its own body (they carry an assert with a different text length) and the vtable names them: slot 212 const, slot 213 non-const. rcx = the ClientInstance; the Router pointer is returned in rax (a reference). The body returns the member router (+5F0h on 26.63, +600h on the Preview) when the client shows no world or an overlay flag is set, else forwards to IMinecraftGame slot 123 (3D8h) on the game member. The mod must call the slot, not read the member. Recipe 0 is a call site: a function that calls the router getter on a client held in a member and then, after a nop, builds a call on the result (`mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; nop; lea rcx,[rbp+..]; mov rdx,rax; mov r8b,imm; call; nop; mov r15,[rbp+..]`); it reads the displacement and turns it into a slot. It is found once on 26.63 and once on the Preview. Recipe 1 shares no bytes with it: the body of the const overload, then slot_of ClientInstance.vtable and add 1 (the non-const overload is the next slot on every build). It is a pattern_any with one alternative per shape. Alternative 0 (26.60 to 26.63, one folded body) is `push rsi; sub rsp,..; mov rsi,rcx; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test al,al; je; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; cmp byte ptr [rax+40h],0; je; mov rax,[rsi+member]`; the first slot that holds it is the const slot (238), so add 1 gives 239. Alternative 1 (Preview) is the prologue with the stack cookie and the member assert up to the length of the assert text (`mov qword ptr [rsp+..],47h`: the length of the const overload text, 59 for the non-const one); that byte is kept, because it is the only thing that tells the two bodies apart; slot_of gives 212 and add 1 gives 213. Every other displacement, immediate and branch byte is a wildcard. The check is a range window 209 to 248. What changed in 6afa4e47-dadc375797a3d31a (Preview 1.26.70.26): the vtable lost virtuals before this slot (239 to 213), the compiler no longer folds the two overloads (assert text), and the Play tab and the route push (the two call sites of the old recipes 0 and 1) now go through a helper or call the IMinecraftGame slot directly; the old recipe 1 matched the call of the game slot (3D8h) and gave slot 123, so it was replaced. Evidence on the Preview, both from 26.63: (1) the call-site function of recipe 0, drafted from 26.63 (and found once there), is found once on the Preview and calls the slot at displacement 6A8h, 213; (2) the 26.63 body (recipe 1, first slot 238 plus 1 for the non-const overload) is found on the Preview as the const overload in slot 212, and the vtable names slot 213 as the non-const getCurrentUIRouter and slot 212 as the const one. Would have survived with a better recipe (class: real code change for the folding)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 48 8D 8D ?? ?? ?? ?? 48 89 C2 41 B0 ?? E8 ?? ?? ?? ?? 90 4C 8B BD ?? ?? ?? ??"
      offset = 3
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
        "56 48 83 EC ?? 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 80 78 ?? ?? 74 ?? 48 8B 86 ?? ?? ?? ??",
        "56 48 83 EC ?? 48 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 83 B9 ?? ?? ?? ?? ?? 75 ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? 47 ?? ?? ??",
      ]
      literal_ok = [
        "1.50: the length of the assert text in the const overload (47h); the non-const overload has the same body with 3Bh, so this byte tells them apart (Preview 1.26.70.26)",
      ]
    }
    step "slot_of" { vtable = "ClientInstance.vtable" }
    step "add" { value = 1 }
  }

  check "range" {
    min = 209
    max = 248
  }
}

entry "ClientInstance.connectByAddress" {
  kind  = "function"
  notes = "The ClientInstance virtual that joins a server by address and port through startExternalNetworkWorld: void (this, const std::string& ip, int port); rcx the object, rdx the ip string, r8d the port (`mov ebx,r8d; mov rdi,rdx; mov rsi,rcx`: 26.63 0x65a0552, 70.26 0x8b29880). The exe has no symbol for it; the name says what it does. It builds the GameConnectionInfo with the six-argument constructor (type 8 `UnknownIP`, transport 4, the ip, the port and a ThirdPartyInfo that the MinecraftGame fills for the address: `mov dx,8; mov r8d,4; mov r9,rdi; call` at 26.63 0x65a06d9 (ctor 0x4ec8b0) and 70.26 0x8b29a9c (ctor 0xeeaba0)), sets a flag word to 101h when the third-party fields are empty, builds a default PlayerJoinWorldContext and passes copies of both to ClientInstance.startExternalNetworkWorld (`mov rax,[rsi]; mov rax,[rax+disp]; lea r8,[rip+..]; lea rdx,..; lea r9,..; mov rcx,rsi; call [guard]`, 26.63 0x65a0751, 70.26 0x8b29bc9). The loader calls it for client.join_server_direct, so the game builds both by-value arguments itself: on 70.26 the PlayerJoinWorldContext is 160h bytes and its default constructor allocates a shared state (a 70h-byte control block, vtable 0x12ded8c0, refcounts 1/1, a mutex) at +150h/+158h, which startExternalNetworkWorld moves and releases (0x8b284c0: `puStack_58[0x2a]`, `[0x2b]` moved, the control block released at the end); the local world's own join (0x13ca580) builds the same object. On 26.63 the function is ClientInstance.vtable slot 327 (`this` the object); on 70.26 it is slot 2 of the vtable at 0x13146670 (a base subobject: the function reads the MinecraftGame at this+130h, which is ClientInstance+1D0h, so the base is at +0A0h; the ClientInstance constructor 0x8b0c810 stores that vtable at +0A0h: `lea rax,[rip+..]; mov [r12+0A0h],rax`, 0x8b0c95b). The loader calls it as the game's virtual call does: through the subobject of the client whose vtable (one of the vptrs the constructor stores, a table in the 2000h bytes after ClientInstance.vtable) holds this function. Recipe 0 is that startExternalNetworkWorld call site (recipe 1 of ClientInstance.startExternalNetworkWorld), then function_start_of. Recipe 1 is the constructor call set-up `mov [rsp+28h],r14; mov [rsp+20h],ebx; lea rcx,[rbp+..]; mov dx,imm; mov r8d,imm; mov r9,rdi; call`, ended at the call, then function_start_of. The two recipes are in the same function and share no bytes, so by the recipe rules they are one fact; the independent evidence is the check in_stored_vtable: the function is held by exactly one vtable, and the ClientInstance constructors store that table as a vptr at exactly one offset (26.63: ClientInstance.vtable, stored at +0 by 0x6588150 (`mov [r13],rax`, 0x6588247) and 0x65f5300 (`mov [rcx],rax`, 0x65f532d); 70.26: the table 0x13146670, stored at +0A0h by 0x8b0c810 (0x8b0c962) and 0x8b5bd10), a fact of two other functions. No call-site recipe can give the address: the game calls it only virtually, through slot 327 of the client on 26.63 (_handleMultiplayerServiceJoinResult, `mov rax,[rax+0A38h]`, 0x8c960b) and slot 2 of the +0A0h base on 70.26 (0x1332bb4). Every immediate, displacement and branch byte is a wildcard. Both patterns are unique on 26.63 (6aed1906-b127418e74d2f2f9: 0x65a0530) and on 70.26 (6afa4e47-dadc375797a3d31a: 0x8b29860). Not hooked."

  recipe {
    step "pattern" {
      bytes = "48 8B 06 48 8B 80 ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 8D ?? ?? ?? ?? 48 89 F1 FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "4C 89 74 24 ?? 89 5C 24 ?? 48 8D 8D ?? ?? ?? ?? 66 BA ?? ?? 41 B8 ?? ?? ?? ?? 49 89 F9 E8"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_stored_vtable" {}
}


entry "ClientInstance.connectByAddressThis" {
  kind  = "field"
  notes = "The offset in a ClientInstance of the base subobject that ClientInstance.connectByAddress is called through (its `this`): 0 on 26.63, where the function is in ClientInstance.vtable itself (slot 327; the constructor stores that table with `mov [reg],rax`); 0A0h on 70.26, where it is slot 2 of the table 0x13146670, which both ClientInstance constructors store as a vptr at +0A0h (0x8b0c810: `lea rax,[rip+..]; mov [r12+0A0h],rax` at 0x8b0c95b; 0x8b5bd10: the lea at 0x8b5bd80). The game's own call does the same: on 70.26 the function reads the MinecraftGame at this+130h, the member that ClientInstance.startExternalNetworkWorld reads at client+1D0h (1D0h - 130h = 0A0h), and _handleMultiplayerServiceJoinResult calls it through a pointer to that base (`mov rcx,[rdi+38h]; ...; mov rax,[rcx]; mov rax,[rax+10h]`, 0x1332ba3..0x1332bb4); on 26.63 the same call goes through the client (`mov rax,[rax+0A38h]`, 0x8c960b, slot 327). Recipe: ref ClientInstance.connectByAddress, then vptr_offset: the walk back from the one data pointer to the function stops at the first lea target, and every vptr store of that table must use one displacement. It refuses when the table is stored at two offsets, so a wrong base cannot be named. One recipe with a range check: the offset is a fact of the constructors' vptr stores, which no second, independent function holds on both builds (the call sites name a slot in two different tables). Range 0..200h (inside the first 200h bytes of the object, where the constructors store their vptrs)."

  recipe {
    step "ref" { entry = "ClientInstance.connectByAddress" }
    step "vptr_offset" {}
  }

  check "range" {
    min = 0
    max = 512
  }
}
