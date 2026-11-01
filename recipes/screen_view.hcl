entry "ScreenView.render" {
  kind     = "function"
  hooked   = true
  notes    = "void ScreenView::render(UIRenderContext&): rcx the ScreenView, rdx the MinecraftUIRenderContext (the UIRenderContext&); LeviLamina's AfterUIRenderEvent hooks this function, calls the original and publishes after it. It runs once per UIScene per frame: the only direct caller is UIScene::render, which builds the MinecraftUIRenderContext on its stack (constructor call), calls this function with the scene's ScreenView at +40h, then flushes and destroys the context. The prologue is eight pushes, `mov eax,1238h` and a __chkstk call. Recipe 0 is a body anchor near the top of the function: the test of two virtual calls on the screen model at +390h (`mov rdi,rcx; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; test al,al; mov [rbp+..],rdi; je; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of. Recipe 1 is the call site: the prologue of UIScene::render (found by its own unique prologue, a stack-protector load and the thread-local guard test) and then, inside that function, the sequence `mov r9,rbx; call <context ctor>; nop; mov rcx,[rbx+..]; lea rdx,[rbp+..]; call <this>`, read as the call target. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and the 26.60 gate. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 4CB3260h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "48 89 CF 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 84 C0 48 89 BD ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "55 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 D7 48 89 CB 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ??" }
    step "pattern" {
      bytes  = "49 89 D9 E8 ?? ?? ?? ?? 90 48 8B 4B ?? 48 8D 55 ?? E8 ?? ?? ?? ??"
      offset = 17
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

entry "ScreenView.mPointerLocationPrevious" {
  kind     = "field"
  notes    = "Offset in ScreenView of mPointerLocationPrevious, a glm::vec2 (two f32): 33Ch on 26.60. ScreenView has no vtable; the header sum from the checked members (mController +40h, mSelectedControl +1F0h, mInputMode +200h, mStartLocation +420h) gives the same offset. Recipe 0 is the pointer-held block of ScreenView::render: the pointer start location is loaded (`movsd xmm1,[rdi+420h]; movsd [rbp+..],xmm1`) and this member is subtracted from it (`movsd xmm2,[rdi+..]; subps xmm1,xmm2; movlps [rbp+..],xmm1; andps; movaps; cmpltps`), read from the displacement of the second load. Recipe 1 is the pointer move handler (a second ScreenView method): `mov rax,[rbp+..]; movsd xmm0,[rax]; movsd xmm1,[rsi+..]; movaps xmm2,xmm0; subps xmm2,xmm1; movlps [rbp+..],xmm2`, read from the displacement of the load of xmm1. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a wide window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 33Ch on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 8F ?? ?? ?? ?? F2 0F 11 4D ?? F2 0F 10 97 ?? ?? ?? ?? 0F 5C CA 0F 13 4D ?? 0F 54 0D"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 45 ?? F2 0F 10 00 F2 0F 10 8E ?? ?? ?? ?? 0F 28 D0 0F 5C D1 0F 13 55 ??"
      offset = 8
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
