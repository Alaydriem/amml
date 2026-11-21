entry "HIDController.resetKeys" {
  kind   = "function"
  hooked = true
  notes  = "void HIDController::resetKeys(), this report's name (the exe has no symbol): the HIDController key reset. It takes only `this` (rcx), sets each of the 256 entries of the HIDController key table to 0, pushes the same 1Ch-byte key-up record that the base HIDController::onKeyUp pushes for each key, and at the end clears a byte member of `this`. It does not call onKeyUp, so a hook on HIDControllerGameCoreDesktop.onKeyUp does not see these ups. Its only caller is a tail jump from the small function in slot 14 of the HIDControllerGameCoreDesktop.vtable global; the game window's focus handler calls that slot on WM_KILLFOCUS and on a deactivate. The key table has three writers: the base onKeyDown, the base onKeyUp and this function (`inspect xrefs` on the table). The loader hooks it so that it can release the keys it holds down before the game resets them. Recipe 0 is the loop step and test: `inc r12; add r15,imm8; cmp r12,imm32; je rel32; cmp dword ptr [r15],imm8; sete [rsp+..]; sete al`, then `function_start_of`. Recipe 1 shares no bytes with recipe 0: the end of the function, `mov rax,[rsp+..]; mov byte ptr [rax+..],imm8` (the member clear), the stack cookie check (`mov rax,[rsp+..]; xor rax,rsp; mov rcx,[rip+..]; cmp rcx,rax; jne`) and the epilogue (`add rsp,..`, eight pops, `ret`), then `function_start_of`. Neither pattern runs past a call. Every immediate, displacement and branch byte of both patterns is a wildcard. Both are unique on 26.63 and agree; both find no match on 70.26. The slot-14 function is not used as a recipe: its body (`push rsi; sub rsp; cmp byte ptr [rcx+..],imm; jne; mov rsi,rcx; call`) matches about 100 functions, a longer pattern would run past its call, and a fixed `vtable_slot` route is a lint finding on a function entry. Both recipes are body anchors in the same function; a changed loop or a changed frame breaks one of them and refuses the entry, it does not give a wrong value. Checks: `function_start` (both recipes end with `function_start_of`, so the second agreeing recipe is what the deriver counts) and the layout gate. Layout gate: 26.63 only (`entry_is LevelChunk.mSubChunks = 456`, as the HIDControllerGameCoreDesktop entries); on 70.x HIDController is removed and the focus handler calls KeyboardEventProcessor._feedKeyState(key, 0) for each key, which is the hooked KeyboardEventProcessor.feedKeyState."

  recipe {
    step "pattern" {
      bytes = "49 FF C4 49 83 C7 ?? 49 81 FC ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 83 3F ?? 0F 94 44 24 ?? 0F 94 C0"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 44 24 ?? C6 40 ?? ?? 48 8B 44 24 ?? 48 31 E0 48 8B 0D ?? ?? ?? ?? 48 39 C1 75 ?? 48 83 C4 ?? 5B 5D 5F 5E 41 5C 41 5D 41 5E 41 5F C3"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
