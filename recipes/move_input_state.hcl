entry "MoveInputState.mAnalogMoveVector" {
  kind  = "field"
  notes = "The offset of MoveInputState::mAnalogMoveVector (Vec2, x left, y forward) inside MoveInputState: 4 on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62). When it is not zero, the move-vector function (0x493020 on 26.63) uses it (normalised only when its length is above 1) and ignores the direction flags; when it is zero, the vector comes from the flag bits. Recipe 0 is the first float store of the state copy in ClientInputUpdateSystem.extractRawHIDInput (RawMoveInputComponent.mInputState is 0 there, so the displacement is the offset in the state). Recipe 1 is the read in the per-entity input body of the tick (0x6d68300 on 26.63) after the jump test (`test byte ptr [r14],imm; je; or byte ptr [r14+..],imm; test r15d,imm; mov rdx,[rsp+..]; mov rdi,[rsp+..]; jne; movsd xmm0,[r14+..]`), r14 the MoveInputComponent whose mInputState is at +0. The two are in two functions and share no bytes; both derived on 26.62 and 26.63. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the state copy in extractRawHIDInput stores through r9 (`mov [r9],r8d`, `movss [r9+4],xmm0`, REX 45/41 instead of 44), and the per-entity input body keeps r13 for the component and loads a different register pair around the jump test, so both recipes are a pattern_any with one alternative for each shape; recipe 1 pads the old alternative with three leading wildcards so that the movsd is at offset 36 in both. Evidence: (1) the old value 4 is the first float store of the raw-state copy (recipe 0 anchors on the repaired extractRawHIDInput) and the same displacement is the movsd read after the jump test in the per-entity body, which also holds the sneak and jump tests, in two separate functions; (2) on 70.26 the body that recipe 1 matches calls ClientInputLockComponent.applyToMoveInputState, and the movsd read is followed by the same zero test (`xorps; ucomiss; shufps 55h`) and normalise code (`mulps; sqrtss; divps`) as on 26.63. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "ref" { entry = "ClientInputUpdateSystem.extractRawHIDInput" }
    step "pattern_any" {
      patterns = [
        "44 8B ?? ?? 44 89 ?? F3 0F 10 ?? ?? F3 0F 11 ?? ??",
        "44 8B ?? ?? 45 89 ?? F3 0F 10 ?? ?? F3 41 0F 11 ?? ??",
      ]
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? 41 F6 06 ?? 74 ?? 41 80 4E ?? ?? 41 F7 C7 ?? ?? ?? ?? 48 8B 94 24 ?? ?? ?? ?? 48 8B 7C 24 ?? 75 ?? F2 41 0F 10 46 ??",
        "41 F6 45 ?? ?? 74 ?? 41 80 4D ?? ?? 4C 8B 8C 24 ?? ?? ?? ?? F7 C3 ?? ?? ?? ?? 4C 8B B4 24 ?? ?? ?? ?? 75 ?? F2 41 0F 10 45 ??",
      ]
      offset = 36
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4
    max = 12
  }
}

entry "MoveInputState.maskJumpDown" {
  kind  = "field"
  notes = "The bit mask of the JumpDown flag (held jump) in the u32 flag bits at MoveInputState+0: 80h (bit 7) on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), the LeviLamina MoveInputState::Flag::JumpDown. The input system reads only this held bit for a jump; in game (26.63) holding it jumps in survival and creative. Recipe 0 is the jump test in the per-entity input body of the tick (0x6d68300 on 26.63: `test byte ptr [r14],imm8; je; or byte ptr [r14+..],imm; test r15d,imm32; mov rdx,[rsp+..]`), the mask read as the immediate of the test. One recipe and a range check: the mask is tested alone at no other site of the exe (a scan of `test byte ptr [reg],80h` with this shape finds one match; the other JumpDown test, `test eax,20080h` in the same body, adds Ascend, and the input-lock leaf clears it inside a wider mask). Every other immediate, displacement and branch byte is a wildcard. 26.60 and the 26.70 preview were not derived (no exe on the machine). What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the test is now `test byte ptr [r13+0],imm8; je; or byte ptr [r13+..],..; mov r9,[rsp+..]; test ebx,imm32` (the register moved and a load moved between the `or` and the `test`), so the recipe is a pattern_any with one alternative for each shape. Evidence: (1) it sits in the per-entity input body that calls ClientInputLockComponent.applyToMoveInputState (the first of its two callers on both builds), and no other `test byte ptr [reg],..` of this shape is in the exe; (2) the same body holds the sneak test `test reg,40001h` after the sprint shift in both builds. Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "41 F6 06 ?? 74 ?? 41 80 4E ?? ?? 41 F7 C7 ?? ?? ?? ?? 48 8B 94 24",
        "41 F6 45 ?? ?? 74 ?? 41 80 4D ?? ?? 4C 8B 8C 24 ?? ?? ?? ?? F7 C3 ?? ?? ?? ??",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 2147483647
  }
}

