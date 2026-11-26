entry "MouseDevice.feed" {
  kind      = "function"
  hooked    = true
  msvc_name = "?feed@MouseDevice@@QEAAXDCFFFF_N@Z"
  notes     = "void MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, bool forceMotionlessPointer), not virtual. LeviLamina raises MouseInputEvent from a hook on this function. No string names it. Found by its call sites: direct calls that pass three shorts and a byte on the stack, each `lea rcx,[global MouseDevice]` with constant `dl`/`r8b`. The body fits LeviLamina's MouseDevice layout (MouseAction of 0x14 bytes appended to the vector at this+18h, the -9999 (0D8F1h) marker on _dx/_dy, _firstMovementType at +30h). Recipe 0 is the prologue (eight pushes, `mov edi,r9d; mov ebx,r8d; mov ebp,edx; mov rsi,rcx`, then the loads of the three stack shorts and the stack byte). Recipe 1 shares no bytes with recipe 0: the -9999 test on _dx (`cmp word ptr [rsi+..],imm; jne; mov dword ptr [rsi+..],imm; mov eax,[rsi+..]; xor ecx,ecx; cmp dword ptr [rsi+..],imm; sete cl`), 0x106 bytes in on 26.51 and 0xFC on 26.60, then `function_start_of`. With the immediate wildcarded, the -9999 marker is no longer part of the pattern; the instruction shape alone is unique. Recipe 2 is a call site that passes y on the stack and false as the last argument, then `lea rcx,[global]; mov dl,imm; mov r8b,imm; call`, read as a branch target. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). All three are unique in both exes and agree; `function_start` confirms the start. What changed in 26.60: the local MouseAction moved 4 bytes down the frame. Gate: `entry_is LevelChunk.mSubChunks = 456` (the 26.6x layout), so the entry means the seven-argument form only. What changed in 6afa4e47-dadc375797a3d31a (70.26): real code change. The function takes an eighth argument, an int controller id on the stack before forceMotionlessPointer, and stores it in the MouseAction where 26.63 stores 0; a seven-argument hook would pass a wrong last argument. That form is the entry MouseDevice.feedWithControllerId, and this entry is not for the 70.x builds. The data files of 26.60, 26.62 and 6ae4eeee-4dd4fa7acacb4c41 keep the fingerprint of this entry before the gate; the loader derives the entry again on a cache miss there. On 6aa482fd-2658118eba0dbac5 (26.51, LevelChunk.mSubChunks 140h) the gate refuses the entry although the seven-argument function exists there; builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC ?? 44 89 CF 44 89 C3 89 D5 48 89 CE 44 0F B7 A4 24 ?? ?? ?? ?? 44 0F B7 AC 24 ?? ?? ?? ?? 44 0F B7 BC 24 ?? ?? ?? ?? 0F B6 84 24"
    }
  }

  recipe {
    step "pattern" {
      bytes = "66 81 7E ?? ?? ?? 75 ?? C7 46 ?? ?? ?? ?? ?? 8B 46 ?? 31 C9 83 7E ?? ?? 0F 94 C1"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "66 89 44 24 ?? C6 44 24 ?? ?? 48 8D 0D ?? ?? ?? ?? B2 ?? 41 B0 ?? E8"
      offset = 22
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

entry "MouseDevice.feedWithControllerId" {
  kind      = "function"
  hooked    = true
  notes     = "void MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, int controllerId, bool forceMotionlessPointer), not virtual: the 70.x form of MouseDevice.feed, with an eighth argument. The new argument is a 32-bit int on the stack at [rsp+38h] of the caller (+40h at entry), before forceMotionlessPointer (now [rsp+40h] of the caller, +48h at entry, the `+ stack 0x48` of `signature`). The body stores it in the MouseAction record at +0Ch (in both pushes, the plain one and the button 4 one), where the 26.63 body stores a constant 0 (`mov dword ptr [rsp+34h],0`). The name comes from the reader: the MouseDevice input device's event function (slot 5 of its vtable, the one that walks the MouseAction vector at MouseDevice+18h) copies MouseAction+0Ch into the input event at +30h and into a new int out-argument; on 26.63 it fills the same event field from a device member that its constructor sets to 0. The gamepad input device's event function on 70.26 fills the same event field and the same out-argument from +30h of the GameControllerEvent record, which GameControllerHandler_GameCore.refresh writes with the gamepad's controller id (the index it also uses for the per-controller GameController list). So the field is the controller id of the event, and the mouse now carries its own. All 14 direct callers on 70.26 store a dword 0 there. No msvc_name: the mangled name of the new form is not certain. Recipe 0 is the prologue: the eight pushes, `mov edi,r9d; mov ebx,r8d; mov ebp,edx; mov rsi,rcx`, then the dword load of the controller id (`mov r14d,[rsp+..]`, `44 8B B4 24`), the three word loads of the stack shorts and the stack byte. The dword load is what the 26.63 prologue does not have (it starts with the word loads), so the pattern does not match there. Recipe 1 shares no bytes with recipe 0: the -9999 test on _dx (`cmp word ptr [rsi+..],imm; jne; mov dword ptr [rsi+..],imm; mov eax,[rsi+..]; xor ecx,ecx; cmp dword ptr [rsi+..],imm; sete cl`), recipe 1 of MouseDevice.feed, then `function_start_of`. Recipe 1 is the same pattern as recipe 1 of MouseDevice.feed and does not tell the seven-argument and eight-argument forms apart (on 26.63 it finds the seven-argument function); the arity is carried by recipes 0 and 2 and the gate, so a build where only recipe 1 matches resolves this entry on one recipe and needs a check of the body. Recipe 2 is the call site in the raw input handler (WM_INPUT, `feed(0,0,0,..)` with the motion delta): the byte store of forceMotionlessPointer, the dword store of the controller id, the word store, then `lea rcx,[global]; xor edx,edx; xor r8d,r8d; xor r9d,r9d; call`, read as a branch target; the dword store is new in 70.x, the 26.63 site has none. Every immediate, displacement and branch byte of the patterns is a wildcard; each pattern ends at a call or before one. Checks: `function_start`, and `entry_is LevelChunk.mSubChunks = 552` (the 70.x layout); on 26.63 the gate refuses and MouseDevice.feed resolves instead. Evidence on 6afa4e47-dadc375797a3d31a, both from the 26.63 function: (1) the -9999 body pattern of the 26.63 MouseDevice.feed body matches once on 70.26, in this function, and the decompiled bodies are the same statement for statement except the new argument; (2) the 26.63 callers: the raw input site of the 26.63 MouseDevice.feed (`lea rcx,[MouseDevice.instance]; xor edx,edx; xor r8d,r8d; xor r9d,r9d; call`) is found on 70.26 in the same handler with one more stack store, and calls this function; the function has 14 direct callers on both builds, each passing MouseDevice.instance or the per-pointer device array after it. What changed in 6afa4e47-dadc375797a3d31a: real code change (new argument)."

  recipe {
    step "pattern" {
      bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC ?? 44 89 CF 44 89 C3 89 D5 48 89 CE 44 8B B4 24 ?? ?? ?? ?? 44 0F B7 A4 24 ?? ?? ?? ?? 44 0F B7 AC 24 ?? ?? ?? ?? 44 0F B7 84 24 ?? ?? ?? ?? 0F B6 84 24"
    }
  }

  recipe {
    step "pattern" {
      bytes = "66 81 7E ?? ?? ?? 75 ?? C7 46 ?? ?? ?? ?? ?? 8B 46 ?? 31 C9 83 7E ?? ?? 0F 94 C1"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "C6 44 24 ?? ?? C7 44 24 ?? ?? ?? ?? ?? 66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 31 D2 45 31 C0 45 31 C9 E8"
      offset = 35
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

entry "MouseDevice.instance" {
  kind  = "global"
  notes = "The game's one MouseDevice object, a static global (not a pointer): every direct caller of MouseDevice.feed that feeds the desktop mouse passes it as `this` with `lea rcx,[rip+..]`. Three callers in three functions use the same address: the GDK gamepad poll (GameControllerHandler_GameCore.refresh, its emulated cursor click: `mov [rsp+..],ax; mov byte ptr [rsp+..],imm; lea rcx,[rip+..]; mov dl,imm; mov r8b,imm; call`), the window handler of the mouse button messages (WM_LBUTTONDOWN to WM_XBUTTONDBLCLK through a jump table on msg-201h; `lea rcx,[rip+..]; mov edx,r14d (button); mov r8d,ebx (state); mov r9d,edi (x); call`), and the WM_INPUT raw mouse handler (GetRawInputData, then `feed(this,0,0,0,..)` with the motion delta: `lea rcx,[rip+..]; xor edx,edx; xor r8d,r8d; xor r9d,r9d; call`). One recipe per caller; each pattern ends at the call and offset gives the `lea`, whose RIP target is the operand. The window code also feeds a second address (0x140 bytes after this one on 26.63, the base of an array of 38h-byte per-pointer devices indexed by the WM_POINTER id, the touch and pen path); no recipe reads that one. Every immediate, displacement and branch byte is a wildcard. Check: section .data. Requested by the mod manager, so fake mouse buttons reach the game before the first real mouse event. Values: 0x128d0010 in 26.62 and 26.63. What changed in 6afa4e47-dadc375797a3d31a (70.26): the frame stores before the call differ in the gamepad site (an extra dword store, so recipe 0 now starts at the lea, which is unique with the two byte loads and the call), the register moves of the window site (recipe 1 has one alternative per register form) and the byte store of the raw input site became a dword store (recipe 2 has one alternative per shape, three wildcard bytes in front of the old shape keep offset 15). Would have survived with a better recipe. Evidence on 70.26: all three recipes, drafted from the three 26.63 callers of MouseDevice.feed, match once in three different functions, each is a caller of MouseDevice.feed on 70.26 (the callers of the entry list them), and they agree on one address."

  recipe {
    step "pattern" {
      bytes  = "48 8D 0D ?? ?? ?? ?? B2 ?? 41 B0 ?? E8"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 44 89 F2 41 89 D8 41 89 F9 E8",
        "66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 89 ?? 41 89 ?? 45 89 ?? E8",
      ]
      offset   = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? C6 44 24 ?? ?? 66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 31 D2 45 31 C0 45 31 C9 E8",
        "C7 44 24 ?? ?? ?? ?? ?? 66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 31 D2 45 31 C0 45 31 C9 E8",
      ]
      offset   = 15
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" { name = ".data" }
}
