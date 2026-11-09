entry "GameControllerHandler_GameCore.refresh" {
  kind      = "function"
  hooked    = true
  msvc_name = "?refresh@GameControllerHandler_GameCore@@UEAAXXZ"
  notes     = "void GameControllerHandler_GameCore::refresh(), the GameControllerHandler::refresh() override: slot 2 of the GameControllerHandler_GameCore.vtable global in 26.51, 26.60 and 26.62 (destructor, refresh(bool), refresh(), shutdown, checkPlatformSpecificControllerError, normalizeAxis, normalizeAxes; MSVC puts the two refresh overloads in reverse order, and slots 1 and 3 hold the same empty base stub). It is the game's only gamepad poll: for each player input it calls mGameInput->GetCurrentReading(GameInputKindGamepad = 40000h, device, &reading) (IGameInput v2 slot 4, mGameInput at this+128h), then reading->GetGamepadState, compares the buttons, triggers and sticks with the last state and pushes GameControllerEvent records into the 600-entry queues of each GameController. It reads no other GameInput kind. Found from the class layout: the inlined make_shared in the app setup function stores the vtable, clears mGameInput at +128h and creates the task queue at +2E0h, which matches the GameControllerHandler_GameCore layout; the vtable slot 2 body is the poll. Recipe 0 is the GetCurrentReading call (`mov rax,[rcx]; mov rax,[rax+..]; mov edx,40000h; lea r9,[rbp+..]; call [guard]`), then `function_start_of`. The slot displacement and the frame displacement are wildcards. The four bytes of the kind immediate are kept: with them wildcarded the pattern matches 3 places in every build, and the kind is the GameInput ABI constant for a gamepad read. Unique in 26.51, 26.60 and 26.62. The checks are `function_start` and `in_vtable` of GameControllerHandler_GameCore.vtable, slots 0 to 6 (the 7 virtuals of GameControllerHandler). Values: 0x8a880 in 26.51, 0x8a450 in 26.60, 26.62 and 26.63, 0x8bf780 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed in 6ae4eeee-4dd4fa7acacb4c41: the poll body grew and its frame is larger, so the out-pointer `lea r9,[rbp+..]` uses a disp32 (`4C 8D 8D`) instead of a disp8 (`4C 8D 4D`); recipe 0 is now a `pattern_any` with one alternative per encoding (alternative 0 matches once on 26.60, 26.62 and 26.63 and not on the Preview; alternative 1 matches once on the Preview and not on the others). Evidence on the Preview, both from the 26.63 body: (1) the GetCurrentReading gamepad call of the old body (`mov r8,[rsi]; mov rax,[rcx]; mov rax,[rax+20h]; mov edx,40000h; lea r9,..; call [guard]`, then `mov reg,eax`, the reading swap and the virtual call at +90h of the reading) is found once on the Preview, in this function; (2) the vtable neighbours: GameControllerHandler_GameCore.vtable (from its destructor, recipe 0 of that entry) holds the function in slot 2 on both builds, with the destructor in slot 0 and the same empty base stub in slots 1 and 3, as on 26.63. Would have survived with a better recipe. A detour that does not call the original drops the gamepad input of that poll; the last state stays stored, so the first poll after the block reports only the changes since the last poll that ran."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 40 ?? BA 00 00 04 00 4C 8D 4D ?? FF 15",
        "48 8B 01 48 8B 40 ?? BA 00 00 04 00 4C 8D 8D ?? ?? ?? ?? FF 15",
      ]
      literal_ok = [
        "0.8: GameInputKindGamepad (40000h), byte 0: the same call shape with other kinds matches 2 more places on 26.51, 26.60 and 26.62",
        "0.9: GameInputKindGamepad (40000h), byte 1: as byte 0",
        "0.10: GameInputKindGamepad (40000h), byte 2: as byte 0",
        "0.11: GameInputKindGamepad (40000h), byte 3: as byte 0",
        "1.8: GameInputKindGamepad (40000h), byte 0: as alternative 0, in the disp32 frame shape of 6ae4eeee-4dd4fa7acacb4c41",
        "1.9: GameInputKindGamepad (40000h), byte 1: as byte 0",
        "1.10: GameInputKindGamepad (40000h), byte 2: as byte 0",
        "1.11: GameInputKindGamepad (40000h), byte 3: as byte 0",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "GameControllerHandler_GameCore.vtable"
    min_slot = 0
    max_slot = 6
  }
}

entry "GameControllerHandler_GameCore.vtable" {
  kind  = "global"
  notes = "The vtable at this+0 of GameControllerHandler_GameCore (the GDK gamepad handler, a GameControllerHandler). The exe has no RTTI for it. Recipe 0 is the scalar deleting destructor: `mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rcx,[rcx+..] (mGameInput); test; je; mov rdx,[rsi+..] (the device callback token); test; je; mov rax,[rcx]; mov rax,[rax+..]; call [guard]` (IGameInput::UnregisterCallback); offset 5 gives the `lea`. Recipe 1 is the inlined make_shared in the app setup function: right after the base constructor call, `lea rax,[rip+..]; mov reg,[rbp+..]; mov [reg+..],rax; movups [reg+..],xmm; movups [reg+..],xmm; mov qword ptr [reg+..],0`, with every REX, ModRM, displacement and immediate byte wildcarded. The two recipes share no bytes and no function. In both, every displacement, immediate and branch byte is a wildcard; both are unique in 26.51, 26.60 and 26.62, and `operand index 1 rip_target` on the `lea` gives the table. The section check keeps the address inside .rdata. It exists for the `in_vtable` check of GameControllerHandler_GameCore.refresh. Values: 0xe6a6380 in 26.51, 0xefc6a60 in 26.60, 0xefc7a60 in 26.62 and 26.63, 0x12c5be30 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed in 6ae4eeee-4dd4fa7acacb4c41: recipe 0 still matches; in the make_shared of recipe 1 the object is held in r15 (no SIB byte), the members after the vtable changed (one `movups`, then `mov qword ptr [r15+..],0; mov [r15+..],r14; mov word ptr [r15+..],bx`), so recipe 1 is now a `pattern_any`: alternative 0 the old shape (26.51 to 26.63), alternative 1 the Preview shape (`lea rax,[rip+..]; mov r15,[rbp+..]; mov [r15+..],rax; movups; mov qword ptr; mov [..],r14; mov word ptr [..],bx`, every REX, ModRM, displacement and immediate byte wildcarded), the `lea` at offset 0 in both. Alternative 1 matches once on the Preview and on no other build; it is in the app setup function, the only function that loads `GameCore desktop requires a TextInputBackend`. Both recipes give the same table on the Preview. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 89 ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 96 ?? ?? ?? ?? 48 85 D2 74 ?? 48 8B 01 48 8B 40 ?? FF 15"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 05 ?? ?? ?? ?? ?? 8B ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ??",
        "48 8D 05 ?? ?? ?? ?? ?? 8B ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? 66 ?? 89 ?? ?? ?? ?? ??",
      ]
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