entry "MoveInputState.maskSneakTest" {
  kind  = "field"
  notes = "The mask the game tests for the sneak state in the u32 flag bits at MoveInputState+0: 40001h on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62): bit 0 SneakDown and bit 18 Descend (LeviLamina Flag::SneakDown and Flag::Descend). A writer sets only bit 0 for sneak; in game (26.63) bit 0 alone sneaks. Recipe 0 is in the per-entity input body of the tick (0x6d68300 on 26.63) after the copy of the raw state into MoveInputComponent (`mov eax,[rdx+..]; mov [rsp+..],eax; and eax,imm; mov ecx,r15d; shr ecx,imm; and ecx,imm; or ecx,eax; mov eax,ecx; and eax,imm; or ecx,imm; test r15d,imm32; cmove ecx,eax; mov word ptr [rdx+..],cx`). Recipe 1 goes to the move-vector function through the call at the end of ClientInputUpdateSystem.extractRawHIDInput (0x493020 on 26.63) and reads the sneak-speed test there (`test dword ptr [rdx],imm32; jne; mov rcx,[r9]; test cl,imm; jne`). The two are in two functions and share no bytes; both derived on 26.62 and 26.63. Every immediate, displacement and branch byte is a wildcard. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): in the per-entity body the shift and mask code uses other registers (`mov ecx,ebx; shr ecx,imm; ... test ebx,imm32`, no REX 41 prefix on the test), so recipe 0 is a pattern_any with one alternative for each shape (the new one has two leading wildcards so that the test opcode is at offset 30 in both). In extractRawHIDInput the lookups of the components are now calls, so `call nth 0` is no longer the move-vector function; recipe 1 now reads the branch target of the move-vector call (`lea rcx,[rsp+..]; mov rdx,rsi; call`, with `mov r9,rdi` between the `mov` and the call on 70.26, an added argument; the old alternative has three leading wildcards so that the call opcode is at offset 11 in both) and then reads the same sneak-speed test in that function, whose body is unchanged. Evidence: (1) the test mask is the one after the sprint shift of the per-entity body that calls ClientInputLockComponent.applyToMoveInputState, the same code that holds the repaired maskJumpDown and bitSprintDown; (2) the move-vector function reached from extractRawHIDInput holds `test dword ptr [rdx],imm32; jne; mov rcx,[r9]; test cl,imm; jne` once, the same as on 26.63, and its first 25 instructions are byte-identical to the 26.63 function except for the displacements. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = [
        "8B 42 ?? 89 44 24 ?? 83 E0 ?? 44 89 F9 C1 E9 ?? 83 E1 ?? 09 C1 89 C8 83 E0 ?? 83 C9 ?? 41 F7 C7 ?? ?? ?? ?? 0F 44 C8 66 89 4A",
        "?? ?? 8B 46 ?? 89 44 24 ?? 83 E0 ?? 89 D9 C1 E9 ?? 83 E1 ?? 09 C1 89 C8 83 E0 ?? 83 C9 ?? F7 C3 ?? ?? ?? ?? 0F 44 C8 66 89 4E ??",
      ]
      offset = 30
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "ref" { entry = "ClientInputUpdateSystem.extractRawHIDInput" }
    step "pattern_any" {
      patterns = [
        "?? ?? ?? 48 8D 4C 24 ?? 48 89 F2 E8",
        "48 8D 4C 24 ?? 48 89 F2 49 89 F9 E8",
      ]
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "pattern" {
      bytes = "F7 02 ?? ?? ?? ?? 75 ?? 49 8B 09 F6 C1 ?? 75 ??"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 2147483647
  }
}

entry "MoveInputState.bitSprintDown" {
  kind  = "field"
  notes = "The bit number of the SprintDown flag (held sprint) in the u32 flag bits at MoveInputState+0: 8 on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), LeviLamina Flag::SprintDown. In game (26.63) holding it with a forward move sprints. The per-entity input body of the tick (0x6d68300 on 26.63) moves it to bit 1 of the MoveInputComponent word at +60h with `shr ecx,7; and ecx,2`, so the bit is the shift plus 1. Recipe 0 reads the shift in the same code as MoveInputState.maskSneakTest recipe 0 and adds 1. One recipe and a range check: no other code of the input system tests the bit (the `and ecx,100h` later in that body tests the derived word, not the state). Every immediate, displacement and branch byte is a wildcard. 26.60 and the 26.70 preview were not derived (no exe on the machine). What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the shift sits in the same place, but the registers differ (`mov ecx,ebx; shr ecx,7`), so the recipe is a pattern_any with one alternative for each shape; the new one starts with a wildcard byte so that the shift immediate is at offset 13 in both. Evidence: (1) the pattern is in the per-entity input body that calls ClientInputLockComponent.applyToMoveInputState (first of its two callers on both builds); (2) the shift is followed by `and ecx,2; or ecx,eax; ...; test reg,40001h` as on the old build. Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "8B 42 ?? 89 44 24 ?? 83 E0 ?? 44 89 F9 C1 E9 ?? 83 E1 ?? 09 C1 89 C8 83 E0 ?? 83 C9 ?? 41 F7 C7 ?? ?? ?? ?? 0F 44 C8 66 89 4A",
        "?? 8B 46 ?? 89 44 24 ?? 83 E0 ?? 89 D9 C1 E9 ?? 83 E1 ?? 09 C1 89 C8 83 E0 ?? 83 C9 ?? F7 C3 ?? ?? ?? ?? 0F 44 C8 66 89 4E ??",
      ]
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
    step "add" { value = 1 }
  }

  check "range" {
    min = 1
    max = 31
  }
}

