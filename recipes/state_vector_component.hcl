entry "StateVectorComponent.mPos" {
  kind     = "field"
  notes    = "Offset of `mPos` (Vec3) in StateVectorComponent: 0 in 26.51 (LeviLamina header: mPos, mPosPrev, mPosDelta, 12 bytes each). Both recipes read the displacement of the `movss xmm,[reg]` that loads the x component of the position from the state vector pointer, in the two functions described in Actor.stateVectorComponent. The load has no displacement byte, so the operand is 0; the loads of z (+8) and y (+4) in the same functions are the evidence that the pointer is a position. Every displacement byte of the other loads is a wildcard. A value of 0 means the position starts at the component base. Recipe 1 is a pattern_any: 26.51 `movss xmm4,[rax]` after the z subtraction; 26.60 `movsd xmm4,[rax]` (x and y as a pair) after the z subtraction and square; both alternatives start 17 bytes before the x load (26.51 at the load of the second actor's pointer, 26.60 at the z load). What changed in 6ad79ee4-17d51891a1119914: recipe 1's function was vectorised (see Actor.stateVectorComponent). Class: real code change (code shape). Evidence for 0 on 26.60: (1) recipe 0, drafted from 26.51, matches once on 26.60 and its `movss xmm0,[rax]` has no displacement, with the z load at +8 and the y load at +4 next to it; (2) in recipe 1's 26.60 function, z is read at +8 and the x/y pair at +0 of the same pointer."

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? F3 0F 10 00 F3 0F 10 70 ?? F3 0F 10 49 ??"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 92 ?? ?? ?? ?? F3 0F 10 48 ?? F3 0F 5C 4A ?? F3 0F 10 20 F3 0F 10 68 ?? F3 0F 5C 6A ?? F3 0F 5C 22",
        "F3 0F 10 48 ?? F3 0F 5C 4A ?? 0F 28 D1 F3 0F 59 D1 F2 0F 10 20 F2 0F 10 1A 0F 5C E3",
      ]
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 8
  }
}
