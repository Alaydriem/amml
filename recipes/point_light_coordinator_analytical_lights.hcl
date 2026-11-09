entry "PointLightCoordinator.getAnalyticalLights" {
  kind   = "function"
  hooked = true
  notes  = "PoC (dynamic-lighting). std::vector<AnalyticalLight>* PointLightCoordinator::getAnalyticalLights(std::vector<AnalyticalLight>* out) const: rcx = the coordinator (*(LevelRenderer+68h) on 1.26.62.01), rdx = the result vector (begin, end, capacity); returns rdx. Vibrant Visuals only: the per-frame point light list, 0xF0 bytes per light (+0 intensity float = 4*pi*e^2, +4 emission byte, +8 colour RGB floats, +14h world position floats, +20h flag byte, +28h id u64, +30h..F0h shadow slots, zero for none). Recipe 0 is the body after the prologue (`mov rsi,rcx; xorps xmm0,xmm0; movups [rdx],xmm0; mov qword [rdx+..]; cmp byte [rcx+..]`), then function_start_of. Recipe 1 is the call in the LevelRenderer update 0x53599d0 (`mov rcx,rdi; call [..]; mov rax,[rbp+..]; mov rcx,[rax+..]; lea rdx,[rbp+..]; call`), read as a branch target. 1.26.62.01: 0x790c9b0. See docs/findings/vibrant-visuals-light/report.md."

  recipe {
    step "pattern" { bytes = "48 89 CE 0F 57 C0 0F 11 02 48 C7 42 ?? ?? ?? ?? ?? 80 79 ?? ?? 48 89 95 ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 8B BE ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 85 ?? ?? ?? ?? 48 8B 48 ?? 48 8D 95 ?? ?? ?? ?? E8"
      offset = 27
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
