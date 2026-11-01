entry "LevelRendererCamera.preRenderUpdate" {
  kind      = "function"
  hooked    = true
  msvc_name = "?preRenderUpdate@LevelRendererCamera@@UEAAXAEAVScreenContext@@AEAVLevelRenderPreRenderUpdateParameters@@@Z"
  notes     = "Address of `void LevelRendererCamera::preRenderUpdate(ScreenContext&, LevelRenderPreRenderUpdateParameters&)`: 0x46855b0 in 26.51 (this in rcx, the ScreenContext in rdx, the parameters in r8). It is virtual, but the hook needs the base function itself: LevelRendererPlayer::preRenderUpdate (slot 22 of the LevelRendererPlayer vtable, 0x46aa3d0) calls it directly as its first real step, and a writer of mCameraPos and mCameraTargetPos must run right after it and before the rest of the player update (chunk sorting reads those members). A vtable slot would give the player override, so the entry is the function address. Two vtables hold it (data pointers at 0xe8de050 and 0xe8de530), and two functions call it directly: LevelRendererPlayer::preRenderUpdate and the preRenderUpdate of another subclass (0x46d7900). The body starts with `mov rbx,rdx; mov rsi,rcx; mov rax,[rdx+0B0h]; movss xmm0,[rax]; movaps xmm1,xmm0; subss xmm1,[rcx+404h]; mov rdi,r8`, then calls mce::Camera::updateViewMatrixDependencies on the camera of the ScreenContext and copies that camera into mWorldSpaceCamera. Recipe 0 is that head (`mov rbx,rdx; mov rsi,rcx; mov rax,[rdx+..]; movss xmm0,[rax]; movaps xmm1,xmm0; subss xmm1,[rcx+..]; mov rdi,r8`), then function_start_of. Recipe 1 starts at the head of LevelRendererPlayer::preRenderUpdate (the argument moves `mov rbx,r8; mov r14,rdx; mov rdi,rcx`, the profiler check through gs:[58h], and the profile scope set-up up to its call; the other subclass sets the scope up with `lea rsi` and `mov rdx,rsi`, so this part is unique), then finds the call that passes the three arguments on (`mov [rbp-10h],al; mov rcx,rdi; mov rdx,r14; mov r8,rbx; call`) in the same function and reads its branch target. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" {
      bytes = "48 89 D3 48 89 CE 48 8B 82 ?? ?? ?? ?? F3 0F 10 00 0F 28 C8 F3 0F 5C 89 ?? ?? ?? ?? 4C 89 C7"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "4C 89 C3 49 89 D6 48 89 CF 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? C6 45 ?? ?? 48 8D 55 ?? 48 C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 4C 8D 45 ?? E8"
    }
    step "pattern" {
      bytes  = "88 45 ?? 48 89 F9 4C 89 F2 49 89 D8 E8"
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "LevelRendererCamera.mCameraPos" {
  kind     = "field"
  notes    = "Offset of `mCameraPos` (Vec3) in LevelRendererCamera: 654h in 26.51, 64Ch in 6ad79ee4-17d51891a1119914 (a member before it shrank by 8; mCameraTargetPos follows 0Ch later in both). LeviLamina header sum on 26.51: from mWorldSpaceCamera 740h back, FrustumCullerType 738h, FrustumCuller 66Ch, mCameraTargetPos 660h, mCameraPos 654h. Recipe 0 is LevelRendererPlayer::setupCamera, which stores the camera position into the LevelRendererCamera in rsi and copies its z to the target z (`mov eax,[rsi+pos+8]; mov [rsi+target+8],eax`, the same two instructions on both builds; the pattern_any alternatives differ only in the code that follows: 26.51 goes on to the camera component lookup (`mov rcx,[rsp+..]; mov eax,[rsp+..]; mov rdx,[rcx+..]; mov r8,[rcx+..]`), 26.60 first copies the target x and y (`mov rax,[rsi+..]; mov [rsi+..],rax`)); it reads the displacement of the load (operand 1) and subtracts 8. Recipe 1 is LevelRendererCamera::preRenderUpdate, which floors the camera position for the chunk sort (`test rbx,rbx; je; movss xmm6,[rsi+pos+8]; movss xmm0,[rsi+pos]; movss xmm7,[rsi+pos+4]; call`, the same shape on both builds); it reads the displacement of the x load. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the layout (the value moved by -8) and the setupCamera code shape (the position is copied with movsd). Class: layout change. Evidence for 64Ch on 26.60, from the 26.51 functions: (1) setupCamera stores the position and then the target in the same order as 26.51 (26.51: qword at 654h, z at 65Ch, qword at 660h, z at 668h; 26.60: movsd at 64Ch, z at 654h, z at 660h, qword at 658h); (2) preRenderUpdate floors the three position floats with the same instruction run (26.51 65Ch, 654h, 658h; 26.60 654h, 64Ch, 650h); (3) createViewRenderObject copies the position and target into the view data in the same order (26.51 one float at a time from 654h to 668h; 26.60 movups from 64Ch, movsd from 65Ch)."

  recipe {
    step "pattern_any" {
      patterns = [
        "8B 86 ?? ?? ?? ?? 89 86 ?? ?? ?? ?? 48 8B 4C 24 ?? 8B 44 24 ?? 48 8B 51 ?? 4C 8B 41 ??",
        "8B 86 ?? ?? ?? ?? 89 86 ?? ?? ?? ?? 48 8B 86 ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 8B 4C 24 ?? 8B 44 24 ??",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" {
      value = -8
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 85 DB 0F 84 ?? ?? ?? ?? F3 0F 10 B6 ?? ?? ?? ?? F3 0F 10 86 ?? ?? ?? ?? F3 0F 10 BE ?? ?? ?? ?? E8"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1024
    max = 2304
  }
}

entry "LevelRendererCamera.mCameraTargetPos" {
  kind     = "field"
  notes    = "Offset of `mCameraTargetPos` (Vec3) in LevelRendererCamera: 660h in 26.51, 658h in 6ad79ee4-17d51891a1119914 (0Ch after mCameraPos on both builds; see LevelRendererCamera.mCameraPos). Recipe 0 is the same setupCamera site as mCameraPos recipe 0 (`mov eax,[rsi+pos+8]; mov [rsi+target+8],eax`): it reads the displacement of the store (operand 0) and subtracts 8. Recipe 1 is a second part of setupCamera: an offset is added to the target, and the z add is the last before the call that follows (`addss reg,[rsi+target+8]; movss [rsi+target+8],reg; mov rcx,[rsi+..]; call`; the register bytes are wildcards, 26.51 uses xmm1 and 26.60 xmm0); it reads the displacement of the add (operand 1) and subtracts 8. The two recipes anchor in two parts of setupCamera that share no bytes (the function is the only one that writes the target). Every displacement, immediate, branch and register byte in the recipe 1 pattern is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the layout (the value moved by -8) and the setupCamera code shape. Class: layout change. Evidence for 658h on 26.60, from the 26.51 functions: (1) setupCamera stores the target after the position in the same order (26.51 qword at 660h and z at 668h; 26.60 z at 660h and qword at 658h); (2) the target add in setupCamera reads x, y, z (26.51 660h, 664h, 668h; 26.60 movsd 658h and 660h); (3) createViewRenderObject copies the target after the position (26.51 660h to 668h; 26.60 the last float of the movups from 64Ch, then movsd from 65Ch)."

  recipe {
    step "pattern_any" {
      patterns = [
        "8B 86 ?? ?? ?? ?? 89 86 ?? ?? ?? ?? 48 8B 4C 24 ?? 8B 44 24 ?? 48 8B 51 ?? 4C 8B 41 ??",
        "8B 86 ?? ?? ?? ?? 89 86 ?? ?? ?? ?? 48 8B 86 ?? ?? ?? ?? 48 89 86 ?? ?? ?? ?? 48 8B 4C 24 ?? 8B 44 24 ??",
      ]
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" {
      value = -8
    }
  }

  recipe {
    step "pattern" {
      bytes = "F3 0F 58 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? E8"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" {
      value = -8
    }
  }

  check "range" {
    min = 1024
    max = 2304
  }
}

entry "LevelRendererCamera.mActorRenderQueue" {
  kind     = "field"
  notes    = "Offset of `mActorRenderQueue` in LevelRendererCamera: 1D8h in both builds. It is a brstd::flat_map<gsl::not_null<Actor*>, ShadowContext>: two MSVC std::vectors side by side, 30h bytes in all. The keys vector (8-byte Actor*) has begin, end and cap at +0, +8, +10h (1D8h, 1E0h, 1E8h); the values vector (1-byte ShadowContext) has them at +18h, +20h, +28h (1F0h, 1F8h, 200h). The member before it, at 1C0h, is mActorRenderCandidates, a plain vector<Actor*> that the base queueRenderEntities sorts and dedupes; it is not the queue. Evidence: the LevelRendererPlayer queueRenderEntities override walks the keys by 8 and the values by 1 and erases an entry with a memmove on each vector and `end -= 8` and `end -= 1`; the LevelRendererCamera destructor frees the values vector (begin 1F0h, cap 200h) and the keys vector (begin 1D8h, cap 1E8h); an inlined emplace in 26.60 grows both triples. The recipe is the head of that walk in the override, after the base call: `mov rbx,[rdi+keys]; mov r14,[rdi+values]; cmp rbx,[rdi+keys_end]; jne`. The `jne` is short in 26.51 and near in 26.60, so the recipe is a `pattern_any` with one alternative for each form; it reads the displacement of operand 1 of the first instruction at offset 0. Every displacement, immediate and branch byte is a wildcard. The entry has one recipe and a check (the rule for a field). What changed in 26.60: nothing in the layout; only the branch form."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 9F ?? ?? ?? ?? 4C 8B B7 ?? ?? ?? ?? 48 3B 9F ?? ?? ?? ?? 75",
        "48 8B 9F ?? ?? ?? ?? 4C 8B B7 ?? ?? ?? ?? 48 3B 9F ?? ?? ?? ?? 0F 85",
      ]
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
