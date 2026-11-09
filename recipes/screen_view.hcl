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

entry "ScreenView.mVisualTree" {
  kind     = "field"
  notes    = "Offset in ScreenView of mVisualTree, the std::unique_ptr<VisualTree> of the view (a raw pointer): 50h on 26.60 and 26.62, right after mController (+40h, the shared_ptr<ScreenController>). The VisualTree holds the root control (VisualTree.mRootControl). Recipe 0 is UIScene::getRawScreenName (slot 58 of UIScene.vtable): `mov rsi,rdx; mov rax,[rcx+..]` (UIScene.mScreenView) `; mov rax,[rax+..]; test rax,rax; je; mov rcx,[rax+..]; test rcx,rcx; je; lock inc dword ptr [rcx+..]`, read as the displacement of the second load. Recipe 1 is UIScene::getAreaOfControlByName (slot 52): `mov rsi,rdx; mov rax,[rcx+..]; pxor xmm0,xmm0; movdqu [rdx],xmm0; mov rcx,[rax+..]; lea rdx,[rbp+..]; xor r9d,r9d; call <name lookup>`, read as the displacement of the load of rcx, the VisualTree handed to the lookup. The recipes are in two functions. Their patterns share only the first two instructions (`mov rsi,rdx; mov rax,[rcx+..]`, the return slot and UIScene.mScreenView, 7 bytes); after them the two bodies differ. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value 50h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" {
      bytes  = "48 89 D6 48 8B 41 ?? 48 8B 40 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 160
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ScreenView.handleButtonEvent" {
  kind     = "function"
  hooked   = true
  notes    = "void ScreenView::handleButtonEvent(uint buttonId, ButtonState buttonState, FocusImpact focusImpact): rcx the ScreenView, edx the button id, r8b the ButtonState (1 pressed, 0 released), r9b the FocusImpact; returns void. Every UI button event of a scene passes through it: UIScene::handleButtonPress (slot 22 of UIScene.vtable) and UIScene::handleButtonRelease (slot 23) are leaf thunks without .pdata that load the ScreenView from UIScene +40h, set r8b to 1 or 0, move the FocusImpact to r9d and jump here; the input event handler of ScreenView (called by UIScene::handleRawInputEvent, slot 24) calls it directly for events of type 1 with FocusImpact 0. A mouse click is a press and a release of a pointer button id (ScreenView.mPointerButtonIds). The function tests the button id against those ids and dispatches the event to the controls at the pointer location (ScreenView.mPointerLocationPrevious, passed as a vec2 pointer to the control dispatch); a hook that returns without the original drops that one press or release and nothing else, so a slot under the pointer never sees it. Recipe 0 is the head of the body after the prologue: `mov [rbp+..],r8b; cmp byte ptr [rcx+..],..; jne; mov rsi,rcx; cmp byte ptr [rcx+..],..; jne; mov edi,edx; test r9b,r9b; je`, then function_start_of. Recipe 1 is the direct call site in the ScreenView input event handler: `cmp dword ptr [rdi],..; jne; movzx r8d,byte ptr [rdi+..]; mov edx,[rdi+..]; mov rcx,rsi; xor r9d,r9d; call <this>`, read as the call target. The two recipes are in two functions and share no pattern bytes; the pattern of recipe 1 ends at the call. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end with function_start_of, so it counts). The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value 4CBC1A0h on 6ad79ee4-17d51891a1119914 (26.60) and 4CBC370h on 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "pattern" { bytes = "44 88 45 ?? 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 89 CE 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 89 D7 45 84 C9 0F 84" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "83 3F ?? 75 ?? 44 0F B6 47 ?? 8B 57 ?? 48 89 F1 45 31 C9 E8 ?? ?? ?? ??"
      offset = 19
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

entry "ScreenView.mPointerButtonIds" {
  kind     = "field"
  notes    = "Offset in ScreenView of mPointerButtonIds, a std::array<uint, 2> of the button ids that are pointer buttons (the mouse buttons of the UI): 38h on 26.60 and 26.62, right before mController (+40h). ScreenView.handleButtonEvent compares the event's button id with them to tell a click from a key or gamepad button. Recipe 0 is inside ScreenView.handleButtonEvent: the std::find over the array, `lea rcx,[rsi+..]` (begin) `; lea rbx,[rsi+..]` (end) `; mov rdx,rbx; mov r8d,edi` (the button id) `; mov r14d,eax; call <find>`, read as the displacement of the first lea. The pattern ends at the call. There is one recipe: no second reader was found (std::find with this array has one caller in ScreenView, and the constructor of ScreenView is inlined into the UIScene constructor, where the stores to the array were not located), so the check is a window around the value, as for a field with one recipe. Every displacement, immediate and branch byte is a wildcard. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate: the entry refuses as not for this build on 26.51. Value 38h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

  recipe {
    step "ref" { entry = "ScreenView.handleButtonEvent" }
    step "pattern" { bytes = "48 8D 4E ?? 48 8D 5E ?? 48 89 DA 41 89 F8 41 89 C6 E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 60
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ScreenView.mInputMode" {
  kind     = "field"
  notes    = "Offset in ScreenView of mInputMode, the game's InputMode enum (4 bytes; LeviLamina headers: Undefined 0, Mouse 1, Touch 2, GamePad 3, MotionController 4): 200h on 26.60, 26.62 and 26.63, right after mSelectedControl (a std::weak_ptr<UIControl> at +1F0h, its control block read at +1F8h) and before mLayoutManager (+208h) and mFocusManager (+210h), as in the LeviLamina header member order. It was found from the two resolved ScreenView functions: both compare this dword with InputMode values. Recipe 0 is in ScreenView.render, right after the two virtual calls on the screen model: `mov rax,[rax+..]; call [guard]; cmp dword ptr [rdi+..],..` (the compare with 1, Mouse: in mouse mode the selected-control block is skipped) `; je; xorps xmm0,xmm0`, read as the displacement of the memory operand of the cmp. The next block loads the weak_ptr control block of mSelectedControl (`mov rcx,[rdi+1F8h]`). Recipe 1 is in ScreenView.handleButtonEvent, on the FocusImpact path: `cmp r9b,..; jne; mov r14b,..; cmp dword ptr [rsi+..],..` (the compare with 3, GamePad) `; jne; cmp byte ptr [rsi+..],..; jne; movss xmm0,[rsi+..]; ucomiss xmm0,[rip+..]`, read as the displacement of the memory operand of the cmp; the same block loads mFocusManager (`mov rbx,[rsi+210h]`) just before. The two recipes are in two functions, start from two different entries and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard (the compared InputMode values too), so lint keeps no byte. The range check is a wide window around the value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) is a build gate, as on the other ScreenView entries: the entry refuses as not for this build on 26.51 (its recipes also depend on ScreenView.render and ScreenView.handleButtonEvent, which refuse there). Value 200h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "ref" { entry = "ScreenView.render" }
    step "pattern" {
      bytes  = "48 8B 40 ?? FF 15 ?? ?? ?? ?? 83 BF ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 0F 57 C0"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ScreenView.handleButtonEvent" }
    step "pattern" {
      bytes  = "41 80 F9 ?? 75 ?? 41 B6 ?? 83 BE ?? ?? ?? ?? ?? 75 ?? 80 BE ?? ?? ?? ?? ?? 75 ?? F3 0F 10 86 ?? ?? ?? ?? 0F 2E 05"
      offset = 9
    }
    step "operand" {
      index = 0
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
