entry "AnimationComponent.applyAnimationsCallback" {
  kind   = "function"
  hooked = true
  notes  = "The brstd::function_ref callback that applies an AnimationComponent's animations: void(Captures* rcx = {AnimationComponent*, bool*}, Args* rdx); it returns void. 0x6194b40 in 26.60 (26.51 not checked). It reads comp = [rcx], calls ActorAnimationPlayer vslot 1 (applyToPose) on [comp+290h] or on each player of [comp+18h..20h] with r8 = comp+88h (the RenderParams: frame_alpha +108h, anim_time +114h, delta_time +118h, life_time +120h) and a 1.0 weight at [rsp+20h]. The callers compute delta_time and life_time from Actor.mTickCount (+1C0h) and the frame alpha just before, inlined. No direct caller: 0x6187220, 0x58b5250, 0x58b7ac0 and 0x6eba910 take its address with lea for AnimationComponent vslot 2, so a hook on the address works. DejaVu's video render reads and replaces delta_time and life_time here. Recipe 0: the head (mov rsi,rdx; mov rdi,[rcx]; cmp byte ptr [rdx+8],1; jne; cmp byte ptr [rsi+0Ah],1; jne; cmp qword ptr [rdi+368h],-1), then function_start_of. Recipe 1: the address-taking lea in 0x6187220 (cmp byte ptr [rsi+378h],0; je; mov [rsp+..],rsi; lea rax,[rsp+..]; mov [rsp+..],rax; lea rax,[rip+..]), read as a RIP target. Every immediate, displacement and branch byte is a wildcard. Found from the query.life_time std::function body (reads [rdx+120h]) back to its RenderParams owner. Findings: dejavu docs/findings/render-idle-speed/anim-clock.md."

  recipe {
    step "pattern" {
      bytes = "48 89 D6 48 8B 39 80 7A ?? ?? 0F 85 ?? ?? ?? ?? 80 7E ?? ?? 75 ?? 48 83 BF ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "80 BE ?? ?? ?? ?? ?? 74 ?? 48 89 74 24 ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 44 24 ??"
      offset = 24
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
