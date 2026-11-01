entry "LevelRendererPlayer.setupCamera" {
  kind      = "function"
  hooked    = true
  msvc_name = "?setupCamera@LevelRendererPlayer@@QEAAXAEAVCamera@mce@@M@Z"
  notes     = "void LevelRendererPlayer::setupCamera(mce::Camera& camera, float), not virtual: this in rcx, the camera in rdx, the float in xmm2 (the body does not read it). Found from InGamePlayScreen::_renderLevelPrep (26.51: the function that loads the source-location record of 'virtual void __cdecl InGamePlayScreen::_renderLevelPrep(ScreenContext &, LevelRenderer &, Actor &)'), which calls this one with the LevelRendererPlayer of the LevelRenderer (`mov rcx,[levelRenderer+disp]`), the camera of the ScreenContext (`mov rdx,[screenContext+18h]`) and a float of the ScreenContext in xmm2. The body marks the view and projection matrix stacks of the camera dirty (`mov byte ptr [camera+38h],1` and `[camera+0B8h],1`) and takes their top matrices, reads the render-camera component of the local user (rotation quaternion at +38h), writes the view matrix from its rotation and position, writes the perspective matrix (tan of half the fov, -1 in element 11), stores the camera position and target in this+654h.. and the camera forward and up vectors (LevelRendererPlayer::mCameraForward and mCameraUp). It has three direct callers in each build. Recipe 0 is the start of the body after the register saves (`mov r14,rdx; mov rsi,rcx; mov byte ptr [rdx+..],imm; ...` the two deque top reads of the view and projection stacks, `mov rdx,[rsi+..]; lea rcx,[rsp+..]; call`), then function_start_of; it matches once on both builds. Recipe 1 is the call site in _renderLevelPrep, read as a branch target; a pattern_any with one alternative per build shape, both ending at the call so that one offset serves both: 26.51 `mov [rbp+..],al; mov rcx,[r15+..]; movss xmm2,[r14+8]; mov rdx,[r14+18h]; call; xorps xmm0,xmm0`; 26.60 `mov rcx,[r14+..]; movzx eax,dl; movss xmm2,[rbx+rax*4+disp]; mov rdx,[rbx+18h]; call; nop; xorps xmm0,xmm0` (the float is now picked from an array of the ScreenContext by a clamped index). Recipe 2 is the call site in a second caller, also a pattern_any ending at the call: 26.51 `cmp byte ptr [rax+..],imm; jne; mov rcx,[r15+..]; movss xmm2,[rsi+8]; mov rdx,[rsi+18h]; call`; 26.60 `cmp al,imm; cmovae eax,r12d; mov rcx,[r15+..]; movzx eax,al; movss xmm2,[rsi+rax*4+disp]; mov rdx,[rsi+18h]; call`. Every ModRM byte of the call sites, every displacement, immediate and branch byte is a wildcard; recipe 0 keeps its r14/rsi ModRM bytes. The three recipes are in three functions and share no bytes. The check is function_start. What changed in 6ad79ee4-17d51891a1119914: the callers changed (the LevelRendererPlayer member of LevelRenderer moved from 468h to 480h, the float argument is read from an indexed array instead of [screenContext+8], and a `nop` follows the call), so the two old call-site patterns matched nothing; the body did not change. Class: real code change in the callers. Evidence for the 26.60 value (cold value, it had been derived by recipe 0 alone): (1) the body pattern of recipe 0, drafted from the 26.51 function, matches once on 26.60, in this function; (2) the Ghidra decompile of the 26.51 function and of this one have the same shape: the dirty flags at camera+38h and +0B8h, the render-camera quaternion at +38h, the stores at this+654h.., the perspective and forward/up writes, in the same order; and its three callers pass the camera as [screenContext+18h] in rdx, as in 26.51."

  recipe {
    step "pattern" {
      bytes = "49 89 D6 48 89 CE C6 42 ?? ?? 48 8B 42 ?? 48 8B 4A ?? 48 01 C8 48 FF C8 48 8B 4A ?? 48 8B 52 ?? 48 FF CA 48 21 C2 48 8B 3C D1 41 C6 86 ?? ?? ?? ?? ?? 49 8B 86 ?? ?? ?? ?? 49 8B 8E ?? ?? ?? ?? 48 01 C8 48 FF C8 49 8B 8E ?? ?? ?? ?? 48 FF C9 48 21 C1 49 8B 86 ?? ?? ?? ?? 4C 8B 24 C8 48 8B 96 ?? ?? ?? ?? 48 8D 4C 24 ?? E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "88 85 ?? ?? ?? ?? 49 8B ?? ?? ?? ?? ?? F3 41 0F 10 ?? ?? 49 8B ?? ?? E8 ?? ?? ?? ?? 0F 57 C0",
        "49 8B ?? ?? ?? ?? ?? 0F B6 ?? F3 0F 10 ?? ?? ?? ?? ?? ?? 48 8B ?? ?? E8 ?? ?? ?? ?? 90 0F 57 C0",
      ]
      offset = 23
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "80 B8 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 49 8B ?? ?? ?? ?? ?? F3 0F 10 ?? ?? 48 8B ?? ?? E8",
        "3C ?? 41 0F 43 ?? 49 8B ?? ?? ?? ?? ?? 0F B6 ?? F3 0F 10 ?? ?? ?? ?? ?? ?? 48 8B ?? ?? E8",
      ]
      offset = 29
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LevelRendererPlayer.createViewRenderObject" {
  kind      = "function"
  hooked    = true
  msvc_name = "?createViewRenderObject@LevelRendererPlayer@@QEAA?AUViewRenderObject@@AEAVScreenContext@@W4SubClientId@@@Z"
  notes     = "ViewRenderObject LevelRendererPlayer::createViewRenderObject(ScreenContext&, SubClientId), not virtual. The result is returned through a hidden pointer, so the arguments are this in rcx, the out pointer in rdx, the ScreenContext in r8 and the sub client id in r9b. Found from the one function that calls it, the per-frame game render function (it is the only function that loads the strings 'Game renderer - Extract frame', 'Game Renderer - prepare frame' and 'Game Renderer - render current frame'); it builds a vector of view objects (stride 3300h in 26.51 and 32E0h in 26.60, the `add r,imm` of its copy loop), asks the client for the sub client id with a virtual call (`mov r9d,eax`), and calls this function with the next vector slot as the out pointer. The body writes the sub client id byte into the ScreenContext (`movzx eax,byte ptr [r8+disp]; ...; mov [r8+disp],r9b`), then reads a renderer member of this and makes a virtual call with the constant 114h in r8d (`mov rax,[rcx+disp]; mov rcx,[rax+disp]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; mov r8d,imm; call [guard]`). Recipe 0 is that body run, from the `mov [r8+disp],r9b` store to the guard call, then function_start_of; it is a pattern_any with one alternative per build shape: 26.51 has `lea rdx,[rbp+disp32]`, 26.60 stores this to the frame first (`mov [rbp+disp],rcx`) and has `lea rdx,[rbp+disp8]`. The prologue (push list, frame size, xmm saves) is no longer in the pattern. Recipe 1 is the call site: string_xref of 'Game renderer - Extract frame' gives the render function, and the only `mov r9d,eax; call` in it is read as a branch target. Every displacement, immediate and branch byte is a wildcard; the r8/r9 bytes are the argument registers of the call. The check is function_start. What changed in 6ad79ee4-17d51891a1119914: the view object shrank (3300h to 32E0h), the frame size changed (so the xmm save displacements changed), an extra frame store of this was added, the ScreenContext offsets moved (sub client id byte at r8+7Ch to r8+74h) and the caller passes `r8 = [rbp+..] + 288h`. The old prologue pattern (it kept `mov r15,rdx; mov r13,rcx`, gone on 26.60) and the old call-site pattern (`mov rcx,r15; mov rdx,rsi; mov r8,[rbp+..]; mov r9d,eax`) both matched nothing. Class: real code change. Evidence for the 26.60 value: (1) the three render strings of the old caller are loaded by one function on 26.60, and that function calls this one with the `mov r9d,eax` sub client id argument; the 26.60 value has exactly that one caller; (2) the body run drafted from the old function (sub client id store into r8, the renderer member chain, the virtual call with 114h, kept in both builds) is found once on 26.60, in this function."

  recipe {
    step "pattern_any" {
      patterns = [
        "45 88 48 ?? 48 8B 81 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 41 B8 ?? ?? ?? ?? FF 15",
        "45 88 48 ?? 48 89 8D ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? 41 B8 ?? ?? ?? ?? FF 15",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "string_xref" {
      text = "Game renderer - Extract frame"
    }
    step "pattern" {
      bytes  = "41 89 C1 E8 ?? ?? ?? ??"
      offset = 3
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LevelRendererPlayer.vtable" {
  kind     = "global"
  notes    = "The primary vtable of LevelRendererPlayer (the one at this+0). A helper entry: LevelRendererPlayer.extractNameTags reads its slot from it. Found with the zoom entry LevelRendererPlayer.PlayerListener.vtable: the constructor (0x46970c0 in 26.51) stores five vtable pointers in a row (`lea rax,[rip+..]; mov [rcx],rax` for this+0, then this+0C88h, +0C90h, +0C98h and +0CB0h), followed by a float constant (`mov rax,imm64`). The first store is this vtable; the third store is the PlayerListener table. Recipe 0 is that constructor run, ended by the `mov rax,imm64` opcode. Recipe 1 is the destructor, which stores the same five pointers and then loads the client instance (`mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]`). Both read the RIP target of the first `lea`. Every REX, ModRM, displacement and immediate byte is a wildcard. They are in two functions (the only two functions that refer to the table; `inspect xrefs` on 0xe8de6b0 lists the two). The section check keeps the address inside .rdata. Value 0xe8de6b0 in 26.51 (slot 12 is extractNameTags). What changed in 6ad79ee4-17d51891a1119914 (26.60): the value is 0xF246FD0. The constructor stores the five pointers and then loads a double (`movsd xmm0,[rip+..]; movsd [rcx+0CC0h],xmm0`) where 26.51 has `mov rax,imm64`, so recipe 0 is a pattern_any with one alternative per tail; recipe 1, the destructor, was not affected. Class: real code change (the constant after the stores). Cold run result: degraded (recipe 0 no match)."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? B8",
        "?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? F2 0F 10 05",
      ]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8B ?? ?? ?? ?? ?? ?? 8B ?? ?? 8B ?? ?? ?? ?? ??"
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

entry "LevelRendererPlayer.queueRenderEntities" {
  kind      = "vslot"
  msvc_name = "?$queueRenderEntities@LevelRendererPlayer@@QEAAXAEBULevelRenderPreRenderUpdateParameters@@@Z"
  notes     = "Slot of void LevelRendererPlayer::queueRenderEntities(LevelRenderPreRenderUpdateParameters const&), the LevelRendererCamera::queueRenderEntities override, in the LevelRendererPlayer.vtable global: 27 in both builds. `this` in rcx is the LevelRendererPlayer, the parameters are in rdx; it returns void. The override does a lookup (`lea rdx,[rcx+1170h]` in 26.51, `+1168h` in 26.60) and returns early when it fails; else it calls the base LevelRendererCamera::queueRenderEntities with the same `this` and rdx (the base is slot 27 of the LevelRendererCamera vtables; it loads the std::function vtables of its lambdas, which the std RTTI names), and then walks LevelRendererCamera.mActorRenderQueue and can erase the local player from it. A hook that runs after the original sees the queue that the camera will draw. Recipe 0 is the override head: `mov rsi,rdx; mov rdi,rcx; lea rdx,[rcx+member]; lea rcx,[rsp+..]; call; cmp byte ptr [rsp+..],imm; jne rel32`, then `function_start_of` and `slot_of` LevelRendererPlayer.vtable. Recipe 1 is the direct call in LevelRendererPlayer::preRenderUpdate (`movss xmm2,[rbx+disp]; mov rcx,rdi; mov rdx,r14; call; mov rcx,rdi; mov rdx,rbx; call`), read as a branch target, then `slot_of`. 26.60 puts a `nop` after each call, so recipe 1 is a `pattern_any` with one alternative for each form; the 26.51 alternative starts with one wildcard so that the second call is at offset 23 in both. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. The range check is a window around slot 27. Values: slot 27 in 26.51 (0x46ab7c0) and 26.60 (0x5390b60). What changed in 26.60: the member offset and the `nop` after calls."

  recipe {
    step "pattern" {
      bytes = "48 89 D6 48 89 CF 48 8D 91 ?? ?? ?? ?? 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 80 7C 24 ?? ?? 0F 85"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "LevelRendererPlayer.vtable" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? F3 0F 10 53 ?? 48 89 F9 4C 89 F2 E8 ?? ?? ?? ?? 48 89 F9 48 89 DA E8",
        "F3 0F 10 53 ?? 48 89 F9 4C 89 F2 E8 ?? ?? ?? ?? 90 48 89 F9 48 89 DA E8",
      ]
      offset = 23
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "slot_of" { vtable = "LevelRendererPlayer.vtable" }
  }

  check "range" {
    min = 22
    max = 32
  }
}

entry "LevelRendererPlayer.extractNameTags" {
  kind     = "vslot"
  notes    = "Slot of NameTagRenderObjectCollection LevelRendererPlayer::extractNameTags(ScreenContext&) const in the LevelRendererPlayer vtable (LevelRendererPlayer.vtable): 12 in 26.51 (neighbours: slot 11 renders the name tags of a view object, slot 13 is callRenderNameTags, slot 14 is extractCracks). The collection (40 bytes) is returned through a hidden pointer: rcx = this, rdx = the 40-byte result buffer, r8 = the ScreenContext; the function returns the buffer in rax. The body shows it: it saves rdx, reads the frame allocator from the ScreenContext (`mov rcx,[r8+108h]`, a LinearAllocator with a shared_ptr), stores the allocator into the first 16 bytes of the result (`mov [rdx],rax; mov [rdx+8],rbx`) and zeroes the vector (`movups [rdx+10h],xmm0; mov qword ptr [rdx+20h],0`). It is not called through the vtable in 26.51: createViewRenderObject calls it directly. Recipe 0 is the head of the body (`mov rsi,rcx; mov [rbp+..],r8; mov rcx,[r8+..]; mov rax,[rcx+8]; test rax,rax; mov [rbp+..],rdx; je; lock inc dword ptr [rax+8]; mov rax,[rcx]; mov rbx,[rcx+8]; xorps xmm0,xmm0; movups [rdx],xmm0`), then function_start_of and slot_of LevelRendererPlayer.vtable. Recipe 1 is the call in createViewRenderObject (`lea rdx,[rbp+..]; mov rcx,r13; mov r8,[rbp+..]; call` twice in a row, the second is extractCracks), read as a branch target and turned into a slot. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard. The check is a window around the known slot. What changed in 6ad79ee4-17d51891a1119914 (26.60): the slot is still 12 (the function is at 0x5397810; slots 13 and 14 hold the next two functions as on 26.51). The call in createViewRenderObject uses `mov rcx,[rbp+..]` instead of `mov rcx,r13` and a `nop` follows each call, so recipe 1 is a pattern_any with one alternative per build (the 26.51 alternative has four leading wildcards so that one offset serves both). Recipe 0 derived on the cold run. Class: real code change (call-site registers, nop). Cold run result: failed (1 recipe agreed, 2 needed)."

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 4C 89 85 ?? ?? ?? ?? 49 8B 88 ?? ?? ?? ?? 48 8B 41 ?? 48 85 C0 48 89 95 ?? ?? ?? ?? 74 ?? F0 FF 40 ?? 48 8B 01 48 8B 59 ?? 0F 57 C0 0F 11 02"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "LevelRendererPlayer.vtable"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 4C 89 E9 4C 8B 85 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 95",
        "48 8D 95 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 4C 8B 85 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8D 95",
      ]
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "slot_of" {
      vtable = "LevelRendererPlayer.vtable"
    }
  }

  check "range" {
    min = 4
    max = 24
  }

}

entry "LevelRendererPlayer.getFov" {
  kind      = "function"
  hooked    = true
  msvc_name = "?getFov@LevelRendererPlayer@@QEBAMM_N@Z"
  notes     = "float LevelRendererPlayer::getFov(float a, bool enableVariableFOV) const, not virtual. Found by its fields: mFov and mOFov at 0xF10 and 0xF14 in LeviLamina's layout; one of the three functions that read both does `(mFov - mOFov) * a + mOFov` with `a` from xmm1, after `test r8b,r8b`, and returns the value in xmm0. Recipe 0 is the prologue (three pushes, three xmm saves, `mov ebx,r8d; movaps xmm6,xmm1; mov rsi,rcx; test r8b,r8b; je`). Recipe 1 is the interpolation itself (two field loads, `subss`, `mulss`, `addss`, `mulss`), 0xCC bytes in, then `function_start_of`. Recipe 2 is the call site that passes true (`mov rcx,rdi; movaps xmm1,xmm0; mov r8b,imm; call`), read as a branch target. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). The three share no bytes, are unique in both exes and agree; `function_start` confirms the start. The entry did not break in the 26.60 cold derive."

  recipe {
    step "pattern" {
      bytes = "56 57 53 48 83 EC ?? 44 0F 29 44 24 ?? 0F 29 7C 24 ?? 0F 29 74 24 ?? 44 89 C3 0F 28 F1 48 89 CE 45 84 C0 0F 84"
    }
  }

  recipe {
    step "pattern" {
      bytes = "F3 0F 10 BE ?? ?? ?? ?? F3 0F 10 86 ?? ?? ?? ?? F3 0F 5C F8 F3 0F 59 FE F3 0F 58 F8 F3 0F 59 FA"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 F9 0F 28 C8 41 B0 ?? E8 ?? ?? ?? ?? E9"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LevelRendererPlayer.onWillChangeDimension" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onWillChangeDimension@LevelRendererPlayer@@QEAAXAEAVPlayer@@@Z"
  notes     = "void LevelRendererPlayer::onWillChangeDimension(Player&), the PlayerListener::onWillChangeDimension override: slot 1 of the LevelRendererPlayer.PlayerListener.vtable global in both builds. It calls the base LevelRendererCameraListeners::onWillChangeDimension with the same `this` and then clears a bool at this+88h; onDimensionChanged (slot 2) follows it in memory and starts by moving `this` back to the LevelRendererPlayer (`add rcx,-0C90h` in 26.51, `-0C88h` in 26.60). The recipe is the whole 26-byte body, its six int3 and onDimensionChanged's prologue up to that adjust and its call (`push rbp; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov rdi,rdx; mov rsi,rcx; add rcx,imm; call`). Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). The body alone matches twice in each build; with the neighbour it is unique. It relies on onDimensionChanged following this function, as in both builds. The checks are `function_start` and `in_vtable` of LevelRendererPlayer.PlayerListener.vtable, slots 0 to 3 (the four PlayerListener virtuals; the window does not reach into the next vtable); the check replaces the old vtable route with a fixed `vtable_slot 1`. The entry has one recipe and two strong checks. What changed in 26.60: the PlayerListener subobject moved (adjust -0C90h to -0C88h); the old recipes 0 and 1 failed there (recipe 0 matched twice, recipe 1 kept the adjust literal) and only the vtable route was left."

  recipe {
    step "pattern" {
      bytes = "56 48 83 EC ?? 48 89 CE E8 ?? ?? ?? ?? C6 86 ?? ?? ?? ?? ?? 48 83 C4 ?? 5E C3 CC CC CC CC CC CC 55 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 D7 48 89 CE 48 81 C1 ?? ?? ?? ?? E8"
    }
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "LevelRendererPlayer.PlayerListener.vtable"
    min_slot = 0
    max_slot = 3
  }
}

entry "LevelRendererPlayer.PlayerListener.vtable" {
  kind     = "global"
  notes    = "The PlayerListener vtable of LevelRendererPlayer (destructor, onWillChangeDimension, onDimensionChanged, onPlayerDestruction). Found through the vtables: the string 'virtual void __cdecl LevelRendererPlayer::preRenderUpdate(ScreenContext &, LevelRenderPreRenderUpdateParameters &)' sits in a source-location record next to the LevelRendererPlayer vtables; the functions that load them are the constructor and the destructor. In the constructor, after the shared-from-this stores (`mov [reg+..],rax; mov [reg+..],r14`), three vtable stores follow (`lea; mov [reg],rax; lea; mov [reg+..],rax; lea; mov [reg+..],rax`); the third `lea` (offset 38) is this vtable, stored at this+0C90h in 26.51 and this+0C88h in 26.60. Every REX, ModRM, displacement and immediate byte is wildcarded, so the registers are free; the pattern is unique in both exes. The section check keeps the address inside .rdata. It exists for the `in_vtable` check of onWillChangeDimension; it was added in Plan 5b. Values: 0xe8de8c0 in 26.51, 0xf2471e0 in 26.60."

  recipe {
    step "pattern" {
      bytes  = "?? 89 ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ??"
      offset = 38
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
