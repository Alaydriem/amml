entry "HIDControllerGameCoreDesktop.onKeyDown" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onKeyDown@HIDControllerGameCoreDesktop@@QEAAXHW4InputOrigin@KeyboardEventProcessor@Input@Bedrock@@@Z"
  notes     = "void HIDControllerGameCoreDesktop::onKeyDown(int keyCode, KeyboardEventProcessor::InputOrigin origin), the KeyboardEventProcessor::onKeyDown override: slot 1 of the HIDControllerGameCoreDesktop.vtable global in both builds (destructor, onKeyDown, onKeyUp, then HIDController's update .. setIMEEnabled). LeviLamina raises KeyInputEvent from a hook on this function. The exe has no RTTI for game classes; the vtable was found through the HIDController constructor (see HIDControllerGameCoreDesktop.vtable). The function runs a CoreTextEdit block only when origin is Hardware and then calls HIDController::onKeyDown with the same three arguments. Recipe 0 is the prologue (four pushes, frame setup, `cmp r8d,imm; jne`, `mov rsi,rcx; mov edi,edx; mov ebx,r8d` and the following `lock inc`). Recipe 1 shares no bytes with recipe 0: the call to the base and the epilogue (`mov r8d,ebx; mov edx,edi; mov rcx,rsi; call; nop; add rsp,..; pop rbx/rdi/rsi/rbp; ret`), then `function_start_of`. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. The checks are `function_start` and `in_vtable` of HIDControllerGameCoreDesktop.vtable, slots 0 to 1 (it leaves out onKeyUp in slot 2). The old third recipe (the vtable route with a fixed `vtable_slot 1`) is replaced by that check. The entry did not break in the 26.60 cold derive. Refused on 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23), a real code change: that build has no HIDController. Key input goes through a window-message handler (a slot of another vtable) that tests WM_KEYDOWN/WM_SYSKEYDOWN against WM_KEYUP/WM_SYSKEYUP, runs the text-input code (the old CoreTextEdit block of onKeyDown is now a separate function without arguments, and the edit-text part of onKeyUp is a separate function called with the key) and then calls `void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int key, int state)` (its __FUNCSIG__ text is in the exe) with state 1 or 0; that function holds the key-state table and event queue code of the 26.63 HIDController::onKeyDown and onKeyUp. No function on the Preview has the onKeyDown(int, InputOrigin) or onKeyUp(int) shape, the HIDController std RTTI is gone, and the 26.63 constructor pattern of the vtable entry finds no match. Every recipe finds no match there, so the entry refuses; a hook on the new path needs new entry names and a mod change."

  recipe {
    step "pattern" {
      bytes = "55 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 41 83 F8 ?? 0F 85 ?? ?? ?? ?? 48 89 CE 89 D7 44 89 C3 F0 48 FF 05"
    }
  }

  recipe {
    step "pattern" {
      bytes = "41 89 D8 89 FA 48 89 F1 E8 ?? ?? ?? ?? 90 48 83 C4 ?? 5B 5F 5E 5D C3"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "HIDControllerGameCoreDesktop.vtable"
    min_slot = 0
    max_slot = 1
  }
}

