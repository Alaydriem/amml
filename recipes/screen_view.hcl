entry "ScreenView.render" {
  kind     = "function"
  hooked   = true
  notes    = "void ScreenView::render(UIRenderContext&): rcx the ScreenView, rdx the MinecraftUIRenderContext (the UIRenderContext&); LeviLamina's AfterUIRenderEvent hooks this function, calls the original and publishes after it. It runs once per UIScene per frame: the only direct caller is UIScene::render, which builds the MinecraftUIRenderContext on its stack (constructor call), calls this function with the scene's ScreenView at +40h, then flushes and destroys the context. The prologue is eight pushes, `mov eax,1238h` and a __chkstk call. Recipe 0 is a body anchor near the top of the function: the test of two virtual calls on the screen model at +390h (`mov <reg>,rcx; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; test al,al; mov [rbp+..],rdi; je; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of. Recipe 1 is the call site: the prologue of UIScene::render (found by its own unique frame setup, a stack-protector load and the thread-local guard test) and then, inside that function, the sequence `mov r9,rbx; call <context ctor>; nop; mov rcx,[rbx+..]; lea rdx,[rbp+..]; call <this>`, read as the call target. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and the 26.60 gate. Value 4CB3260h on 6ad79ee4-17d51891a1119914 (26.60). What changed in 6afa4e47-dadc375797a3d31a (70.26): the register that holds the ScreenView changed (rdi to rsi, so the ModRM and the register of two moves differ in recipe 0), and UIScene::render gained a `push r14` in its prologue (recipe 1 started with the push list); would have survived with a better recipe. Recipe 0 now wildcards those registers and recipe 1 starts at the `mov qword ptr [rbp+..],-2` that follows the frame setup. The anchor of recipe 1 is the first scan step; the in-function pattern and the call read stay. Evidence on 70.26: recipe 0 (a body anchor drafted from the 26.63 function) and recipe 1 (the call at the same place in the call order of UIScene::render, after the context constructor call) reach the same function. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" { bytes = "48 89 ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 84 C0 48 89 ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 D7 48 89 CB 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ??" }
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
}

