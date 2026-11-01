entry "OreUIRouter.pushRoute" {
  kind      = "function"
  hooked    = true
  msvc_name = "?_pushRoute@Router@OreUI@@QEAA_NAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@W4RouterPushMode@12@@Z"
  notes     = "bool OreUI::Router::_pushRoute(std::string const& route, RouterPushMode mode): 0x4302930 in 26.51, 0xf3c740 in 26.60. A plain function, not virtual; the inlined public pushRoute calls it (124 direct callers in 26.51, 131 in 26.60). rcx = the Router, rdx = the route string (MSVC std::string: data or the small buffer at +0, size at +10h, capacity at +18h), r8d = the push mode; the result is a bool in al (false when the route is refused). Found in 26.51 through the OreUI script binding of RouterHistory, which binds \"push\" to a thunk that jumps here with r8d = 1; that binding is not in the 26.60 exe. Recipe 0 is the head after the frame set-up: `mov edi,r8d; mov r8,rdx; mov rsi,rcx; mov rcx,[rcx+member]; mov rax,[rcx]; mov rax,[rax+disp]; lea rbx,[rbp+..]` (the member is mRouteMatcher at +80h), then `function_start_of`. Recipe 1 is a navigate helper in the ClientInstance code (0x5da6130 in 26.51, 0x6595c00 in 26.60) that ends in two tail jumps: to replaceRoute when its bool is set, and here with r8d = 0 otherwise. The recipe finds the helper by its head (`mov ebx,r8d; mov rsi,rdx; mov rdi,rcx; stack cookie; the string data and size reads; lea rdx,[rsp+..]; mov rcx,rdi; call`), then finds `mov rcx,rdi; mov rdx,rsi; xor r8d,r8d; add rsp,imm; pop rbx; pop rdi; pop rsi; jmp` in it and reads the jump target at offset 16. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60: the NonOwnerPointer asserts are gone; the head did not change."

  recipe {
    step "pattern" {
      bytes = "44 89 C7 49 89 D0 48 89 CE 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D 9D"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "44 89 C3 48 89 D6 48 89 CF 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 42 ?? 48 83 7A ?? ?? 48 89 D1 72 ?? 48 8B 0E 48 89 4C 24 ?? 48 89 44 24 ?? 48 8D 54 24 ?? 48 89 F9 E8"
    }
    step "pattern" {
      bytes  = "48 89 F9 48 89 F2 45 31 C0 48 83 C4 ?? 5B 5F 5E E9"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "OreUIRouter.replaceRoute" {
  kind      = "function"
  hooked    = true
  msvc_name = "?replaceRoute@Router@OreUI@@QEAA_NAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
  notes     = "bool OreUI::Router::replaceRoute(std::string const& route): 0x4304ec0 in 26.51, 0xf3d000 in 26.60. A plain function, not virtual (20 direct callers in each build). rcx = the Router, rdx = the route string (MSVC std::string: data or the small buffer at +0, size at +10h, capacity at +18h); the result is a bool in al. Found in 26.51 through the OreUI script binding of RouterHistory, which binds \"replace\" to a thunk that jumps here; that binding is not in the 26.60 exe. The body starts with the empty-history test, `mov r8,rdx; mov rsi,rcx; mov rax,[rcx]; cmp rax,[rcx+8]; je`, then the same two route-matcher calls as _pushRoute. The `je` is short in 26.51 and near in 26.60, so recipe 0 is a `pattern_any` with one alternative for each build (each is unique on its own build and has no match on the other): `mov r8,rdx; mov rsi,rcx; mov rax,[rcx]; cmp rax,[rcx+disp]; je; mov rcx,[rsi+member]; mov rax,[rcx]; mov rax,[rax+disp]` (26.51 also `lea rdi,[rbp+..]`), then `function_start_of`. Recipe 1 is the navigate helper in the ClientInstance code (see OreUIRouter.pushRoute recipe 1): it finds the helper by its head, then `mov rcx,rdi; mov rdx,rsi; add rsp,imm; pop rbx; pop rdi; pop rsi; jmp` in it (the tail jump when the bool is set; the _pushRoute jump has `xor r8d,r8d` in between) and reads the jump target at offset 13. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60: the first `je` became a near jump (the body after it grew) and the asserts are gone."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 89 D0 48 89 CE 48 8B 01 48 3B 41 ?? 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D BD",
        "49 89 D0 48 89 CE 48 8B 01 48 3B 41 ?? 0F 84 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8D BD",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "44 89 C3 48 89 D6 48 89 CF 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 42 ?? 48 83 7A ?? ?? 48 89 D1 72 ?? 48 8B 0E 48 89 4C 24 ?? 48 89 44 24 ?? 48 8D 54 24 ?? 48 89 F9 E8"
    }
    step "pattern" {
      bytes  = "48 89 F9 48 89 F2 48 83 C4 ?? 5B 5F 5E E9"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
