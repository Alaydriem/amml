entry "MinecraftGame.grabMouse" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$grabMouse@MinecraftGame@@QEAAXXZ"
  notes     = "void MinecraftGame::grabMouse(), the IMinecraftGame::grabMouse override: slot 135 of the MinecraftGame.vtable global in both builds. Found through ClientInstance: ClientInstance::grabMouse and releaseMouse (IClientInstance slots 310 and 311 in 26.51, 306 and 307 in 26.60) are `if (isPrimaryClient()) tail-call minecraftGame->X()` with vtable offsets 438h and 440h. The function returns at once when the grabbed flag (1E8h in 26.51, 1F8h in 26.60) is set, and otherwise ends by setting it to 1. The recipe anchors inside the function, on the flag test and the first virtual call on the subobject at this+30h: `cmp byte ptr [rcx+member],imm; jne rel32; mov rsi,rcx; add rcx,imm; mov rax,[rsi+disp]; mov rax,[rax+disp]; lea rdx,[rbp+..]`, then `function_start_of`. This replaces the two per-build prologue recipes of the 26.60 repair (26.51 pushes r14, 26.60 does not): the anchor comes after the pushes, so it is the same in both builds. Every immediate, displacement and branch byte of the pattern is a wildcard except one kept byte: the 00 of `cmp byte ptr [rcx+flag],0`, because releaseMouse, one slot later, tests the same flag against 1; without the `cmp; jne` pair the rest of the anchor has 8 matches. It is unique in both exes. The checks are `function_start` and `in_vtable` of MinecraftGame.vtable, slots 131 to 135: the window ends at grabMouse (135) so that it leaves out releaseMouse (136) on both builds; the check replaces the old vtable route with a fixed `vtable_slot 135`. `function_start` cannot fail after `function_start_of`, so the window and the kept byte are the facts besides the anchor. What changed in 26.60 (a real code change): r14 is no longer saved, the frame is smaller, the asserts are gone and the flag moved."

  recipe {
    step "pattern" {
      bytes      = "80 B9 ?? ?? ?? ?? 00 0F 85 ?? ?? ?? ?? 48 89 CE 48 83 C1 ?? 48 8B 46 ?? 48 8B 40 ?? 48 8D 55"
      literal_ok = [
        "6: cmp byte ptr [rcx+flag],0: grabMouse runs when the grabbed flag is 0; releaseMouse, one slot later, tests the same flag against 1, so this byte tells the two apart",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "MinecraftGame.vtable"
    min_slot = 131
    max_slot = 135
  }
}

entry "MinecraftGame.vtable" {
  kind     = "global"
  notes    = "The vtable that the MinecraftGame constructor stores at this+0: the IMinecraftGame part of MinecraftGame, and so the vtable that ClientInstance calls through its IMinecraftGame* (at [this+1A8h] in 26.51, [this+1C8h] in 26.60). Slot 99 is getMouseGrabbed, 135 grabMouse, 136 releaseMouse. The recipe has two steps. Step 1 is an anchor in the MinecraftGame constructor: the allocation call of a shared control block (`call [rip+..]; test rax,rax; je; movabs r12,100000001h`), the set-up of the block (`mov [rax+8],r12; lea r14,[rip+..]; mov [rax],r14; mov rcx,rax; add rcx,10h; mov byte ptr [rax+10h],1; mov [rbx+8],rcx; mov [rbx+10h],rax; mov qword ptr [rbx+..],0`) and the first three stores of the first vtable run. Every REX, ModRM, displacement and immediate byte is wildcarded except three kept bytes: the 1 of the flag store and the store displacements 18h and 20h (unchanged from 26.51 to 26.60). Another function has the same control-block set-up, but there a `mov rdi,rax` sits between the call and the test, so the call prefix keeps the anchor unique on both builds even with only the first store. Step 2 scans that function for a run of 13 `lea rax,[rip+..]; mov [reg+..],rax` stores, the first to [reg] and the next eleven with an 8-bit displacement; the constructor's first run is shorter and the destructor is another function, so only the final MinecraftGame run matches. The operand is the RIP target of the first `lea`, the store to this+0. The section check keeps the address inside .rdata. What changed in 26.60: MinecraftGame lost one 8-byte base subobject and the store run uses rdx instead of rcx. Values: 0xe76e2d0 in 26.51, 0xf091b70 in 26.60."

  recipe {
    step "pattern" {
      bytes      = "FF 15 ?? ?? ?? ?? ?? 85 ?? 0F 84 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 89 ?? ?? 83 ?? ?? C6 ?? ?? 01 ?? 89 ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 18 ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 20"
      literal_ok = [
        "49: kept for margin (Plan 5b review): the control block flag is set to 1 (`mov byte ptr [reg+10h],1`); unchanged from 26.51 to 26.60",
        "86: kept for margin (Plan 5b review): the second vtable store goes to this+18h; unchanged from 26.51 to 26.60",
        "97: kept for margin (Plan 5b review): the third vtable store goes to this+20h; unchanged from 26.51 to 26.60",
      ]
    }
    step "pattern" {
      bytes = "48 8D 05 ?? ?? ?? ?? 48 89 ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89"
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

entry "MinecraftGame.onAppFocusLost" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onAppFocusLost@MinecraftGame@@QEAAXXZ"
  notes     = "void MinecraftGame::onAppFocusLost(), the AppPlatformListener::onAppFocusLost override: slot 6 of the MinecraftGame.AppPlatformListener.vtable global in both builds (destructor, onAppPaused, onAppUnpaused, onAppPreSuspended, onAppSuspended, onAppResumed, onAppFocusLost). Its body starts by moving `this` back to the MinecraftGame object (`lea rsi,[rcx-90h]` in 26.51, `-88h` in 26.60). No direct call reaches it. Recipe 0 is the prologue (six pushes, frame setup, the this-adjust, the first virtual call and its bool test). Recipe 1 shares no bytes with recipe 0: the null-checked virtual call through a member (`mov rcx,[rdi+member]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; lea rax,[rip+..]`), 0x48 bytes in, then `function_start_of`. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. The checks are `function_start` and `in_vtable` of MinecraftGame.AppPlatformListener.vtable, slots 2 to 10; the check replaces the old vtable route with a fixed `vtable_slot 6`, which failed on 26.60 (the constructor's store run changed). What changed in 26.60: the AppPlatformListener subobject moved from +90h to +88h."

  recipe {
    step "pattern" {
      bytes = "55 41 57 41 56 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CF 48 8D B1 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 84 C0 0F 84"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 8F ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8D 05"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "MinecraftGame.AppPlatformListener.vtable"
    min_slot = 2
    max_slot = 10
  }
}

entry "MinecraftGame.AppPlatformListener.vtable" {
  kind     = "global"
  notes    = "The AppPlatformListener vtable of MinecraftGame, stored by the constructor at this+90h in 26.51 and this+88h in 26.60. Found through the vtable: string_xref on 'virtual MinecraftGame::onAppSuspended' gives one function, whose only pointer in .rdata is slot 4 of this vtable. The recipe has two steps. Step 1 is the same constructor anchor as MinecraftGame.vtable (the allocation call, the control block and three stores, with three kept bytes). Step 2 scans the constructor for the place in the final store run where the 8-bit-displacement stores end: `mov [reg+disp8],rax; lea rax,[rip+..]; mov [reg+disp32],rax; lea rax,[rip+..]; mov [reg+disp32],rax; lea rax,[rip+..]` (the stores to +78h then +90h in 26.51, +70h then +88h in 26.60); offset 4 is the `lea` of the first disp32 store, this vtable. It relies on AppPlatformListener being the first base at or beyond +80h in both builds. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). It matches once in the constructor of each exe. The section check keeps the address inside .rdata. It exists for the `in_vtable` check of onAppFocusLost; it was added in Plan 5b. Values: 0xe76ec60 in 26.51, 0xf0924e0 in 26.60."

  recipe {
    step "pattern" {
      bytes      = "FF 15 ?? ?? ?? ?? ?? 85 ?? 0F 84 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 89 ?? ?? 83 ?? ?? C6 ?? ?? 01 ?? 89 ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 18 ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 20"
      literal_ok = [
        "49: kept for margin (Plan 5b review): the control block flag is set to 1 (`mov byte ptr [reg+10h],1`); unchanged from 26.51 to 26.60",
        "86: kept for margin (Plan 5b review): the second vtable store goes to this+18h; unchanged from 26.51 to 26.60",
        "97: kept for margin (Plan 5b review): the third vtable store goes to this+20h; unchanged from 26.51 to 26.60",
      ]
    }
    step "pattern" {
      bytes  = "48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05"
      offset = 4
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

entry "MinecraftGame.appPlatformListenerOffset" {
  kind  = "field"
  notes = "Offset of the AppPlatformListener subobject in MinecraftGame: 88h on 6aed1906-b127418e74d2f2f9 (26.63), 90h on 6afa4e47-dadc375797a3d31a (1.26.70.26); the older recipe notes give 90h on 26.51 and 88h on 26.60. It is the displacement of the store that puts the MinecraftGame.AppPlatformListener.vtable pointer into the object: the `this` that an AppPlatformListener override (MinecraftGame.onAppFocusLost, the onAppFocusGained slot) receives in rcx is MinecraftGame + this value, and a caller that has the MinecraftGame passes MinecraftGame + this value to call an AppPlatformListener slot. Recipe 0 is the MinecraftGame constructor: step 1 is the constructor anchor of MinecraftGame.vtable and MinecraftGame.AppPlatformListener.vtable (the allocation call, the control block and three stores, with the same three kept bytes and reasons), step 2 is the place in the final store run where the 8-bit-displacement stores end, the pattern of MinecraftGame.AppPlatformListener.vtable step 2, with offset 11: the `mov [reg+disp32],rax` right after the `lea` of the listener vtable, read as the displacement of operand 0. Recipe 1 shares no bytes with recipe 0 and runs in another function, the MinecraftGame destructor, which stores the same vtables again in the same order: step 1 is its head (`mov qword ptr [rbp+..],imm32; mov reg,rcx`, then the run `lea rax,[rip+..]; mov [reg],rax` and eight `lea rax,[rip+..]; mov [reg+disp8],rax`, up to the next `lea`), step 2 the same end-of-disp8-run pattern inside the destructor (once in it on both builds), offset 11, the displacement of operand 0. Every ModRM, displacement and immediate byte of the new patterns is a wildcard (the REX and opcode bytes of `mov`, `lea` and `mov qword ptr` stay). Both recipes rely on AppPlatformListener being the first base stored with a 32-bit displacement in the run, as MinecraftGame.AppPlatformListener.vtable does. Because of that shared assumption the two recipes are one fact about the store run: a build that stores another base or member with a 32-bit displacement before the listener, or the listener with an 8-bit one, gives the same wrong value from both and the range check does not catch it. A consumer must check that the qword at MinecraftGame + this value is MinecraftGame.AppPlatformListener.vtable before it calls a slot through it. A recipe of another shape is not possible with the current steps: the onAppFocusLost and onAppFocusGained adjusts (`lea rsi,[rcx-88h]` / `-90h`) are negative displacements that no step negates (`operand displacement` and `member_displacement` refuse a negative value), and the positive uses in the destructor (`lea rcx,[rsi+88h]; call` to the AppPlatformListener base destructor) have no shape that is unique on both builds without running past a call. Check: range 8..512. No build gate: the member means the same on every build, only its offset moves. Evidence, both from the 26.63 functions: (1) on 26.63 the `lea` of the first disp32 store of the constructor run loads MinecraftGame.AppPlatformListener.vtable (the vtable that holds MinecraftGame.onAppFocusLost in slot 6) and the store goes to +88h; on 70.26 the constructor and the destructor (the two functions that load that vtable, `inspect xrefs` of the resolved global) store it to +90h; (2) MinecraftGame.onAppFocusLost moves `this` back by the same amount on each build: `lea rsi,[rcx-88h]` on 26.63, `lea rsi,[rcx-90h]` on 70.26, and its first virtual call reads the MinecraftGame vtable through [rcx-88h] / [rcx-90h]. Needed by the loader's pause-screen call (an AppPlatformListener slot called on the MinecraftGame). What changed in 6afa4e47-dadc375797a3d31a: the members before the subobject grew by 8 bytes (the disp8 store run ends at +78h instead of +70h), so the subobject moved from +88h to +90h. Class: new entry."

  recipe {
    step "pattern" {
      bytes      = "FF 15 ?? ?? ?? ?? ?? 85 ?? 0F 84 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 89 ?? ?? 83 ?? ?? C6 ?? ?? 01 ?? 89 ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 18 ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? 20"
      literal_ok = [
        "49: kept for margin (Plan 5b review): the control block flag is set to 1 (`mov byte ptr [reg+10h],1`); unchanged from 26.51 to 26.60",
        "86: kept for margin (Plan 5b review): the second vtable store goes to this+18h; unchanged from 26.51 to 26.60",
        "97: kept for margin (Plan 5b review): the third vtable store goes to this+20h; unchanged from 26.51 to 26.60",
      ]
    }
    step "pattern" {
      bytes  = "48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 C7 45 ?? ?? ?? ?? ?? 48 89 ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05"
    }
    step "pattern" {
      bytes  = "48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? ?? ?? ?? 48 8D 05"
      offset = 11
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

entry "MinecraftGame.update" {
  kind      = "vslot"
  msvc_name = "?$update@MinecraftGame@@QEAAXXZ"
  notes     = "Slot of void IMinecraftGame::update() in the MinecraftGame.vtable global: 2 in both builds (header order: the EnableNonOwnerReferences destructor 0, initialize 1, update 2, stop 3). `this` in rcx is the MinecraftGame; no other argument is read. The function runs one whole frame: it calls MinecraftGame::_update, which calls _clientUpdate (and through it ClientInstance.update), onTick, updateGraphics and endFrame. A hook on it runs before and after the whole frame. Found through its profiler strings: it is the only function that loads \"Main app outer update\" (and \"Main app yield to OS\") in both builds. Recipe 0 is `string_xref` of \"Main app outer update\", then `slot_of` MinecraftGame.vtable. Recipe 1 shares no bytes with it: the body after the first profiler scope, `lea r8,[rbp+..]; mov rdx,rsi; call; mov [rbp+..],al; cmp qword ptr [rdi+member],0; je rel32; call; call` (the two calls read the clock; member 1E60h in 26.51, 1EE8h in 26.60), then `function_start_of` and `slot_of`. Every immediate, displacement and branch byte of the pattern is a wildcard. Both recipes are unique in both exes and agree. The range check is the start of the vtable. Values: slot 2 in 26.51 (0x7d2d70) and 26.60 (0x879030). What changed in 26.60: only the member offset."

  recipe {
    step "string_xref" { text = "Main app outer update" }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  recipe {
    step "pattern" {
      bytes = "4C 8D 45 ?? 48 89 F2 E8 ?? ?? ?? ?? 88 45 ?? 48 83 BF ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  check "range" {
    min = 0
    max = 8
  }
}

entry "MinecraftGame.onAppFocusGained" {
  kind      = "vslot"
  msvc_name = "?$onAppFocusGained@MinecraftGame@@QEAAXXZ"
  notes     = "Slot of void AppPlatformListener::onAppFocusGained() in the MinecraftGame.AppPlatformListener.vtable global: 7 in both builds (destructor, onAppPaused, onAppUnpaused, onAppPreSuspended, onAppSuspended, onAppResumed, onAppFocusLost 6, onAppFocusGained 7). `this` in rcx is the AppPlatformListener subobject, not the MinecraftGame: the body moves it back (`lea rsi,[rdi-90h]` in 26.51, `-88h` in 26.60, the same adjust as MinecraftGame.onAppFocusLost). At its end it calls MinecraftGame slot 135 (grabMouse, vtable offset 438h). Recipe 0 is the head: `mov rdi,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test eax,eax; jg rel32; lea rsi,[rdi-adjust]; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]` (member 1CE0h in 26.51, 1D70h in 26.60), then `function_start_of` and `slot_of`. Recipe 1 shares no bytes with it: the tail, a virtual call on rdi, `and bl,al; cmp bl,imm; jne`, the grabMouse call through rsi and the call that destroys a local (`lea rcx,[rbp+..]; call`). 26.60 puts a `nop` after each indirect call, so recipe 1 is a `pattern_any` with one alternative for each form. Then `function_start_of` and `slot_of`. Every immediate, displacement and branch byte is a wildcard. Both recipes are unique in both exes and agree. The range check is a window around slot 7. Values: slot 7 in 26.51 (0x803d70) and 26.60 (0x8a3ba0). What changed in 26.60: the subobject adjust, the member offset, the frame size and the `nop` after indirect calls."

  recipe {
    step "pattern" {
      bytes = "48 89 CF 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 85 C0 0F 8F ?? ?? ?? ?? 48 8D B7 ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftGame.AppPlatformListener.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 F9 FF 15 ?? ?? ?? ?? 20 C3 80 FB ?? 75 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8D 4D ?? E8",
        "48 89 F9 FF 15 ?? ?? ?? ?? 90 20 C3 80 FB ?? 75 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 90 48 8D 4D ?? E8",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftGame.AppPlatformListener.vtable" }
  }

  check "range" {
    min = 3
    max = 11
  }
}

entry "MinecraftGame.isInGame" {
  kind     = "vslot"
  notes    = "Slot of bool IMinecraftGame::isInGame() const in the MinecraftGame.vtable global: 24 in both builds (the header order minus 2 at this point). The body is a 10-byte leaf with no .pdata entry: `movzx eax,byte ptr [rcx+528h]; and al,1; ret` (538h in 26.60). Slot 31 (setIsInGame) tests and writes the same bit. `this` in rcx is the MinecraftGame; the result is in al. That body shape has 12 matches in each exe, so the entry is the fixed slot with a body check. Recipe 0 finds the body by its neighbours: the leaf, six int3 and the head of the function after it (`push rsi; sub rsp,imm; mov rax,[rcx+member]`); offset 0 is the leaf start. Then `slot_of` MinecraftGame.vtable. It relies on this link order. Recipe 1 is the fixed slot: `vtable_slot 24` of MinecraftGame.vtable, `read_pointer`, then `slot_of`. It cannot carry the entry alone (the vtable-only rule). The two recipes must agree, so the entry resolves only when the body that recipe 0 finds sits in slot 24; a moved slot or a changed body makes them disagree, and the entry fails. Every immediate, displacement and branch byte of the pattern is a wildcard. The range check is a window around slot 24. Values: slot 24 in 26.51 (0x7f4620) and 26.60 (0x896430). What changed in 26.60: only the member offset. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the function before the leaf no longer ends with `call [guard]; nop; add rsp,imm; ret` (the old recipe 0 started 12 bytes before the leaf and failed), so recipe 0 now starts at the leaf and keeps the head of the next function as its only anchor; it matches once on 26.63 and on 70.26. Evidence from the old function: (1) the leaf plus the head of the function after it, drafted from the old leaf, matches once on 70.26 and that match sits in slot 24 of MinecraftGame.vtable; (2) slot 31 (setIsInGame) has the same prologue and reads and writes the same member (538h) on both builds, and the member read by the leaf is the same (538h), so the slot order around 24 is unchanged. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "0F B6 81 ?? ?? ?? ?? 24 ?? C3 CC CC CC CC CC CC 56 48 83 EC ?? 48 8B 81"
      offset = 0
    }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  recipe {
    step "ref" { entry = "MinecraftGame.vtable" }
    step "vtable_slot" { slot = 24 }
    step "read_pointer" {}
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  check "range" {
    min = 20
    max = 28
  }
}

entry "MinecraftGame.isPrimaryClientInstanceReady" {
  kind     = "vslot"
  notes    = "Slot of bool IMinecraftGame::isPrimaryClientInstanceReady() const in the MinecraftGame.vtable global: 52 in both builds (the header order minus 5 at this point; slot 53 is ensureAllClientsAreLeaving, which loads its own name string). The body is a leaf with no .pdata entry: it walks the std::map<SubClientId, shared_ptr<IClientInstance>> at [rcx+970h] (9D0h in 26.60), looks for key 0 (the primary client) and returns `found != end` with `setne al`. `this` in rcx is the MinecraftGame. Recipe 0 is the first bytes of the leaf (`mov rax,[rcx+member]; mov rdx,[rax+disp]; cmp byte ptr [rdx+disp],imm; mov rcx,rax; jne`), at offset 0, then `slot_of` MinecraftGame.vtable. Recipe 1 shares no bytes with it: the function before it in memory (a loop over the same client map that calls a client virtual with edx = 0), from `mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+disp]; xor edx,edx; call [guard]` to its last `jmp` and the two int3 after it; offset 88 is the first byte after that padding, the leaf. Then `slot_of`. It relies on this link order, which is the same in both builds. Every immediate, displacement and branch byte of the patterns is a wildcard. Both recipes are unique in both exes and agree. The range check is a window around slot 52. Values: slot 52 in 26.51 (0x81e360) and 26.60 (0x8bcf70). What changed in 26.60: only the map member offset."

  recipe {
    step "pattern" {
      bytes = "48 8B 81 ?? ?? ?? ?? 48 8B 50 ?? 80 7A ?? ?? 48 89 C1 75"
    }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 31 D2 FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 57 ?? 80 7A ?? ?? 74 ?? 0F 1F 40 ?? 48 8B 4F ?? 80 79 ?? ?? 75 ?? 48 89 FA 48 89 CF 48 3B 51 ?? 74 ?? 48 89 F9 EB ?? 0F 1F 44 00 ?? 48 89 D1 48 8B 12 80 7A ?? ?? 74 ?? EB ?? CC CC"
      offset = 88
    }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  check "range" {
    min = 48
    max = 56
  }
}

entry "MinecraftGame.mSimTimer" {
  kind     = "field"
  notes    = "Offset of `mSimTimer` in MinecraftGame: 1268h in 26.51, 12C8h in 26.60. The member is a std::unique_ptr<Timer> (8 bytes; the code loads the Timer* from it and passes it as `this`), and mRealTimer is the next member, 8 bytes later. Evidence: the MinecraftGame timer function (0x7d9e40 in 26.51, 0x87f740 in 26.60) advances [this+1268h] then [this+1270h] with Timer::advanceTime; the code that builds MinecraftArguments copies them in the header order mSimTimer, mRealTimer; a frame function reads mAlpha (+8) of [this+1268h] and passes it as the frame alpha. Recipe 0 is that frame function: `mov rcx,[rdi+member]; movss xmm2,[rcx+disp]; mov rcx,[rax]; mov r8,[rcx+disp]`; it reads the displacement of operand 1 of the first instruction. Recipe 1 is the advance pair in the timer function: `mov rcx,[reg+sim]; movss xmm6,[rip+..]; movaps xmm1,xmm6; call; mov rcx,[reg+real]; movaps xmm1,xmm6; call`. 26.60 passes a third argument (`xor r8d,r8d` before each call), so recipe 1 is a `pattern_any` with one alternative for each form; both start at the first `mov`, and it reads the displacement of operand 1 there. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. What changed in 26.60: the member moved by 60h (a layout change)."

  recipe {
    step "pattern" {
      bytes = "48 8B 8F ?? ?? ?? ?? F3 0F 10 51 ?? 48 8B 08 4C 8B 41 ??"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE E8",
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4096
    max = 6144
  }
}

entry "MinecraftGame.mRealTimer" {
  kind     = "field"
  notes    = "Offset of `mRealTimer` in MinecraftGame: 1270h in 26.51, 12D0h in 26.60. Like mSimTimer it is a std::unique_ptr<Timer>, 8 bytes after mSimTimer in both builds. The recipe is the advance pair in the MinecraftGame timer function (see MinecraftGame.mSimTimer recipe 1): `mov rcx,[reg+sim]; movss xmm6,[rip+..]; movaps xmm1,xmm6; call; mov rcx,[reg+real]; movaps xmm1,xmm6; call`, with one `pattern_any` alternative for each build (26.60 adds `xor r8d,r8d` before each call). The 26.51 alternative starts with three wildcards so that the second `mov` is at offset 26 in both; the recipe reads the displacement of operand 1 there. Every displacement, immediate and branch byte is a wildcard. The entry has one recipe and a check (the rule for a field). What changed in 26.60: the member moved by 60h (a layout change)."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? 48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE E8",
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8",
      ]
      offset = 26
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4096
    max = 6144
  }
}

entry "MinecraftGame.endFrame" {
  kind      = "function"
  hooked    = true
  msvc_name = "?endFrame@MinecraftGame@@QEAAXXZ"
  notes     = "Address of void MinecraftGame::endFrame(): 0x7e3550 in 26.51, 0x887880 in 26.60. `this` in rcx is the MinecraftGame; it returns void. It hands the drawn frame to Render Dragon (one Present for each call), so a count of its calls is the frame number of a video render. It has one direct caller, MinecraftGame::_update (0x7d31e0 / 0x8794a0), at 0x7d60ab / 0x87bd2f, after startFrame, _clientUpdate, onTick and updateGraphics. Recipe 0 is the start of the body, after the prologue: `cmp dword ptr [rdi+member],imm; je rel32; cmp qword ptr [rip+..],imm; je rel32; mov eax,[rip+..]; mov ecx,[rip+..]; mov rdx,gs:[58h]; mov rcx,[rdx+rcx*8]; cmp eax,[rcx+4]; mov [rbp+..],rdi` (a thread-local static guard), then `function_start_of`. Recipe 1 is the call in _update: `mov rcx,rdi; call; mov rcx,[rdi+member]; test rcx,rcx; je rel32; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]`. 26.60 has a `nop` after the call, so the recipe is a `pattern_any` with one alternative for each form; each alternative matches once on its own build and not on the other. It reads the branch target at offset 3. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard, except the eight bytes of `gs:[58h]` and `[rcx+4]` in recipe 0, which the ABI fixes (literal_ok). Both are unique in both exes and agree. Check: function_start. What changed in 26.60: the member offsets and the first checks before recipe 0 (26.51 reads two dwords, 26.60 one). What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the function kept its body, but the compiler now keeps `this` in rsi, not rdi, so the register byte of the first `cmp dword ptr [reg+member],imm` and the ModRM byte of the `mov [rbp+..],reg` after the guard are wildcards (these two bytes were the only fixed register bytes); and the caller _update no longer holds `this` in rdi: it reloads it from the frame, so the call is `call; nop; mov rax,[rbp+..]; mov rcx,[rax+member]; test rcx,rcx; je rel32; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]` and the pattern_any has a third alternative for that form, with three wildcards before the call so that the call stays at offset 3. Recipe 0 is unique on every build: the sibling functions of the same prologue (updateGraphics and others) have no `mov [rbp+..],reg` between the epoch compare and the branch. Evidence on 70.26 from the old function: (1) the recipe 0 body drafted from the old endFrame (movaps xmm saves, thread-local guard) matches once in the new exe, in the function that _update calls after the same pre-frame call chain; (2) the old call site in _update (call, nop, load of an object member, test, vcall through the CFG guard) matches once in the new exe in the same function with the new frame-based form, and its call target is the same function. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes      = "83 ?? ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 83 3D ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 58 00 00 00 48 8B 0C CA 3B 81 04 00 00 00 48 89 ??"
      literal_ok = ["44: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "45: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "46: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "47: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "54: [rcx+4] is the thread-local init epoch of the MSVC thread-safe static guard, fixed by the MSVC runtime", "55: [rcx+4] is the thread-local init epoch of the MSVC thread-safe static guard, fixed by the MSVC runtime", "56: [rcx+4] is the thread-local init epoch of the MSVC thread-safe static guard, fixed by the MSVC runtime", "57: [rcx+4] is the thread-local init epoch of the MSVC thread-safe static guard, fixed by the MSVC runtime"]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 F9 E8 ?? ?? ?? ?? 48 8B 8F ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15",
        "48 89 F9 E8 ?? ?? ?? ?? 90 48 8B 8F ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15",
        "?? ?? ?? E8 ?? ?? ?? ?? 90 48 8B 85 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? FF 15",
      ]
      offset = 3
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "MinecraftGame.joinMultiplayerWithAddress" {
  kind      = "vslot"
  notes     = "Slot of virtual void MinecraftGame::joinMultiplayerWithAddress(Social::GameConnectionInfo, bool isLocal, const std::string&, const std::string&, NetworkType, std::shared_ptr<bool>, PlayerJoinWorldContext) in the MinecraftGame.vtable global: 39 on 26.63 (0x8b1970). The Play > Servers join handler and the localhost join call it through `mov rax,[rax+138h]` and the CFG guard; both reach ClientInstance::onStartJoinGame, not ClientInstance::startExternalNetworkWorld. Arguments: rcx = MinecraftGame, rdx = pointer to the by-value GameConnectionInfo, r8b = isLocal, r9 = first std::string, then stack arguments at entry+28h (second std::string), +30h (NetworkType), +38h (pointer to a shared_ptr<bool> block) and +40h (pointer to the PlayerJoinWorldContext). Recipe 0 is the head after the frame set-up: `mov r14b,r8b; mov [rbp+..],rdx; mov r15,rcx; call; nop; test al,al; jne rel32`, then `function_start_of` and `slot_of` MinecraftGame.vtable. Recipe 1 shares no bytes with it: the function loads its own __FUNCSIG__ text, so `string_xref` finds it, then `slot_of`. Every displacement, immediate and branch byte of the pattern is a wildcard. The range check is a window around slot 39. Value slot 39 (0x8b1970) on 6aed1906-b127418e74d2f2f9 (26.63). Slot 39 on 1.26.70.26 (6afa4e47-dadc375797a3d31a) as well (function 0x1316340 there). What changed in 1.26.70.26: the frame set-up grew (`mov eax,imm; call chkstk; sub rsp,rax` replaces `sub rsp,imm`), the by-value argument r9 is saved to the frame (`mov [rbp+..],r9`) instead of `mov rbx,r9`, and the registers loaded after the first branch changed, so recipe 0 lost its first bytes (`mov rbx,r9`) and its tail (the `mov r13d` / `mov rdi` loads) and keeps the middle that did not change; the ModRM of `mov r14b,r8b` and of `mov r15,rcx` are wildcards. It matches once on 26.63 and on 70.26. Evidence from the old function: (1) the shortened body pattern from the old function matches once on 70.26, at the function whose address sits in slot 39 of MinecraftGame.vtable, the same slot as on 26.63; (2) recipe 1, the function's own __FUNCSIG__ string (the old function loads it, and exactly one function on 70.26 loads it), gives slot 39 in the same vtable. The two recipes share no bytes. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes = "45 88 ?? 48 89 95 ?? ?? ?? ?? 49 89 ?? E8 ?? ?? ?? ?? 90 84 C0 0F 85 ?? ?? ?? ??"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  recipe {
    step "string_xref" { text = "virtual void __cdecl MinecraftGame::joinMultiplayerWithAddress(Social::GameConnectionInfo, bool, const std::string &, const std::string &, NetworkType, std::shared_ptr<bool>, PlayerJoinWorldContext)" }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  check "range" {
    min = 24
    max = 60
  }
}

entry "MinecraftGame.getWorldSystem" {
  kind  = "vslot"
  notes = "Slot of virtual World::WorldSystem& IMinecraftGame::getWorldSystem() const in the MinecraftGame.vtable global: 152 (byte offset 4C0h) on 26.51, 26.60, 26.62 and 26.63 (the LeviLamina header order minus 6 at this point). `this` in rcx is the MinecraftGame; rax returns the World::WorldSystem*. The body is an 8-byte leaf with no .pdata entry, `mov rax,[rcx+member]; ret` (1538h on 26.51, 1598h on 26.60 to 26.63), and the next leaf, slot 153, is isWorldSystemReady: `cmp qword ptr [rcx+member],0; setne al; ret` on the same member, the header pair getWorldSystem / isWorldSystemReady. Recipe 0: a caller (0xa243c0 on 26.63, 0x978420 on 26.51) found by the chain `mov rdi,[rcx+..]; mov rax,[rdi]; mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`, offset 14, the displacement of operand 1, `displacement_to_slot`; the pattern starts at the first load of the chain and ends at the call. Recipe 1: a `pattern_any` at offset 0 then `slot_of` MinecraftGame.vtable. Alternative 0 is the two leaves (the getter, eight int3, isWorldSystemReady), which relies on the link order of the two leaves and matches on 26.51 to 26.63. Alternative 1 is the body of the getter when it is a function (70.26): the stack-cookie prologue, the null test of the member, the assert strings with their lengths, ending at the call of the assert function. Every immediate, displacement and branch byte is a wildcard (the immediate 0 of the `cmp` too), except the two lengths kept with literal_ok in alternative 1. The two recipes share no bytes, and agree on every build. The range check is a window around slot 152 (144 to 160), well inside the IMinecraftGame interface; a range check alone is weak, so the entry relies on its agreeing recipes. Values: slot 152 (0x98) on 6aa482fd-2658118eba0dbac5 (26.51, leaf 0x81d9c0), 6ad79ee4-17d51891a1119914 (26.60, 0x8bc6c0), 6aea437e-7eff13daabb430da (26.62, 0x8bc6a0) and 6aed1906-b127418e74d2f2f9 (26.63, 0x8bc5d0). What changed: only the member offset (1538h to 1598h on 26.60); the slot did not move. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): slot 152 again (the member is 15E0h); the accessor is no longer a leaf: it is a function with a stack cookie and an assert that the member is not null (it loads its own __FUNCSIG__ text), so the old recipe 2 does not match, and the join-result function is no longer found by its __FUNCSIG__ text (old recipe 0, removed because it cannot run on this build; the recipes must run on both builds). Recipe 0 is the old recipe 1 without its prologue (the caller changed its pushes) and recipe 1 gained alternative 1. Evidence from the old function: (1) the caller found from the old build (the chain with the displacement of slot 152 at 4C0h) matches once in the new exe, in a function whose call reads the same displacement; (2) isWorldSystemReady, the old getter pair partner (`cmp qword ptr [rcx+member],0; setne al; ret`, the leaf after the getter), sits at slot 153 on 70.26, reads the same member (15E0h) as the function in slot 152, and slot 157 follows it at the same distance in the code as on 26.63. The accessor in slot 152 loads the text `virtual World::WorldSystem &MinecraftGame::getWorldSystem() const` (inspect vtable --names). Class: real code change (the leaf became a function), repaired by a second code shape."
  recipe {
    step "pattern" {
      bytes  = "48 8B 79 ?? 48 8B 07 48 8B 48 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
      offset = 14
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
        "48 8B 81 ?? ?? ?? ?? C3 CC CC CC CC CC CC CC CC 48 83 B9 ?? ?? ?? ?? ?? 0F 95 C0 C3",
        "56 48 83 EC ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 81 ?? ?? ?? ?? 48 85 C0 75 ?? 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? 41 ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? 0C ?? ?? ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 8D 0D ?? ?? ?? ?? 48 8D 54 24 ?? 4C 8D 4C 24 ?? 41 B8 ?? ?? ?? ?? E8",
      ]
      literal_ok = [
        "1.52: length 41h of the function's own __FUNCSIG__ text (the accessor template of 46 siblings differs only in this length and the next one)",
        "1.94: length 0Ch of the member name text mWorldSystem in the same assert, the second byte that tells the accessor apart from its siblings",
      ]
    }
    step "slot_of" { vtable = "MinecraftGame.vtable" }
  }

  check "range" {
    min = 144
    max = 160
  }
}

entry "MinecraftGame.applyActiveResourcePacks" {
  kind   = "function"
  hooked = true
  notes  = "The MinecraftGame function that applies the active resource pack stack: `void (MinecraftGame*, ResourcePackManager&, bool changed)`. rcx the MinecraftGame, rdx the ResourcePackManager, r8b the bool (passed on as the whole register by a hook); it returns void. There is no symbol: the name says what its two callers do. Caller 1 is the MinecraftGame override of `ResourcePackListener::onActiveResourcePacksChanged(ResourcePackManager&)`, slot 1 of the ResourcePackListener vtable that the MinecraftGame constructor stores at this+B8h on 26.63 and this+C0h on 70.26 (the third store after the AppPlatformListener vtable at +88h / +90h, the five-slot table: destructor, onActiveResourcePacksChanged, onFullPackStackInvalid, onLanguageSubpacksChanged, onResourceManagerDestroyed): on 26.63 it is a full function (a profiler zone, then `add rdi,-0B8h; mov rcx,rdi; mov rdx,rsi; mov r8b,1; call`), on 70.26 a thunk without .pdata (`add rcx,-0C0h; mov r8b,1; jmp`). Caller 2 is the startup work-list step whose label is `initNotifyOnActiveResourcePacksChanged`: it asks the MinecraftGame for its ResourcePackManager through a virtual call and calls this function with r8b = 0. So it runs once at startup with false and once for every change of the active pack stack with true. The body clears per-pack state in the pack list of the ResourcePackManager (`[rdx+58h]`, a loop over 128h-byte (26.63) / 120h-byte (70.26) elements that zeroes two members of each), reloads the pack resources and queues the follow-up tasks. The loader hooks it to bind the mod textures again after a pack change. Recipe 0 is that loop (`mov rdx,[rax+disp32]; mov r8,[rdx]; mov qword ptr [r8+disp32],imm32; mov rdx,[rdx]; mov qword ptr [rdx+disp32],imm32; add rax,imm32; cmp rax,rcx; jne`), then `function_start_of`; every ModRM, displacement, immediate and branch byte is a wildcard, the REX and opcode bytes stay; it is unique on both builds. Recipe 1 is the call in the startup step: step 1 is the label test at the start of the step (`cmp qword ptr [reg+disp8],26h; jne; mov reg,[reg]; lea rdx,[rip+..]; mov r8d,imm32; call`), with one kept byte, the 26h of the compare, the length of the label `initNotifyOnActiveResourcePacksChanged`: the other steps of the work list compare labels of other lengths, and without it the shape matches the other steps; step 2 is the end of the same function (`mov rdx,rax; xor r8d,r8d; call`), once in it, offset 6, read as the branch target. The two recipes are in two functions and share no pattern bytes; they agree on both builds. Check: function_start. Values: 26.63 (6aed1906-b127418e74d2f2f9) and 70.26 (6afa4e47-dadc375797a3d31a) both derive it with both recipes. What changed in 6afa4e47-dadc375797a3d31a: the pack element shrank from 128h to 120h bytes and the listener override became a thunk; the body and the step kept their shape. Evidence, from the 26.63 functions: (1) the label string `initNotifyOnActiveResourcePacksChanged` is loaded by two functions on each build, the work-list builder and the step, and the step ends with the `xor r8d,r8d; call` of this function on both builds; (2) the ResourcePackListener vtable of each build (found from the constructor store run, the third store after the AppPlatformListener vtable) holds in slot 1 the override that calls (26.63) or jumps to (70.26) this function with r8b = 1. Class: new entry."

  recipe {
    step "pattern" {
      bytes = "48 8B ?? ?? ?? ?? ?? 4C 8B ?? 49 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 8B ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 05 ?? ?? ?? ?? 48 39 ?? 75 ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "48 83 ?? ?? 26 75 ?? 48 8B ?? 48 8D 15 ?? ?? ?? ?? 41 B8 ?? ?? ?? ?? E8"
      literal_ok = ["4: 26h is the length of the step label initNotifyOnActiveResourcePacksChanged; the other work-list steps compare labels of other lengths"]
    }
    step "pattern" {
      bytes  = "48 89 C2 45 31 C0 E8"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