entry "ScreenView.mPointerLocationPrevious" {
  kind     = "field"
  notes    = "Offset in ScreenView of mPointerLocationPrevious, a glm::vec2 (two f32): 33Ch on 26.60. ScreenView has no vtable; the header sum from the checked members (mController +40h, mSelectedControl +1F0h, mInputMode +200h, mStartLocation +420h) gives the same offset. Recipe 0 is the pointer-held block of ScreenView::render: the pointer start location is loaded (`movsd xmm1,[<reg>+420h]; movsd [rbp+..],xmm1`) and this member is subtracted from it (`movsd xmm2,[rdi+..]; subps xmm1,xmm2; movlps [rbp+..],xmm1; andps; movaps; cmpltps`), read from the displacement of the second load. Recipe 1 is the pointer move handler (a second ScreenView method): `movsd xmm0,[<reg>]; movsd xmm1,[rsi+..]; movaps xmm2,xmm0; subps xmm2,xmm1; movlps [rbp+..],xmm2`, read from the displacement of the load of xmm1. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a wide window around the known value. Value 33Ch on 6ad79ee4-17d51891a1119914 (26.60) and 33Ch on 6afa4e47-dadc375797a3d31a (70.26). What changed in 6afa4e47-dadc375797a3d31a (70.26): the register that holds the ScreenView changed in ScreenView::render (rdi to rbx, so the ModRM bytes of the loads differ, recipe 0), and the move handler loads the previous location through another base register (recipe 1 no longer starts with `mov rax,[rbp+..]`); the member did not move. Would have survived with a better recipe: recipe 0 now wildcards the registers, recipe 1 starts at the two loads. Evidence on 70.26: the two recipes are in two functions (ScreenView::render and the move handler) drafted from the 26.63 code and give the same offset; the member sits between mInputMode (200h) and mStartLocation (420h) as in the header sum. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 ?? ?? ?? ?? ?? F2 0F 11 ?? ?? F2 0F 10 ?? ?? ?? ?? ?? 0F 5C ?? 0F 13 ?? ?? 0F 54"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 ?? F2 0F 10 ?? ?? ?? ?? ?? 0F 28 ?? 0F 5C ?? 0F 13 ?? ??"
      offset = 4
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

entry "ScreenView.mVisualTree" {
  kind     = "field"
  notes    = "Offset in ScreenView of mVisualTree, the std::unique_ptr<VisualTree> of the view (a raw pointer): 50h on 26.60 and 26.62, right after mController (+40h, the shared_ptr<ScreenController>). The VisualTree holds the root control (VisualTree.mRootControl). Recipe 0 is UIScene::getRawScreenName (slot 58 of UIScene.vtable): `mov rsi,rdx; mov rax,[rcx+..]` (UIScene.mScreenView) `; mov rax,[rax+..]; test rax,rax; je; mov rcx,[rax+..]; test rcx,rcx; je; lock inc dword ptr [rcx+..]`, read as the displacement of the second load. Recipe 1 is UIScene::getAreaOfControlByName (slot 52): `mov rsi,rdx; mov rax,[rcx+..]; pxor xmm0,xmm0; movdqu [rdx],xmm0; mov rcx,[rax+..]; lea rdx,[rbp+..]; xor r9d,r9d; call <name lookup>`, read as the displacement of the load of rcx, the VisualTree handed to the lookup. The recipes are in two functions. Their patterns share only the first two instructions (`mov rsi,rdx; mov rax,[rcx+..]`, the return slot and UIScene.mScreenView, 7 bytes); after them the two bodies differ. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. Value 50h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62) and 6afa4e47-dadc375797a3d31a (70.26). What changed in 6afa4e47-dadc375797a3d31a (70.26): both functions changed shape (real code change in the code, same member). On 70.26 UIScene::getRawScreenName (slot 58, same vtable neighbours as before) no longer loads the member itself: it calls a ScreenView helper that does `mov rsi,rdx; mov rcx,[rcx+..]` (the VisualTree) `; test rcx,rcx; je; lea rdx,[rbp+..]; call`; recipe 0 therefore has a second alternative for that helper, with four wildcard bytes in front so that the displacement sits at the same offset (7) as in the old shape. In UIScene::getAreaOfControlByName the zeroing of the return slot is `xorps; movups` instead of `pxor; movdqu`; recipe 1 has a second alternative with two wildcard bytes in front (offset 15 for both). The two alternatives of each recipe match one shape on one build and nothing on the other. Class: would have survived with a better recipe. Evidence on 70.26: the vtable of UIScene lines up around slot 58 and 52 (slots 51, 52, 56, 57, 58, 59 are functions in the same places on both builds; slot 58 calls the helper of recipe 0 and slot 52 holds the function of recipe 1), and the two recipes, in two different functions, read the same displacement, 50h, as the old function did. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 D6 48 8B 41 ?? 48 8B 40 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ??",
        "?? ?? ?? ?? 48 89 D6 48 8B 49 ?? 48 85 C9 74 ?? 48 8D 55 ?? E8",
      ]
      offset   = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8",
        "?? ?? 48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8",
      ]
      offset   = 15
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
}

entry "ScreenView.handleButtonEvent" {
  kind     = "function"
  hooked   = true
  notes    = "void ScreenView::handleButtonEvent(uint buttonId, ButtonState buttonState, FocusImpact focusImpact): rcx the ScreenView, edx the button id, r8b the ButtonState (1 pressed, 0 released), r9b the FocusImpact; returns void. Every UI button event of a scene passes through it: UIScene::handleButtonPress (slot 22 of UIScene.vtable) and UIScene::handleButtonRelease (slot 23) are leaf thunks without .pdata that load the ScreenView from UIScene +40h, set r8b to 1 or 0, move the FocusImpact to r9d and jump here; the input event handler of ScreenView (called by UIScene::handleRawInputEvent, slot 24) calls it directly for events of type 1 with FocusImpact 0. A mouse click is a press and a release of a pointer button id (ScreenView.mPointerButtonIds). The function tests the button id against those ids and dispatches the event to the controls at the pointer location (ScreenView.mPointerLocationPrevious, passed as a vec2 pointer to the control dispatch); a hook that returns without the original drops that one press or release and nothing else, so a slot under the pointer never sees it. Recipe 0 is the head of the body after the prologue: `mov [rbp+..],r8b; cmp byte ptr [rcx+..],..; jne; mov rsi,rcx; cmp byte ptr [rcx+..],..; jne; mov edi,edx; test r9b,r9b`, then function_start_of (the pattern ends before the `je`: it is a near jump on 26.63 and earlier, a short jump on 70.26). Recipe 1 is the direct call site in the ScreenView input event handler: `cmp dword ptr [rdi],..; jne; movzx r8d,byte ptr [rdi+..]; mov edx,[rdi+..]; mov rcx,rsi; xor r9d,r9d; call <this>`, read as the call target. The two recipes are in two functions and share no pattern bytes; the pattern of recipe 1 ends at the call. Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 1 does not end with function_start_of, so it counts). Value 4CBC1A0h on 6ad79ee4-17d51891a1119914 (26.60) and 4CBC370h on 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (70.26): the `je` after `test r9b,r9b` became a short jump, so recipe 0 no longer matched; would have survived with a better recipe. Evidence on 70.26: recipe 0 (a body anchor drafted from the 26.63 function) and recipe 1 (the call site in the input event handler) reach the same function. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" { bytes = "44 88 45 ?? 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 48 89 CE 80 B9 ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 89 D7 45 84 C9" }
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
}

