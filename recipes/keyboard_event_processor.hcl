entry "KeyboardEventProcessor.feedKeyState" {
  kind      = "function"
  hooked    = true
  notes     = "void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int key, int state): the keyboard key-state feed of the 70.x builds. rcx is `this` but the body never reads it (the callers pass whatever rcx holds), edx is the virtual-key code, r8d the state (1 down, 0 up). A key of 100h or more goes to an assert (the function's __FUNCSIG__ text and the KeyboardEventProcessor.cpp path) and returns. Else it compares and stores the state in the 256-entry int key table (`changed` = old state equal to the new one), then pushes a 1Ch-byte record {state, key byte, four globals, changed byte} onto a global event vector (`add qword ptr [end],1Ch`, or a vector grow). It is the replacement of HIDControllerGameCoreDesktop.onKeyDown and .onKeyUp, which do not exist on 70.x: in 26.63 the same table and record code sat inline in HIDController::onKeyDown (state 1) and HIDController::onKeyUp (state 0), the base functions that those two overrides call. Callers on 70.26: the key-message handler of the window (WM_KEYDOWN / WM_SYSKEYDOWN give state 1 after the text-input notify, the other key messages state 0), and three focus or reset paths that loop key 0..0FFh with state 0. A hook here sees every key down and key up that reaches the game's key state, and can drop one by not calling the original; a call of the original with (any, key, 1 or 0) injects a key with no controller pointer. Recipe 0: the function that loads its own __FUNCSIG__ text `void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int, int)` (one function loads it). Recipe 1 shares no bytes with recipe 0 and anchors in the key-message handler: the key-down tail `mov edx,ebx; mov r8d,imm; jmp` followed by the key-up text-input call (`add rdi,imm; mov rcx,rdi; mov edx,ebx; call`); offset 8 is the `jmp`, its branch target is the shared `call` of the feed, and that call's branch target is the value. Every immediate, displacement and branch byte is a wildcard; the pattern ends at a call. Checks: `function_start`, and `entry_is LevelChunk.mSubChunks = 552` (the 70.x layout): 26.63 has no such function (no __FUNCSIG__ text, the code is inline in two functions), so the entry is for 70.x builds only. Evidence for 6afa4e47-dadc375797a3d31a: (1) the record push drafted from the 26.63 HIDController::onKeyDown body (`mov byte ptr [reg+18h],reg8; add qword ptr [rip+..],1Ch`) matches exactly the two 26.63 base functions HIDController::onKeyDown and ::onKeyUp, and exactly one function on 70.26, this one; the 26.63 bound `cmp key,0FFh; ja` and the table compare are here as `cmp edx,100h; jb` and `cmp [table+key*4],r8d; sete`. (2) In 26.63 HIDControllerGameCoreDesktop::onKeyDown / onKeyUp (the KeyboardEventProcessor::onKeyDown / onKeyUp overrides) pass the key to those base functions, onKeyDown after a text-input block for hardware keys; on 70.26 the key-message handler that tests the WM_KEYDOWN / WM_SYSKEYDOWN pair calls this function with state 1 and 0, and its own __FUNCSIG__ text names it a KeyboardEventProcessor member. What changed in 6afa4e47-dadc375797a3d31a: real code change (HIDController removed, key input moved into the window-message handler and this feed)."

  recipe {
    step "string_xref" {
      text = "void Bedrock::Input::KeyboardEventProcessor::_feedKeyState(int, int)"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "89 DA 41 B8 ?? ?? ?? ?? EB ?? 48 83 C7 ?? 48 89 F9 89 DA E8"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
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