entry "HIDControllerGameCoreDesktop.onKeyUp" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onKeyUp@HIDControllerGameCoreDesktop@@QEAAXH@Z"
  notes     = "void HIDControllerGameCoreDesktop::onKeyUp(int keyCode), the KeyboardEventProcessor::onKeyUp override: slot 2 of the HIDControllerGameCoreDesktop.vtable global in both builds. LeviLamina raises KeyInputEvent from a hook on this function. It copies the edit text (the std::string at this+18h), calls HIDController::onKeyUp, and when the text box is selected calls _notifyCoreTextEdit. Slot 4 (updateEditContext(int)) has the same code byte for byte except the base function it calls, so no pattern inside the body can tell the two apart. The `in_vtable` check (HIDControllerGameCoreDesktop.vtable, slots 2 to 3) is what tells this function apart from the identical slot-4 function, and it also excludes onKeyDown in slot 1; `function_start` is the other check. The entry has a single recipe, and that recipe has to find exactly one place on its own: it anchors on the code just before the function. onKeyDown (the function before it) has two unwind funclets, the `lock dec` of its CoreTextEdit counter and the release of its winrt pointer at [rbp-8], and this function starts right after the int3 padding that follows them. The pattern runs from the first funclet's `lock dec` through the alignment nops (`66 66 66 66 66 2E 0F 1F 84 00 ..`), the second funclet and the five int3 to the eight pushes of this function's prologue and the REX byte of its `sub rsp`; it gives the start 0x4F bytes after its own start. One byte is kept: the 40h of the second funclet's `lea rbp,[rdx+40h]` (onKeyDown's frame base), because another function with the same two funclets uses 70h. The alignment nops and the int3 count are literal (they are not immediate or branch fields). Two more steps check the body: the function that holds the start must contain `mov rdi,rcx; lea r14,[rcx+..]; xorps xmm0,xmm0` (the copy of the edit text), then `function_start_of` gives its start. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60) apart from the kept byte. This funclet/padding anchor is the recipe most likely to break in a later build: other alignment, a changed onKeyDown frame or reordered funclets make it fail (a refusal, not a wrong value), and then the entry has no recipe left. The old second recipe (the vtable route with a fixed `vtable_slot 2`) is replaced by the `in_vtable` check. What changed in 26.60: the funclet's `je` is +0Ah instead of +09h and this function's frame is smaller; both are wildcarded now. Refused on 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23), a real code change: that build has no HIDController. Key input goes through a window-message handler (a slot of another vtable) that tests WM_KEYDOWN/WM_SYSKEYDOWN against WM_KEYUP/WM_SYSKEYUP, runs the text-input code (the old CoreTextEdit block of onKeyDown is now a separate function without arguments, and the edit-text part of onKeyUp is a separate function called with the key) and then calls `void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int key, int state)` (its __FUNCSIG__ text is in the exe) with state 1 or 0; that function holds the key-state table and event queue code of the 26.63 HIDController::onKeyDown and onKeyUp. No function on the Preview has the onKeyDown(int, InputOrigin) or onKeyUp(int) shape, the HIDController std RTTI is gone, and the 26.63 constructor pattern of the vtable entry finds no match. Every recipe finds no match there, so the entry refuses; a hook on the new path needs new entry names and a mod change."

  recipe {
    step "pattern" {
      bytes      = "F0 48 FF 0D ?? ?? ?? ?? 48 83 C4 ?? 5B 5F 5E 5D C3 66 66 66 66 66 2E 0F 1F 84 00 ?? ?? ?? ?? 48 89 54 24 ?? 55 56 57 53 48 83 EC ?? 48 8D 6A 40 48 83 7D ?? ?? 74 ?? 48 8D 4D ?? E8 ?? ?? ?? ?? 90 48 83 C4 ?? 5B 5F 5E 5D C3 CC CC CC CC CC 55 41 57 41 56 41 55 41 54 56 57 53 48"
      offset     = 79
      literal_ok = [
        "47: lea rbp,[rdx+40h] of onKeyDown's second funclet: on 26.51 another function with the same two funclets uses 70h (26.60 is unique without this byte), so this byte keeps the match after onKeyDown",
      ]
    }
    step "pattern" {
      bytes = "48 89 CF 4C 8D 71 ?? 0F 57 C0"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "HIDControllerGameCoreDesktop.vtable"
    min_slot = 2
    max_slot = 3
  }
}

entry "HIDControllerGameCoreDesktop.vtable" {
  kind     = "global"
  notes    = "The vtable at this+0 of HIDControllerGameCoreDesktop (KeyboardEventProcessor and HIDController virtuals: destructor, onKeyDown, onKeyUp, then HIDController's update .. setIMEEnabled). Found through the HIDController constructor: the std type `_Binder<..., HIDController::*(ClipboardPaste const&), ...>` has RTTI, its std::function impl's _Target_type returns it, and the impl's vtable is loaded by one function, the HIDController constructor. That constructor has one caller, which allocates 120h bytes, calls it, then stores the HWND at this+0C8h, this vtable at this+0, clears mCallbacksRegistered at +0D0h and the event tokens from +0D8h (three `movups` of a zeroed xmm and a qword store), and stores mCoreTextEdit at +110h: the HIDControllerGameCoreDesktop layout. The pattern is that whole sequence from the reload of `this` (`mov reg,[rbp+..]; mov [reg+..],rax; lea rax,[rip+..]; mov [reg],rax; mov byte ptr [reg+..],imm; xorps; movups x3; mov qword ptr [reg+..],imm; mov rcx,[rbp+..]; mov [reg+..],rcx`) with every REX, ModRM, displacement and immediate byte wildcarded, so the registers are free; it is unique in both exes, and offset 14 gives the `lea`, whose RIP target is the operand. The section check keeps the address inside .rdata. It exists for the `in_vtable` checks of onKeyDown and onKeyUp; it was added in Plan 5b. Values: 0xe76d050 in 26.51, 0xf08ff60 in 26.60. Refused on 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23), a real code change: that build has no HIDController. Key input goes through a window-message handler (a slot of another vtable) that tests WM_KEYDOWN/WM_SYSKEYDOWN against WM_KEYUP/WM_SYSKEYUP, runs the text-input code (the old CoreTextEdit block of onKeyDown is now a separate function without arguments, and the edit-text part of onKeyUp is a separate function called with the key) and then calls `void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int key, int state)` (its __FUNCSIG__ text is in the exe) with state 1 or 0; that function holds the key-state table and event queue code of the 26.63 HIDController::onKeyDown and onKeyUp. No function on the Preview has the onKeyDown(int, InputOrigin) or onKeyUp(int) shape, the HIDController std RTTI is gone, and the 26.63 constructor pattern of the vtable entry finds no match. Every recipe finds no match there, so the entry refuses; a hook on the new path needs new entry names and a mod change."

  recipe {
    step "pattern" {
      bytes  = "?? 8B ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? C6 ?? ?? ?? ?? ?? ?? 0F 57 ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? 0F 11 ?? ?? ?? ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 8B ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ??"
      offset = 14
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
