entry "ActorRotationComponent.mRot" {
  kind     = "field"
  notes    = "Offset of `mRot` (Vec2) in ActorRotationComponent: 0 in 26.51 (LeviLamina header: mRot, then mRotPrev, 8 bytes each). x is the pitch and y is the yaw: Actor::getYawSpeedInDegreesPerSecond wraps `[rot+4] - [rot+0Ch]` (mRot.y - mRotPrev.y) to -180..180 and multiplies by 20; a goal function turns `[rot+4]` into a horizontal heading (-sin, cos through the 65536-entry sine table, scale 10430.378 = 65536/2pi after the degree-to-radian factor); and the throw function of recipe 0 uses sin(x) alone for the vertical speed (-0.3 * sin(x)) and sin(y), cos(y) times cos(x) for the horizontal speed, which is the pitch/yaw direction formula. Recipe 0 reads the displacement of `movss xmm6,[rax]` (the x load) in the throw function described in Actor.actorRotationComponent (recipe 0). Recipe 1 reads the displacement of `movss [rax],xmm7` (the x store) in the rotation setter described there (recipe 2). Neither instruction has a displacement byte, so the operand is 0; the +4, +8 and +0Ch accesses next to them are the evidence that the pointer is the rotation pair. Every other displacement byte is a wildcard. Recipe 0 follows the pattern_any of Actor.actorRotationComponent recipe 0: the x load is `movss xmm6,[rax]` in 26.51 and `movsd xmm6,[rax]` (x and y as a pair) in 26.60, both 7 bytes after the start. What changed in 6ad79ee4-17d51891a1119914: the throw function was vectorised. Class: real code change (code shape). Evidence for 0 on 26.60: (1) recipe 1 (the rotation setter, drafted from 26.51) matches once on 26.60 and its store `movss [rax],xmm7` has no displacement, with +4, +8 and +0Ch stores next to it; (2) in the 26.60 throw function the pitch/yaw pair is read with `movsd xmm6,[rax]` from the rotation pointer, also without a displacement."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 30 F3 0F 10 78 ?? 48 8D 4D ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 FA E8",
        "48 8B ?? ?? ?? ?? ?? F2 0F 10 30 48 8D 4D ?? F3 0F 10 15 ?? ?? ?? ?? 48 89 FA E8",
      ]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 81 ?? ?? ?? ?? F3 0F 11 38 F3 0F 11 40 ?? 48 8B 81 ?? ?? ?? ?? F3 0F 11 78 ?? F3 0F 11 40 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ??"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 8
  }
}
