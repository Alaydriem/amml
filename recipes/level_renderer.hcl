entry "LevelRenderer.renderLevel" {
  kind      = "function"
  hooked    = true
  msvc_name = "?renderLevel@LevelRenderer@@QEAAXAEAVScreenContext@@AEBUFrameRenderObject@@@Z"
  notes     = "void LevelRenderer::renderLevel(ScreenContext&, FrameRenderObject const&): rcx = this, rdx = screen, r8 = frame. InGamePlayScreen::_renderLevel (named by its `virtual void __cdecl InGamePlayScreen::_renderLevel(ScreenContext &, const FrameRenderObject &)` profiler zone) calls it twice with its own screen and frame arguments passed through. The body reads this+408h (mClientInstance, header sum 1032) and calls its slot 79, then reads this+468h, whose +654h..+668h are LevelRendererCamera.mCameraPos and mCameraTargetPos. Recipe 0 is the head of the body (`mov r14,r8; mov rsi,rdx; mov rbx,rcx; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of. Recipe 1 is the second call site, after a pointer-fill loop: `cmp r8,rax; jne; mov rcx,rbx; mov rdx,[rbp+..]; mov r8,r15; call`, read as a branch target. Every displacement and branch byte is a wildcard. Check function_start. Recipe 0 now starts at the frame setup store before the head (`mov qword ptr [rbp+..],-2`) and wildcards the ModRM byte of `mov r64,rcx`. Recipe 1 now finds InGamePlayScreen::_renderLevel by the text \"Game Play Screen - Render 1st person objects workaround\" (only that function loads it, on both builds), then the second call (`mov r8,r15; call`) by the code after it, one pattern_any alternative per shape: alternative 0 (26.51) `xor r12d,r12d; cmp qword ptr [rsi+..],0; jne`, alternative 1 (26.60) `xor edi,edi; mov rax,[rsi+..]; test rax,rax`. Values per build: mClientInstance 408h and slot 79 on 26.51, 418h and slot 78 on 26.60; the camera object 468h on 26.51, 480h on 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): LevelRenderer members moved (+10h), so `this` is kept in rdi; the caller keeps the screen in r13. The 26.60 function was accepted on two facts from the 26.51 function: it is called twice by the function that loads the same text (the 26.51 caller of both call sites), and it has the same body head (the ClientInstance virtual call, then the camera pos and target reads at the LevelRendererCamera.mCameraPos and mCameraTargetPos values of 26.60, 64Ch and 658h). Would have survived with a better recipe. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): `this` is kept in r13 (`mov r13,rcx`, REX 49), so recipe 0 wildcards the REX byte of that move too; the caller passes the frame in r14 (`mov r8,r14`) and again clears r12 after the second call (`xor r12d,r12d; cmp qword ptr [rsi+..],0; jne`), so recipe 1 alternative 0 wildcards the ModRM byte of `mov r8,..` and now matches 26.51 and the Preview. Both alternatives run past the call by design of the 26.60 repair (the bytes after the call tell the two call sites apart); on the Preview there is no nop after it. Evidence on the Preview: (1) the function that loads the text calls it twice, each time after the same argument setup as on 26.63 (`mov rcx,rbx; mov rdx,r13; mov r8,..`), the second time after the same pointer-fill loop; (2) the body head is the 26.63 head: the save of r8, rdx and rcx, the ClientInstance virtual call through `[this+418h]` (slot 1C8h/8 = 57 on the Preview, 78 on 26.63), the BaseActorRenderContext constructor call with r15 = `[rbp-..]`, then the camera reads `[this+488h]` (480h on 26.63) at +654h, +65Ch, +660h, +668h (LevelRendererCamera.mCameraPos and mCameraTargetPos are 8 bytes later than on 26.63). Class: would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4D 89 C6 48 89 D6 ?? 89 ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "string_xref" { text = "Game Play Screen - Render 1st person objects workaround" }
    step "pattern_any" {
      patterns = [
        "4D 89 ?? E8 ?? ?? ?? ?? 45 31 E4 48 83 BE ?? ?? ?? ?? ?? 75",
        "4D 89 F8 E8 ?? ?? ?? ?? 31 FF 48 8B 86 ?? ?? ?? ?? 48 85 C0",
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

entry "LevelRenderer.mAtlasTexture" {
  kind     = "field"
  notes    = "Offset of mce::TexturePtr LevelRenderer::mAtlasTexture: 1C0h on 26.51 (header sum 448: LevelListener vptr, AppPlatformListener 32 bytes, then 408 bytes of members). The LevelRenderer constructor (found from the vtable that the destructor stores) builds a 13-character resource location on the stack, then does `add rbx,1C0h` and constructs the TexturePtr at this+1C0h from the terrain atlas and that location (`mov rcx,rbx; call`). The destructor releases the two control blocks of that TexturePtr at +1D8h and +1C8h. The recipe starts after the call before it (`add rbx,imm; xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov rax,[rdi+..]; test rax,rax; mov rcx,rdi; je`) and reads the immediate. It is one pattern_any, one alternative per shape: alternative 0 (26.51) has `mov rcx,rdi` before the je; alternative 1 (26.60) has no such mov, and reads the control block again through rdi. Range window 1B0h..1D0h. Value 1C0h on 26.51, 1D0h on 26.60 (the value is an offset entry; the mod has no LevelRenderer mirror). What changed in 6ad79ee4-17d51891a1119914 (26.60): a member was added before mAtlasTexture (the header forecast), so the members from there on are 10h later: the atlas 1C0h to 1D0h, the tessellator pointer that the destructor frees 390h to 3A0h, mClientInstance 408h to 418h. The `mov rcx,rdi` before the je went away, so the 26.51 pattern did not match. The new site is the same code in the LevelRenderer constructor (add, then the TexturePtr built from the atlas at [rdi] and the 13-character location); `fields-LevelRenderer.ctor.txt` shows the first 30h bytes unchanged. Real code change (layout); the entry gives the new offset."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 81 C3 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 8B 47 ?? 48 85 C0 48 89 F9 74",
        "48 81 C3 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 8B 47 ?? 48 85 C0 74 ?? F0 FF 40 ?? 48 8B 47",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 432
    max = 464
  }
}

entry "LevelRenderer.mLevelRendererPlayer" {
  kind     = "field"
  notes    = "Offset of the LevelRendererPlayer pointer in LevelRenderer: 480h on 6ad79ee4-17d51891a1119914 (26.60). The object it points to is a LevelRendererPlayer, whose LevelRendererCamera base is at offset 0. Found in LevelRenderer::renderLevel (0x535f980): the body loads `mov rax,[rdi+480h]` (rdi holds this) and reads the camera position and target through it at once: `movss xmm6,[rax+654h]; movsd xmm7,[rax+64Ch]; movss xmm0,[rax+660h]`, then `movsd xmm10,[rax+658h]`. 64Ch is LevelRendererCamera.mCameraPos and 658h is LevelRendererCamera.mCameraTargetPos on this build, so the pointer is the LevelRendererCamera base. InGamePlayScreen::_renderLevel also calls 0x539dc10 on LevelRenderer+480h before it calls renderLevel. Recipe 0 starts at LevelRenderer.renderLevel and scans its function for `mov rax,[rdi+..]; movss xmm6,[rax+..]; movsd xmm7,[rax+..]; movss xmm0,[rax+..]` (the load, then the three camera reads); it reads the displacement of operand 1 of the load at offset 0. Every displacement byte is a wildcard. Range window 400h..500h. The entry has one recipe and a range check (the rule for a field). 26.51 is not derived for this entry; the 26.51 notes of LevelRenderer.renderLevel name the camera object at 468h there."

  recipe {
    step "ref" { entry = "LevelRenderer.renderLevel" }
    step "pattern" {
      bytes = "48 8B 87 ?? ?? ?? ?? F3 0F 10 B0 ?? ?? ?? ?? F2 0F 10 B8 ?? ?? ?? ?? F3 0F 10 80"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1024
    max = 1280
  }
}
