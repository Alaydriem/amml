entry "MouseDevice.feed" {
  kind      = "function"
  hooked    = true
  msvc_name = "?feed@MouseDevice@@QEAAXDCFFFF_N@Z"
  notes     = "void MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, bool forceMotionlessPointer), not virtual. LeviLamina raises MouseInputEvent from a hook on this function. No string names it. Found by its call sites: direct calls that pass three shorts and a byte on the stack, each `lea rcx,[global MouseDevice]` with constant `dl`/`r8b`. The body fits LeviLamina's MouseDevice layout (MouseAction of 0x14 bytes appended to the vector at this+18h, the -9999 (0D8F1h) marker on _dx/_dy, _firstMovementType at +30h). Recipe 0 is the prologue (eight pushes, `mov edi,r9d; mov ebx,r8d; mov ebp,edx; mov rsi,rcx`, then the loads of the three stack shorts and the stack byte). Recipe 1 shares no bytes with recipe 0: the -9999 test on _dx (`cmp word ptr [rsi+..],imm; jne; mov dword ptr [rsi+..],imm; mov eax,[rsi+..]; xor ecx,ecx; cmp dword ptr [rsi+..],imm; sete cl`), 0x106 bytes in on 26.51 and 0xFC on 26.60, then `function_start_of`. With the immediate wildcarded, the -9999 marker is no longer part of the pattern; the instruction shape alone is unique. Recipe 2 is a call site that passes y on the stack and false as the last argument, then `lea rcx,[global]; mov dl,imm; mov r8b,imm; call`, read as a branch target. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). All three are unique in both exes and agree; `function_start` confirms the start. What changed in 26.60: the local MouseAction moved 4 bytes down the frame."

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
}

entry "MouseDevice.instance" {
  kind  = "global"
  notes = "The game's one MouseDevice object, a static global (not a pointer): every direct caller of MouseDevice.feed that feeds the desktop mouse passes it as `this` with `lea rcx,[rip+..]`. Three callers in three functions use the same address: the GDK gamepad poll (GameControllerHandler_GameCore.refresh, its emulated cursor click: `mov [rsp+..],ax; mov byte ptr [rsp+..],imm; lea rcx,[rip+..]; mov dl,imm; mov r8b,imm; call`), the window handler of the mouse button messages (WM_LBUTTONDOWN to WM_XBUTTONDBLCLK through a jump table on msg-201h; `lea rcx,[rip+..]; mov edx,r14d (button); mov r8d,ebx (state); mov r9d,edi (x); call`), and the WM_INPUT raw mouse handler (GetRawInputData, then `feed(this,0,0,0,..)` with the motion delta: `lea rcx,[rip+..]; xor edx,edx; xor r8d,r8d; xor r9d,r9d; call`). One recipe per caller; each pattern ends at the call and offset gives the `lea`, whose RIP target is the operand. The window code also feeds a second address (0x140 bytes after this one on 26.63, the base of an array of 38h-byte per-pointer devices indexed by the WM_POINTER id, the touch and pen path); no recipe reads that one. Every immediate, displacement and branch byte is a wildcard. Check: section .data. Requested by the mod manager, so fake mouse buttons reach the game before the first real mouse event. Values: 0x128d0010 in 26.62 and 26.63."

  recipe {
    step "pattern" {
      bytes  = "66 89 44 24 ?? C6 44 24 ?? ?? 48 8D 0D ?? ?? ?? ?? B2 ?? 41 B0 ?? E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 44 89 F2 41 89 D8 41 89 F9 E8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "C6 44 24 ?? ?? 66 C7 44 24 ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 31 D2 45 31 C0 45 31 C9 E8"
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" { name = ".data" }
}