entry "ScreenView.mPointerButtonIds" {
  kind     = "field"
  notes    = "Offset in ScreenView of mPointerButtonIds, a std::array<uint, 2> of the button ids that are pointer buttons (the mouse buttons of the UI): 38h on 26.60 and 26.62, right before mController (+40h). ScreenView.handleButtonEvent compares the event's button id with them to tell a click from a key or gamepad button. Recipe 0 is inside ScreenView.handleButtonEvent: the std::find over the array, `lea rcx,[rsi+..]` (begin) `; lea rbx,[rsi+..]` (end) `; mov rdx,rbx; mov r8d,edi` (the button id) `; mov r14d,eax; call <find>`, read as the displacement of the first lea. The pattern ends at the call. There is one recipe: no second reader was found (std::find with this array has one caller in ScreenView, and the constructor of ScreenView is inlined into the UIScene constructor, where the stores to the array were not located), so the check is a window around the value, as for a field with one recipe. Every displacement, immediate and branch byte is a wildcard. No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a (70.26): nothing in this entry; the dependency (ScreenView.handleButtonEvent) was repaired and the recipe, unchanged, matches once in the new body (the std::find call over [this+38h, this+40h)). Value 38h on 6afa4e47-dadc375797a3d31a (70.26) too. Value 38h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62)."

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
}

