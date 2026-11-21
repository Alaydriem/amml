entry "WorldSystem.mNetworkWorldJoiner" {
  kind  = "field"
  notes = "Offset of the std::unique_ptr<World::NetworkWorldJoiner> in World::WorldSystem, the object that MinecraftGame.getWorldSystem returns: 70h on 26.51, 80h on 26.60, 26.62 and 26.63. In the LeviLamina header it is WorldSystem::mModules (a World::WorldSystemModules at offset 0) .mNetworkWorldJoiner; the header layout (bool, then unique_ptr and shared_ptr members) puts it at 80h, the value on 26.60 to 26.63. The code loads the NetworkWorldJoiner* from it (an 8-byte load) and uses it as `this` or adds NetworkWorldJoiner.mServerWorldJoiner to it. Evidence: MinecraftGame::_handleMultiplayerServiceJoinResult calls getWorldSystem and computes `mov ecx,268h; add rcx,[rax+80h]` (26.51: `mov ecx,210h; add rcx,[rax+70h]`), the ServerWorldJoiner*; a second function (0xa243c0 on 26.63, 0x978420 on 26.51) calls getWorldSystem and loads `mov rsi,[rax+80h]` (26.51 `[rax+70h]`). Recipe 0 is in MinecraftGame::_handleMultiplayerServiceJoinResult: `mov ecx,imm; add rcx,[rax+disp]` and then the next instructions, a `pattern_any` with one alternative for each shape of what follows the add (the stores of the zeroed string on 26.60 to 26.63, the `lea rdx,[rbp+..]; mov r8,rbx; mov r9,rdi; call` argument set-up on 70.26), offset 5, the displacement of operand 1; the 26.51 alternative (the disp8 form) was removed: without the string anchor it matched another function on 26.63 and on 70.26, and 26.51 is frozen (recipe 1 still has its disp8 form). Recipe 1 is the second function found by the chain `mov rdi,[rcx+..]; mov rax,[rdi]; mov rcx,[rax+..]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]` (no prologue: the pushes of that function changed on 70.26), then inside that function `mov rsi,[rax+disp]; xorps xmm0,xmm0; movaps [rbp+..],xmm0`, a `pattern_any` with the disp8 and disp32 forms, the displacement of operand 1. The two recipes are in two functions and share no bytes. Every immediate, displacement and branch byte is a wildcard. Check: range 64 to 176 (inside the 184-byte WorldSystemModules). Values: 0x70 on 6aa482fd-2658118eba0dbac5 (26.51); 0x80 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 26.60: the member moved by 10h (the members before it grew, a layout change); the meaning (an owning NetworkWorldJoiner pointer) is the same on every build. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the member is still 80h; the join-result function is no longer found by its __FUNCSIG__ text (it is not in the exe), and its code after the add changed; the second function lost the prologue pattern (other pushes). Evidence from the old function: (1) the second function is the one with the chain that reads getWorldSystem through slot 152 (4C0h) on both builds, and it loads `mov rsi,[rax+80h]` right after the call on 70.26, as on 26.63; (2) the join-result function has `mov ecx,268h; add rcx,[rax+80h]` on both builds (268h is NetworkWorldJoiner.mServerWorldJoiner), and the pattern matches once on 70.26 in a function of 14.7 KB (26.63: 10.3 KB). Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "B9 ?? ?? ?? ?? 48 03 88 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ??",
        "B9 ?? ?? ?? ?? 48 03 88 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 49 89 D8 49 89 F9 E8",
      ]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 79 ?? 48 8B 07 48 8B 48 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
    }
    step "pattern_any" {
      patterns = [
        "48 8B 70 ?? 0F 57 C0 0F 29 45",
        "48 8B B0 ?? ?? ?? ?? 0F 57 C0 0F 29 45",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 176
  }
}

entry "WorldSystem.mLocalWorldStarter" {
  kind  = "field"
  notes = "Offset of the std::unique_ptr<World::LocalWorldStarter> in World::WorldSystem, the object that MinecraftGame.getWorldSystem returns: 78h on 26.63. In the LeviLamina header it is WorldSystem::mModules (a World::WorldSystemModules at offset 0) .mLocalWorldStarter, the member right before mNetworkWorldJoiner (WorldSystem.mNetworkWorldJoiner, 80h on 26.63); the header layout (bool, then unique_ptr and shared_ptr members) puts it at 78h. The code loads the LocalWorldStarter* from it (an 8-byte load); that pointer is the `this` of LocalWorldStarter.startLocalWorld. Evidence: the function that makes the Ore UI WorldStartupFacet (0x62d8240 on 26.63) calls getWorldSystem three times and loads `mov rbx,[rax+38h]`, `mov r14,[rax+50h]`, `mov r15,[rax+78h]`, then passes r15, r14 and rbx as the first three arguments of the WorldStartupFacet constructor (0x7aca290), whose header order is LocalWorldStarter&, IWorldCloudSyncer&, IWorldResourcePackHandler&; 38h, 50h and 78h are mWorldResourcePackHandler, mWorldCloudSyncer and mLocalWorldStarter in the header layout. The constructor stores the first argument at facet+18h, and the facet's start task passes [facet+18h] as `this` to LocalWorldStarter.startLocalWorld. Recipe 0: the second and third getWorldSystem calls and the allocation of the facet after them, `call [guard]; nop; mov r14,[rax+disp8]; mov rcx,[r15+..]; mov rax,[rcx]; mov rax,[rax+disp32]; call [guard]; nop; mov r15,[rax+disp8]; mov rcx,[rip+..]; mov rax,[rcx]; mov rax,[rax+disp8]; mov edx,imm; call [guard]`, offset 32, the displacement of operand 1 (the third load). The first call of the pattern runs into a `nop` after each call, which every build of this entry has; the shorter pattern of 26.63 (from the `nop`) is no longer unique on 70.26 (a twin function). Every immediate, displacement and branch byte is a wildcard. One recipe and a range check, as for the other field entries found in one function. Check: range 64 to 176 (inside the 184-byte WorldSystemModules). Values: 0x78 on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62). Found on 26.63; 26.60 (6ad79ee4-17d51891a1119914) was not checked, because no 26.60 exe was available. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the offset is still 78h; the old short pattern matches a second function on 70.26, so the pattern starts two calls earlier. Evidence from the old function: (1) the pattern drafted from the old function (the second and third getWorldSystem loads and the allocation) matches once on 70.26, in the function that calls getWorldSystem (4C0h) three times and loads 38h, 50h and 78h from the result, as the old function does; (2) in that function the three loads go to rbx, r14 and r15 and are passed as the third, fourth and fifth arguments of the next call (`mov rdx,r15; mov r8,r14; mov r9,rbx`), the order of the WorldStartupFacet constructor on 26.63. Class: would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "FF 15 ?? ?? ?? ?? 90 4C 8B 70 ?? 49 8B 4F ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 4C 8B 78 ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? BA ?? ?? ?? ?? FF 15"
      offset = 32
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 176
  }
}
