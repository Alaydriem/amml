entry "ClientExitLevel.hook" {
  kind      = "function"
  hooked    = true
  msvc_name = "?$onLevelExit@ClientInstance@@QEAAXXZ"
  notes     = "void ClientInstance::onLevelExit(), the GameCallbacks::onLevelExit override: slot 7 of the ClientInstance.GameCallbacks.vtable global in both builds. LeviLamina raises ClientExitLevelEvent from a hook on this function, and LeviLamina.dll imports this name. The body moves `this` back by 88h (`lea rsi,[rcx-88h]`) to the ClientInstance and calls ClientInstance virtuals on it. No direct call reaches it. Recipe 0 is the prologue up to the first virtual call. Recipe 1 shares no bytes with it: the last virtual call and the epilogue (`... mov rdx,rsi; xor r8d,r8d; call [guard]; nop; add rsp,..; pop rbx/rdi/rsi/rbp; ret`), 0xF9 bytes in on 26.51 and 0xF6 on 26.60, then `function_start_of`. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both are unique in both exes and agree. The checks are `function_start` and `in_vtable` of ClientInstance.GameCallbacks.vtable, slots 3 to 9 (the window ends at updateScreens, the last of GameCallbacks' ten virtuals); the check replaces the old vtable route with a fixed `vtable_slot 7`. What changed in 26.60: the body is 3 bytes shorter."

  recipe {
    step "pattern" {
      bytes = "55 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CF 48 8D B1 ?? ?? ?? ?? C7 45 ?? ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 8B 80 ?? ?? ?? ?? 48 8D 55"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 89 F2 45 31 C0 FF 15 ?? ?? ?? ?? 90 48 83 C4 ?? 5B 5F 5E 5D C3"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ClientInstance.GameCallbacks.vtable"
    min_slot = 3
    max_slot = 9
  }
}
