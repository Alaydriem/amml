entry "UIScene.vtable" {
  kind  = "global"
  notes = "The vtable of UIScene (no RTTI: the complete-object-locator qword before it is 0), a helper for the in_vtable window of UIScene.handleDirection. It follows the AbstractScene order: slot 19 is UIScene::render (the only direct caller of ScreenView.render), slot 26 the folded `ret` of handlePointerPressed, slot 27 handleDirection. The only code that names it is the UIScene constructor, which stores it at [rcx] and then copies the shared_ptr arguments into the object (the constructor also allocates the 4B8h-byte ScreenView next). Recipe 0 is that run: `lea rax,[rip+..]; mov [rcx],rax; xorps xmm0,xmm0; movups [rcx+..],xmm0; mov rax,[rdx+..]; test rax,rax; je; lock inc dword ptr [rax+..]; lea rax,[rcx+..]; mov [rbp+..],rax; mov rax,[rdi]; mov [rcx+..],rax; mov rax,[rdi+..]; mov [rcx+..],rax; mov rax,[rdi+..]`, read as the RIP target of the lea. The first 24 bytes alone have 18 matches; the full run has one. Every displacement, immediate and branch byte is a wildcard. The constructor is the only code reference, so there is no second recipe: the in_vtable check of UIScene.handleDirection (slot 27, found by two body anchors of its own) ties this value to the class. The check is section .rdata. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value F20EC60h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 01 0F 57 C0 0F 11 41 ?? 48 8B 42 ?? 48 85 C0 74 ?? F0 FF 40 ?? 48 8D 41 ?? 48 89 85 ?? ?? ?? ?? 48 8B 07 48 89 41 ?? 48 8B 47 ?? 48 89 41 ?? 48 8B 47 ??" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "UIScene.handleDirection" {
  kind     = "function"
  hooked   = true
  notes    = "void UIScene::handleDirection(DirectionId, float x, float y, FocusImpact): rcx the UIScene, dl the DirectionId (u8; Scroll = 2), xmm2 x and xmm3 y (f32), the FocusImpact (u8) on the stack at [rsp+28h]; returns void. Slot 27 of UIScene.vtable (the AbstractScene::handleDirection override); no other table holds it and no direct call reaches it. It loads the ScreenView at +40h, runs an optional focus block when the FocusImpact is not 0, then branches on the DirectionId. For Scroll it stores x and y in the ScreenView (+2B8h, +2BCh) and calls ScreenView::_processControllerDirection(view, 2), which turns the wheel step into the scroll of the control under the pointer and the controller events; a hook that returns without the original drops that one wheel step and nothing else. Recipe 0 is that Scroll branch: `movss [rbx+..],xmm7; movss [rbx+..],xmm6; mov rcx,rbx; mov dl,2; call`, then function_start_of. Recipe 1 is the head of the body after the prologue: `movaps xmm6,xmm3; movaps xmm7,xmm2; movzx eax,byte ptr [rbp+..]; mov rbx,[rcx+..]; test al,al; mov [rbp+..],rbx; je`, then function_start_of. The two anchors are in different parts of the body and share no pattern bytes; both end in function_start_of in one function, so the in_vtable window is the independent fact. The prologue (eight pushes, `sub rsp,2B8h`) has 739 matches and is not an anchor. Every displacement and branch byte is a wildcard; one immediate is kept (literal_ok): the 2 of `mov dl,2`, the DirectionId::Scroll that names the branch. The checks are function_start and in_vtable on UIScene.vtable, window 23..31 (slot 27). The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 4CD0570h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes      = "F3 0F 11 BB ?? ?? ?? ?? F3 0F 11 B3 ?? ?? ?? ?? 48 89 D9 B2 02 E8"
      literal_ok = [
        "20: mov dl,2: DirectionId::Scroll, the branch this anchor names",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "0F 28 F3 0F 28 FA 0F B6 85 ?? ?? ?? ?? 48 8B 59 ?? 84 C0 48 89 9D ?? ?? ?? ?? 0F 84" }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "UIScene.vtable"
    min_slot = 23
    max_slot = 31
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "UIScene.mScreenView" {
  kind     = "field"
  notes    = "Offset in UIScene of the ScreenView pointer (the object that ScreenView.render draws): 40h on 26.60. UIScene::render passes it as `this` to ScreenView::render, and UIScene::handleDirection loads it first. Recipe 0 is inside UIScene.handleDirection: `movzx eax,byte ptr [rbp+..]; mov rbx,[rcx+..]; test al,al`, read from the displacement of the load of rbx. Recipe 1 is the call site of ScreenView::render in UIScene::render: the prologue of UIScene::render (found by its own unique prologue, a stack-protector load and the thread-local guard test, as in recipe 1 of ScreenView.render), then inside it `mov r9,rbx; call <context ctor>; nop; mov rcx,[rbx+..]; lea rdx,[rbp+..]; call`, read from the displacement of the load of rcx. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check is a wide window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 40h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "ref" { entry = "UIScene.handleDirection" }
    step "pattern" {
      bytes  = "0F B6 85 ?? ?? ?? ?? 48 8B 59 ?? 84 C0"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" { bytes = "55 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 D7 48 89 CB 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ??" }
    step "pattern" {
      bytes  = "49 89 D9 E8 ?? ?? ?? ?? 90 48 8B 4B ?? 48 8D 55 ?? E8"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 512
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
