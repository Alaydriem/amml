entry "ClientInputLockComponent.applyToMoveInputState" {
  kind  = "function"
  notes = "A static leaf that applies the ClientInputLockComponent to a MoveInputState: rcx the lock component (it reads the lock bits as a word at +2), rdx the MoveInputState (flag bits at +0, analog vector at +4), void. The name is ours; the exe has no symbol for it. 0x4932d0 on 6aed1906-b127418e74d2f2f9 (26.63) and 0x4933f0 on 6aea437e-7eff13daabb430da (26.62), a leaf without .pdata (it starts after int3 padding and ends with a ret). When a direction is locked it rewrites the diagonal flags into single directions (`sete r8b/r9b; shl r8d/r9d,bit; or`, the shift counts are the bit numbers of Up, Down, Left and Right) and clamps the analog vector with minss/maxss; when movement or jump is locked it clears the matching flag bits. Found from the per-entity input body of the client input tick (0x6d68300 on 26.63), which calls it on a copy of RawMoveInputComponent.mInputState before it stores the state into MoveInputComponent. The entry exists so that the MoveInputState bit entries can anchor at its first byte (the lint decodes a match in a leaf only from a proven code start). Recipe 0 is that call (`mov [rsp+..],rdi; movups xmm0,[rbp+..]; movaps [rsp+..],xmm0; lea rdx,[rsp+..]; mov rcx,r15; call`), read as a branch target. Recipe 1 is the call in a second input function (0xa1f93a0 on 26.63), right after a call to the movement-start bookkeeping function (`mov rdx,rbx; mov r8,rax; mov r9,r14; call; mov rcx,r14; mov rdx,rsi; call`), read as a branch target. The two are in two functions and share no bytes; both derived on 26.62 and 26.63. Every displacement and branch byte is a wildcard; both patterns end at the call. leaf_start confirms the start. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes  = "48 89 7C 24 ?? 0F 10 45 ?? 0F 29 44 24 ?? 48 8D 54 24 ?? 4C 89 F9 E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 DA 49 89 C0 4D 89 F1 E8 ?? ?? ?? ?? 4C 89 F1 48 89 F2 E8"
      offset = 20
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}
