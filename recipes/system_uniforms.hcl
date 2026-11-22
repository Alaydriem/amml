entry "SystemUniforms.mNow" {
  kind  = "field"
  notes = "Offset of mNow (a steady_clock time point, a signed 64-bit count of nanoseconds) in PredictedMovementSystem::SystemUniforms: 8 on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). The struct is argument 10 of PredictedMovementSystem.tickPredictedMovement (`mov rsi,[rsp+148h]` on 26.63, `mov rsi,[rsp+168h]` on 70.26: entry rsp+50h after 8 pushes and a frame of B8h / D8h). The system tick wrapper (26.63 0x7b33ff0, 70.26 0x5b6b010) builds the struct on its frame: it stores the result of steady_clock::now() at local+8 (`mov [rbp+58h],rdx` with the local at rbp+50h, 26.63 0x7b34417; `mov [rbp-8],rax` with the local at rbp-10h, 70.26 0x5b6b766), then takes the address of tickPredictedMovement. tickPredictedMovement passes the address of the member to two callees as r8. Recipe 0 is the first of them: `mov rcx,[rdi+..]; lea r8,[rsi+..]; lea rdx,[rsp+..]; call` (26.63 0x7b05022, 70.26 0x5b4a9ca), scanned inside tickPredictedMovement; offset 4, the displacement of operand 1 of the lea. Recipe 1 is the second: `add rsi,..; lea rdx,[rsp+..]; mov rcx,rdi; mov r8,rsi; call` (26.63 0x7b0513c, 70.26 0x5b4ab61); offset 0, the immediate of operand 1 of the add. The two recipes are two call sites with two different callees and share no pattern bytes; both are in tickPredictedMovement, because the only other function that names the struct (the wrapper) builds it on the frame and the distance from the now() store to a lea of the local differs between the builds (99h on 26.63, 16Dh on 70.26), so member_displacement cannot cover both builds there. Every displacement, immediate and branch byte is a wildcard. Range check 0..40h. Class: new entry; the same value on both builds."

  recipe {
    step "ref" { entry = "PredictedMovementSystem.tickPredictedMovement" }
    step "pattern" {
      bytes  = "48 8B 4F ?? 4C 8D 46 ?? 48 8D 54 24 ?? E8"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "PredictedMovementSystem.tickPredictedMovement" }
    step "pattern" { bytes = "48 83 C6 ?? 48 8D 54 24 ?? 48 89 F9 49 89 F0 E8" }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 64
  }
}
