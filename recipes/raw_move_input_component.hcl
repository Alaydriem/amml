entry "RawMoveInputComponent.mInputState" {
  kind  = "field"
  notes = "The offset of RawMoveInputComponent::mInputState (a MoveInputState, 0Fh bytes: u32 flag bits at +0, Vec2 mAnalogMoveVector at +4, three look bytes at +0Ch) in the 18h-byte component (the storage stride in the tick is 18h): 0 on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62). Recipe 0 reads the destination of the first store of the state copy in ClientInputUpdateSystem.extractRawHIDInput (`mov r10d,[rdx+..]; mov [rsi],r10d; movss`; the store has no displacement, so the value is 0). Recipe 1 is in the per-entity input body that runs later in the same tick (0x6d68300 on 26.63, 0x6d685a0 on 26.62), which reads the raw state when its context gate returns true: `test al,al; je; mov [rsp+..],rdi; movups xmm0,[rbp+..]; movaps [rsp+..],xmm0; lea rdx,[rsp+..]; mov rcx,r15; call` (rbp the RawMoveInputComponent, the call the input-lock leaf), read as the displacement of the movups; it ends at the call. The two are in two functions and share no bytes; both derived on 26.62 and 26.63. Every displacement and branch byte is a wildcard. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "ref" { entry = "ClientInputUpdateSystem.extractRawHIDInput" }
    step "pattern" {
      bytes  = "44 8B ?? ?? 44 89 ?? F3 0F 10"
      offset = 4
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "84 C0 0F 84 ?? ?? ?? ?? 48 89 7C 24 ?? 0F 10 45 ?? 0F 29 44 24 ?? 48 8D 54 24 ?? 4C 89 F9 E8"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 16
  }
}

entry "RawMoveInputComponent.mRawMove" {
  kind  = "field"
  notes = "The offset of RawMoveInputComponent::mRawMove (Vec2, x = left minus right, y = forward minus back, already scaled by the sneak factor): 10h on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62). Only the next system callback of the tick reads it (movement-start bookkeeping); the per-entity input body computes the move vector again from mInputState, so a writer sets both. Recipe 0 is the store after the call to the move-vector function at the end of ClientInputUpdateSystem.extractRawHIDInput (`lea rcx,[rsp+..]; mov rdx,rsi; call; movsd xmm0,[rsp+..]; movsd [rsi+..],xmm0`), read as a displacement. Recipe 1 is that next system callback (0x6d51370 on 26.63; its view is RawMoveInputComponent, ActorDataFlagComponent, ClientInputLockComponent and others), which passes the vector by value (`mov r9,[rsp+..]; mov r10,[rsp+..]; mov rcx,[rdx+..]; mov [rsp+..],r10; mov rdx,r8; mov r8,rax; call`), read as the displacement of `mov rcx,[rdx+..]`. The two are in two functions and share no bytes; both derived on 26.62 and 26.63. Every displacement is a wildcard. The range check keeps the value plausible. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "ref" { entry = "ClientInputUpdateSystem.extractRawHIDInput" }
    step "pattern" {
      bytes  = "48 8D 4C 24 ?? 48 89 F2 E8 ?? ?? ?? ?? F2 0F 10 44 24 ?? F2 0F 11 46 ??"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8B 4C 24 ?? 4C 8B 54 24 ?? 48 8B 4A ?? 4C 89 54 24 ?? 4C 89 C2 49 89 C0 E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 32
  }
}
