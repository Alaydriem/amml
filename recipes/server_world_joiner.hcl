entry "ServerWorldJoiner.joinExternalServer" {
  kind      = "function"
  msvc_name = "?joinExternalServer@ServerWorldJoiner@World@@QEAAXAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@G0V?$function@$$A6AXW4JoinServerWorldResult@World@@@Z@5@@Z"
  notes     = "Address of void World::ServerWorldJoiner::joinExternalServer(const std::string& server, unsigned short port, const std::string& externalServerName, std::function<void(World::JoinServerWorldResult)> onErrorCallback): 0x1b4a750 on 26.51, 0x21f3e0 on 26.60 and 26.62, 0x21f2c0 on 26.63. A mod calls it; it is not hooked. rcx = ServerWorldJoiner* (NetworkWorldJoiner* + NetworkWorldJoiner.mServerWorldJoiner), rdx = host text, r8w = port, r9 = name, [rsp+20h] = pointer to the by-value std::function (the callee moves it out and destroys the rest). It returns void at once; the resolve and the join run later as task work. Identity: the RTTI list holds a lambda scoped in `joinExternalServer@ServerWorldJoiner@World@@QEAAXAEBV?$basic_string...@G0V?$function@$$A6AXW4JoinServerWorldResult@World@@@Z@5@@Z` on every build (the msvc_name is that scope); the body takes the port with `movzx r8w`; it is one of the three callers of ServerWorldJoiner::_joinNetworkWorld (found by its __FUNCSIG__ text) and passes ServerType 1 (ExternalServer) with `mov r8d,1`; its only direct caller is Parties::PartyTravelManager::_travelToDestination (found by its __FUNCSIG__ text), which passes a host, a port and a name. Recipe 0: the function head at offset 0, from the pushes to the first call (`push rbp; push rsi; push rdi; sub rsp,imm; lea rbp,[rsp+..]; mov qword ptr [rbp+..],imm; ...; movzx eax,r8w; mov [rsp+..],eax; mov qword ptr [rsp+..],imm; lea rcx,[rbp+..]; xor edx,edx; mov r8,..; call`), a `pattern_any` with the 26.51 form (alternative 0: `mov rax,rdx; mov rsi,rcx; movzx ecx,r8w`, then `mov r8,r9; mov r9,rax`) and the 26.60 and later form (alternative 1: `mov rdi,r9; mov r9,rdx; mov rsi,rcx; movzx eax,r8w`, then `mov r8,rdi`); each alternative matches once on its own builds and not on the others. Recipe 1: the call in _travelToDestination (the same set-up as NetworkWorldJoiner.mServerWorldJoiner recipe 0), offset 47, the branch target. The two recipes are in two functions and share no bytes. Every immediate, displacement and branch byte is a wildcard. Check: function_start. Values: 0x1b4a750 on 6aa482fd-2658118eba0dbac5 (26.51), 0x21f3e0 on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62), 0x21f2c0 on 6aed1906-b127418e74d2f2f9 (26.63). What changed in 26.60: a real code change (_joinNetworkWorld gained the RecentlyPlayedPendingTargetData argument and the head allocates registers differently); the signature of this function is the same on every build."

  recipe {
    step "pattern_any" {
      patterns = [
        "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 D0 48 89 CE 41 0F B7 C8 89 4C 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 31 D2 4D 89 C8 49 89 C1 E8",
        "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 4C 89 CF 49 89 D1 48 89 CE 41 0F B7 C0 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 31 D2 49 89 F8 E8",
      ]
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8D ?? ?? 44 0F B7 ?? ?? B9 ?? ?? ?? ?? 48 03 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 89 ?? E8"
      offset = 47
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