entry "MoveInputState.bitUp" {
  kind  = "field"
  notes = "The bit number of the Up (forward) flag in the u32 flag bits at MoveInputState+0: 13 (mask 2000h) on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), LeviLamina Flag::Up. In game (26.63) the Up bit with a zero analog vector walks forward; the per-entity input body of the tick also tests the mask 2000h (`test r15d,2000h`) before its sprint check. When a direction is locked, the input-lock leaf rewrites the diagonal flags into single directions with `sete r8b/r9b; shl r8d/r9d,bit; or`, so the shift counts are the bit numbers. Recipe 0 is a source pattern that starts at the first byte of ClientInputLockComponent.applyToMoveInputState (0x4932d0 on 26.63, 0x4933f0 on 26.62, a leaf without .pdata; the lint decodes the match because it starts at that entry's value) and runs through the leaf's opening lock tests to the end of the second lock block, past the first block and the `ret` of the no-lock path: the block that works in r10d/r11d and ends with `minss xmm0,[rdx+..]` (it builds Down and Up), read as the immediate of its second shift, `shl r9d,imm8`. One recipe and a range check: the four lock blocks of the leaf are the only code that builds these bits from a count, and a second block of the same leaf would share this recipe's bytes from the leaf start (no independent route). Every immediate, displacement and branch byte is a wildcard; the register bytes and the minss/maxss opcode are kept. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes  = "0F B7 41 ?? 84 C0 0F 88 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 74 ?? 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 84 C0 41 0F 99 C1 8B 02 41 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 A9 ?? ?? ?? ?? 44 0F 44 C8 44 89 C8 25 ?? ?? ?? ?? 41 C1 E0 ?? 41 09 C0 44 89 C8 25 ?? ?? ?? ?? 41 F7 C1 ?? ?? ?? ?? 41 0F 45 C0 89 02 0F 57 C0 F3 0F 5F 42 ?? F3 0F 11 42 ?? 0F B7 41 ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? C3 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 A9 ?? ?? ?? ?? 41 0F 94 C1 44 8B 12 45 89 D3 41 81 E3 ?? ?? ?? ?? 41 C1 E0 ?? 45 09 D8 41 F7 C2 ?? ?? ?? ?? 45 0F 44 C2 45 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 45 89 C2 41 81 E2 ?? ?? ?? ?? 41 F7 C0 ?? ?? ?? ?? 45 0F 45 D1 44 89 12 0F 57 C0 F3 0F 5D 42 ??"
      offset = 241
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 31
  }
}

entry "MoveInputState.bitDown" {
  kind  = "field"
  notes = "The bit number of the Down (back) flag in the u32 flag bits at MoveInputState+0: 14 (mask 4000h) on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), LeviLamina Flag::Down. When a direction is locked, the input-lock leaf rewrites the diagonal flags into single directions with `sete r8b/r9b; shl r8d/r9d,bit; or`, so the shift counts are the bit numbers. Recipe 0 is a source pattern that starts at the first byte of ClientInputLockComponent.applyToMoveInputState (0x4932d0 on 26.63, 0x4933f0 on 26.62, a leaf without .pdata; the lint decodes the match because it starts at that entry's value) and runs through the leaf's opening lock tests to the end of the second lock block, past the first block and the `ret` of the no-lock path: the block that works in r10d/r11d and ends with `minss xmm0,[rdx+..]` (it builds Down and Up), read as the immediate of its first shift, `shl r8d,imm8`. One recipe and a range check: the four lock blocks of the leaf are the only code that builds these bits from a count, and a second block of the same leaf would share this recipe's bytes from the leaf start (no independent route). Every immediate, displacement and branch byte is a wildcard; the register bytes and the minss/maxss opcode are kept. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes  = "0F B7 41 ?? 84 C0 0F 88 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 74 ?? 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 84 C0 41 0F 99 C1 8B 02 41 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 A9 ?? ?? ?? ?? 44 0F 44 C8 44 89 C8 25 ?? ?? ?? ?? 41 C1 E0 ?? 41 09 C0 44 89 C8 25 ?? ?? ?? ?? 41 F7 C1 ?? ?? ?? ?? 41 0F 45 C0 89 02 0F 57 C0 F3 0F 5F 42 ?? F3 0F 11 42 ?? 0F B7 41 ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? A8 ?? 0F 85 ?? ?? ?? ?? C3 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 A9 ?? ?? ?? ?? 41 0F 94 C1 44 8B 12 45 89 D3 41 81 E3 ?? ?? ?? ?? 41 C1 E0 ?? 45 09 D8 41 F7 C2 ?? ?? ?? ?? 45 0F 44 C2 45 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 45 89 C2 41 81 E2 ?? ?? ?? ?? 41 F7 C0 ?? ?? ?? ?? 45 0F 45 D1 44 89 12 0F 57 C0 F3 0F 5D 42 ??"
      offset = 213
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 31
  }
}

