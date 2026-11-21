entry "AnimationComponent.applyAnimationsCallback" {
  kind   = "function"
  hooked = true
  notes  = "The brstd::function_ref callback that applies an AnimationComponent's animations: void(Captures* rcx = {AnimationComponent*, bool*}, Args* rdx); it returns void. 0x6194b40 in 26.60 (26.51 not checked). It reads comp = [rcx], calls ActorAnimationPlayer vslot 1 (applyToPose) on [comp+290h] or on each player of [comp+18h..20h] with r8 = comp+88h (the RenderParams: frame_alpha +108h, anim_time +114h, delta_time +118h, life_time +120h) and a 1.0 weight at [rsp+20h]. The callers compute delta_time and life_time from Actor.mTickCount (+1C0h) and the frame alpha just before, inlined. No direct caller: 0x6187220, 0x58b5250, 0x58b7ac0 and 0x6eba910 take its address with lea for AnimationComponent vslot 2, so a hook on the address works. DejaVu's video render reads and replaces delta_time and life_time here. Recipe 0: the head (mov rdi,[rcx]; cmp byte ptr [rdx+8],1; jne; cmp byte ptr [rsi+0Ah],1; jne; cmp qword ptr [rdi+comp field],-1; the first instructions of the function, `mov rsi,rdx` and on 1.26.70.26 also the stack cookie, are not in the pattern), then function_start_of. Recipe 1: the address-taking lea in a caller (the jump before the test, `cmp byte ptr [rsi+..],0; je; mov [frame],rsi; lea rax,[frame]; mov [frame],rax; lea rax,[rip+..]`), read as a RIP target; a pattern_any with one alternative per frame form (26.60 to 26.63 address the frame by rsp, 1.26.70.26 by rbp), both with the lea at index 26 of their bytes. Every immediate, displacement and branch byte is a wildcard. Found from the query.life_time std::function body (reads [rdx+120h]) back to its RenderParams owner. Findings: dejavu docs/findings/render-idle-speed/anim-clock.md. What changed in 6afa4e47-dadc375797a3d31a: the function got a stack cookie (`mov rbx,rcx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax`) between `mov rsi,rdx` and `mov rdi,[rcx]`, and the caller addresses its frame by rbp; the body is the same (same checks, the component field moved from 368h to 378h); class: would have survived with a better recipe (anchor after the prologue). The signature is unchanged: the same two arguments. Evidence on 6afa4e47: (1) the head drafted from the 26.63 function (without the first instruction) matches once in the new exe, in a function whose address is taken by `lea rax,[rip+..]` in four functions, as the old function is (4 references each, `inspect xrefs --rva`); (2) the caller recipe, drafted from a 26.63 caller, matches once in a 70.26 caller and its lea target is the same function that recipe 0 finds."

  recipe {
    step "pattern" {
      bytes = "48 8B 39 80 7A ?? ?? 0F 85 ?? ?? ?? ?? 80 7E ?? ?? 75 ?? 48 83 BF ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "7D ?? 80 BE ?? ?? ?? ?? ?? 74 ?? 48 89 74 24 ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 8D 05",
        "48 39 ?? 7D ?? 80 BE ?? ?? ?? ?? ?? 74 ?? 48 89 75 ?? 48 8D 45 ?? 48 89 45 ?? 48 8D 05",
      ]
      offset = 26
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