entry "ScreenView.mInputMode" {
  kind     = "field"
  notes    = "Offset in ScreenView of mInputMode, the game's InputMode enum (4 bytes; LeviLamina headers: Undefined 0, Mouse 1, Touch 2, GamePad 3, MotionController 4): 200h on 26.60, 26.62 and 26.63, right after mSelectedControl (a std::weak_ptr<UIControl> at +1F0h, its control block read at +1F8h) and before mLayoutManager (+208h) and mFocusManager (+210h), as in the LeviLamina header member order. It was found from the two resolved ScreenView functions: both compare this dword with InputMode values. Recipe 0 is in ScreenView.render, right after the two virtual calls on the screen model: `mov rax,[rax+..]; call [guard]; cmp dword ptr [rdi+..],..` (the compare with 1, Mouse: in mouse mode the selected-control block is skipped) `; je; xorps xmm0,xmm0`, read as the displacement of the memory operand of the cmp. The next block loads the weak_ptr control block of mSelectedControl (`mov rcx,[rdi+1F8h]`). Recipe 1 is in ScreenView.handleButtonEvent, on the FocusImpact path: `cmp r9b,..; jne; mov r14b,..; cmp dword ptr [rsi+..],..` (the compare with 3, GamePad) `; jne; cmp byte ptr [rsi+..],..; jne; movss xmm0,[rsi+..]; ucomiss xmm0,[rip+..]`, read as the displacement of the memory operand of the cmp; the same block loads mFocusManager (`mov rbx,[rsi+210h]`) just before. The two recipes are in two functions, start from two different entries and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard (the compared InputMode values too), so lint keeps no byte. The range check is a wide window around the value. No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a (70.26): recipe 0: the compiler uses rsi for the ScreenView, so the ModRM byte of the cmp is a wildcard now (would have survived with a better recipe). Recipe 1: the GamePad block calls a helper (`mov rcx,rsi; call`) in place of the inlined flag tests, so recipe 1 is a pattern_any with the old shape and the 70.26 shape (real code change in the body, same member). Value 200h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63)."

  recipe {
    step "ref" { entry = "ScreenView.render" }
    step "pattern" {
      bytes  = "48 8B 40 ?? FF 15 ?? ?? ?? ?? 83 ?? ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 0F 57 C0"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ScreenView.handleButtonEvent" }
    step "pattern_any" {
      patterns = [
        "41 80 F9 ?? 75 ?? 41 B6 ?? 83 BE ?? ?? ?? ?? ?? 75 ?? 80 BE ?? ?? ?? ?? ?? 75 ?? F3 0F 10 86 ?? ?? ?? ?? 0F 2E 05",
        "41 80 F9 ?? 75 ?? 41 B6 ?? 83 BE ?? ?? ?? ?? ?? 75 ?? 48 89 F1 E8",
      ]
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
}

entry "ScreenView.mSize" {
  kind     = "field"
  notes    = "Offset in ScreenView of mSize, a glm::vec2 (two f32, the screen size in GUI units): 18h on 26.63 and 70.26, right before mPointerButtonIds (38h) and mController (40h). It is read as a vec2 by the gamepad cursor update, which scales it (`movsd xmm5,[<reg>+..]; movaps xmm4,[rip+..]; mulps xmm4,xmm5; lea rdx,[<reg>+..]; movlps [<reg>+..],xmm4`: the cursor step is a fraction of the screen size, stored to the cursor position) and later converts it to integers (`mulps xmm7,xmm6; movsd xmm6,[<reg>+..]; cvttps2dq xmm6,xmm6; cvtdq2ps xmm6,xmm6; movsd xmm8,[<reg>+..]`). On 26.63 the cursor update is its own ScreenView method; on 70.26 it is inlined into ScreenView::render. Recipe 0 reads the displacement of the first load, recipe 1 the displacement of the load in the integer conversion; they are two blocks of the same code and share no pattern bytes. Every register, displacement and branch byte is a wildcard. The range check is a window around the value. Value 18h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). Evidence: the two blocks give one offset on both builds, and the member sits before mPointerButtonIds (38h) and mController (40h) as in the LeviLamina header member order. It is a plain field: a mirror reads it with the ScreenView pointer, and a mirror selects no layout with it."

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 ?? ?? 0F 28 ?? ?? ?? ?? ?? 0F 59 ?? 48 8D ?? ?? ?? ?? ?? 0F 13 ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 59 ?? F2 0F 10 ?? ?? F3 0F 5B ?? 0F 5B ?? F2 44 0F 10 ?? ?? ?? ?? ??"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 48
  }
}

entry "ScreenView.mController" {
  kind     = "field"
  notes    = "Offset in ScreenView of mController, the std::shared_ptr<ScreenController> of the view (the raw pointer at this offset, the control block right after it): 40h on 26.63 and 70.26. Recipe 0 is the pop loop of the pending-event queue in ScreenView::render: a deque is drained by `mov rdx,[rax+rcx*8]; mov rcx,[<view>+..]; mov rax,[rcx]; mov rax,[rax+18h]; call [guard]` (a virtual call of the controller with the event), then the front element is released (`mov rcx,[rax+40h]; test rcx,rcx; je; lock dec dword ptr [rcx+0Ch]; jne; mov rax,[rcx]; mov rax,[rax+8]; call [guard]; jmp`), and the first pattern ends at the `cmp qword ptr [<view>+..],0` of the next queue. The function has two identical loops over two queues; the pattern ends with the compare of the next queue, so only the first loop matches. It reads the displacement of the load of the controller. Recipe 1 is the call before the visual tree is touched: `mov rcx,[<view>+..]; mov rax,[rcx]; mov rax,[rax+10h]; call [guard]; mov rcx,[<view>+..]; mov rdx,[rcx]; mov r8,[rdx+20h]; mov edx,eax; mov rax,r8; call [guard]` (a virtual call of the controller, then a virtual call of the object at +50h, the visual tree, with the result), read as the displacement of the first load. The two recipes are two blocks of ScreenView::render, in different loops, and share no pattern bytes. Every register, displacement, immediate and branch byte is a wildcard. The range check is a window around the value. Value 40h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). Evidence: recipe 1 reads the same displacement at the point where the code reads mVisualTree (50h, the entry of the same name) from the same view pointer, and the member sits between mPointerButtonIds (38h) and mVisualTree (50h) as in the header member order."

  recipe {
    step "pattern" {
      bytes  = "48 8B ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 FF C8 48 23 ?? ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 8B 04 ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 49 ?? 75 ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? E9 ?? ?? ?? ?? 48 83 ?? ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B ?? ?? 48 8B 11 4C 8B 42 ?? 89 C2 4C 89 C0 FF 15"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 96
  }
}