entry "MoveInputState.bitLeft" {
  kind  = "field"
  notes = "The bit number of the Left flag in the u32 flag bits at MoveInputState+0: 15 (mask 8000h) on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), LeviLamina Flag::Left. When a direction is locked, the input-lock leaf rewrites the diagonal flags into single directions with `sete r8b/r9b; shl r8d/r9d,bit; or`, so the shift counts are the bit numbers. Recipe 0 is a source pattern that starts at the first byte of ClientInputLockComponent.applyToMoveInputState (0x4932d0 on 26.63, 0x4933f0 on 26.62, a leaf without .pdata; the lint decodes the match because it starts at that entry's value) and runs through the leaf's opening lock tests to the end of the first lock block, the one that tests the sign of the lock byte (`test al,al; setns r9b`), works in eax and ends with `maxss xmm0,[rdx+..]` (it builds Left and Right), read as the immediate of its first shift, `shl r9d,imm8`. One recipe and a range check: the four lock blocks of the leaf are the only code that builds these bits from a count, and a second block of the same leaf would share this recipe's bytes from the leaf start (no independent route). Every immediate, displacement and branch byte is a wildcard; the register bytes and the minss/maxss opcode are kept. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes  = "0F B7 41 ?? 84 C0 0F 88 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 74 ?? 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 84 C0 41 0F 99 C1 8B 02 41 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 A9 ?? ?? ?? ?? 44 0F 44 C8 44 89 C8 25 ?? ?? ?? ?? 41 C1 E0 ?? 41 09 C0 44 89 C8 25 ?? ?? ?? ?? 41 F7 C1 ?? ?? ?? ?? 41 0F 45 C0 89 02 0F 57 C0 F3 0F 5F 42 ??"
      offset = 74
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 31
  }
}

entry "MoveInputState.bitRight" {
  kind  = "field"
  notes = "The bit number of the Right flag in the u32 flag bits at MoveInputState+0: 16 (mask 10000h) on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62), LeviLamina Flag::Right. When a direction is locked, the input-lock leaf rewrites the diagonal flags into single directions with `sete r8b/r9b; shl r8d/r9d,bit; or`, so the shift counts are the bit numbers. Recipe 0 is a source pattern that starts at the first byte of ClientInputLockComponent.applyToMoveInputState (0x4932d0 on 26.63, 0x4933f0 on 26.62, a leaf without .pdata; the lint decodes the match because it starts at that entry's value) and runs through the leaf's opening lock tests to the end of the first lock block, the one that tests the sign of the lock byte (`test al,al; setns r9b`), works in eax and ends with `maxss xmm0,[rdx+..]` (it builds Left and Right), read as the immediate of its second shift, `shl r8d,imm8`. One recipe and a range check: the four lock blocks of the leaf are the only code that builds these bits from a count, and a second block of the same leaf would share this recipe's bytes from the leaf start (no independent route). Every immediate, displacement and branch byte is a wildcard; the register bytes and the minss/maxss opcode are kept. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes  = "0F B7 41 ?? 84 C0 0F 88 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? A9 ?? ?? ?? ?? 74 ?? 45 31 C0 A9 ?? ?? ?? ?? 41 0F 94 C0 45 31 C9 84 C0 41 0F 99 C1 8B 02 41 89 C2 41 81 E2 ?? ?? ?? ?? 41 C1 E1 ?? 45 09 D1 A9 ?? ?? ?? ?? 44 0F 44 C8 44 89 C8 25 ?? ?? ?? ?? 41 C1 E0 ?? 41 09 C0 44 89 C8 25 ?? ?? ?? ?? 41 F7 C1 ?? ?? ?? ?? 41 0F 45 C0 89 02 0F 57 C0 F3 0F 5F 42 ??"
      offset = 98
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 31
  }
}
