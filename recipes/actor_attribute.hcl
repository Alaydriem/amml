entry "ActorAttribute.getHealth" {
  kind      = "function"
  msvc_name = "?getHealth@ActorAttribute@@YAHAEBVEntityContext@@@Z"
  notes     = "Address of the free function `int ActorAttribute::getHealth(EntityContext const&)`: 0x25397a0 in 26.51. The context is in rcx, the int comes back in eax. LeviLamina's `Actor::getHealth` calls it with the actor's entity context. The body looks up the AttributesComponent through the entt registry at `[rcx+8]` and the entity id at `[rcx+10h]` (the component type id is an immediate), gets the attribute instance of SharedAttributes::HEALTH (`lea r8,[rip+..]`, then a call), reads its current value (`movss xmm0,[rax+7Ch]`) and rounds it up to an int (`cvttss2si; cvttps2dq; cvtdq2ps; ucomiss; adc eax,0`); without the component it returns 0. The next function, ActorAttribute::getMaxHealth (0x25398c0), is the same code with the maximum value at `[rax+78h]`. Recipe 0 is the tail after the instance call (`mov rax,[rsp+..]; movss xmm0,[rax+7Ch]; cvttss2si eax,xmm0; cvttps2dq; cvtdq2ps; ucomiss; adc eax,..`), then function_start_of; the displacement 7Ch is kept, because only it tells this function from getMaxHealth. Recipe 1 is the call in a small Actor virtual (`sub rsp,..; cmp byte ptr [rcx+..],..; je; xor eax,eax; add rsp,..; ret; add rcx,..; call`), read as a branch target. The two recipes are in two functions and share no pattern bytes. Every other displacement, immediate and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the float to int conversion of the value clamps at 0 and at INT_MAX (`xor eax,eax; xorps; ucomiss; jae; mov eax,7FFFFFFFh; ucomiss`) instead of rounding up, so recipe 0 is a pattern_any with one alternative per shape; the value is still read from [rax+7Ch]. Evidence on 26.60: recipe 0 and recipe 1 (the call in the Actor virtual) reach the same function."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 44 24 ?? F3 0F 10 40 7C F3 0F 2C C0 F3 0F 5B C8 0F 5B C9 0F 2E C8 83 D0 ??",
        "48 8B 44 24 ?? F3 0F 10 40 7C 31 C0 0F 57 C9 0F 2E C8 73 ?? B8 ?? ?? ?? ?? 0F 2E 05",
      ]
      literal_ok = [
        "0.9: movss xmm0,[rax+7Ch] reads the current value; getMaxHealth, the next function, is the same code with [rax+78h]",
        "1.9: the same byte in the 26.60 shape",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 EC ?? 80 B9 ?? ?? ?? ?? ?? 74 ?? 31 C0 48 83 C4 ?? C3 48 83 C1 ?? E8"
      offset = 24
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
