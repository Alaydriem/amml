entry "LocalPlayer.applyTurnDelta" {
  kind      = "function"
  hooked    = true
  msvc_name = "?_applyTurnDelta@LocalPlayer@@QEAAXAEBVVec2@@@Z"
  notes     = "void LocalPlayer::_applyTurnDelta(Vec2 const& turnOffset), not virtual. Found from mouse look down: the source-location record of InGamePlayScreen::render gives the render function, its pointer in .rdata is slot 19 of the InGamePlayScreen vtable, slot 15 of that vtable (applyInput) makes one direct call to localPlayerTurn (after getLocalPlayer), and the last direct call of localPlayerTurn (`lea rdx,[rsp+..]; mov rcx,rsi; call`) is this function, its only direct caller. It reads x and y of the Vec2 (pointer in rdi in 26.51, r12 in 26.60) and adds them to the rotation through the camera systems of the client (`[this+0D70h]`, 0D98h in 26.60). Recipe 0 anchors after the prologue, which differs between the builds: `mov rsi,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]` (the first camera-system call) and the shared_ptr copy after it (`mov rax,[rbp+..]; test rax,rax; je; cmp byte ptr [rax],imm; jne; ... mov rcx,[rax+..]; test rcx,rcx; je; lock inc; mov r14,[rax+..]; jmp; xor r14d,r14d; mov rbx,[rax+..]`), then `function_start_of`. The prefix `mov rsi,rcx; mov rcx,[rcx+member]` keeps it apart from a function with the same shared_ptr copy. Recipe 1 is the call at the end of localPlayerTurn, in two steps: an anchor in localPlayerTurn that is the same in both builds (`ucomiss xmm0,xmm6; jbe; xorps xmm1,xmm1; ucomiss xmm0,xmm1; jne; jnp; divss xmm6,xmm0; movaps xmmN,xmm6`, the division of the turn scale), then a `pattern` in that function for `lea rdx,[rsp+..]; mov rcx,rsi; call`, read as a branch target. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). The two share no bytes, are unique in both exes and agree; `function_start` confirms the start. Plan 5b replaced the two per-build prologue recipes by recipe 0. What changed in 26.60 (a real code change): other saved registers, one xmm save less, the Vec2 pointer in r12, and localPlayerTurn scales with one `mulps`."

  recipe {
    step "pattern" {
      bytes = "48 89 CE 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? FF 15 ?? ?? ?? ?? 48 8B 45 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 80 38 ?? 0F 85 ?? ?? ?? ?? 48 8B 45 ?? 0F 57 C0 0F 29 45 ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ?? 4C 8B 70 ?? EB ?? 45 31 F6 48 8B 58 ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "0F 2E C6 76 ?? 0F 57 C9 0F 2E C1 75 ?? 7B ?? F3 0F 5E F0 44 0F 28"
    }
    step "pattern" {
      bytes  = "48 8D 54 24 ?? 48 89 F1 E8"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LocalPlayer.mClientLocatorBar" {
  kind     = "field"
  notes    = "The offset of LocalPlayer::mClientLocatorBar (ClientLocatorBar, 48h bytes: 8 bytes, then an unordered_map<WaypointHandle, unique_ptr<ClientWaypoint>> at +8 whose list head pointer is at +10h): EF0h on 26.60, EC8h on 26.51. Recipe 0 is ClientNetworkHandler::handle(LocatorBarPacket) (slot 38 on 26.60, 32 on 26.51: 0x6ba1ec0 and 0x130af50), which gets the local player and hands `player+mClientLocatorBar` and the packet to the ClientLocatorBar packet function (`test rax,rax; je; add rax,imm32; mov rcx,rax; mov rdx,rsi; call`); it reads the immediate of the add. Recipe 1 is HudLocatorBarRenderer.render, which walks the waypoint list from its head (`xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov qword ptr [rbp+..],imm; mov r13,[r13+mClientLocatorBar+10h]; mov rX,[r13+..]; cmp rX,r13; je`, with the ModRM bytes of the list registers wildcarded: r12 on 26.60, r14 on 26.51); it reads the displacement of `mov r13,[r13+..]` and subtracts 10h (the map at +8, its list head pointer at +8 in the map). The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "48 85 C0 74 ?? 48 05 ?? ?? ?? ?? 48 89 C1 48 89 F2 E8"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4D 8B AD ?? ?? ?? ?? 4D 8B ?? ?? 4D 39 ?? 74"
      offset = 21
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -16 }
  }

  check "range" {
    min = 3584
    max = 3968
  }
}
