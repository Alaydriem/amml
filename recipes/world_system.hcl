entry "WorldSystem.mNetworkWorldJoiner" {
  kind  = "field"
  notes = "Offset of the std::unique_ptr<World::NetworkWorldJoiner> in World::WorldSystem, the object that MinecraftGame.getWorldSystem returns: 70h on 26.51, 80h on 26.60, 26.62 and 26.63. In the LeviLamina header it is WorldSystem::mModules (a World::WorldSystemModules at offset 0) .mNetworkWorldJoiner; the header layout (bool, then unique_ptr and shared_ptr members) puts it at 80h, the value on 26.60 to 26.63. The code loads the NetworkWorldJoiner* from it (an 8-byte load) and uses it as `this` or adds NetworkWorldJoiner.mServerWorldJoiner to it. Evidence: MinecraftGame::_handleMultiplayerServiceJoinResult calls getWorldSystem and computes `mov ecx,268h; add rcx,[rax+80h]` (26.51: `mov ecx,210h; add rcx,[rax+70h]`), the ServerWorldJoiner*; a second function (0xa243c0 on 26.63, 0x978420 on 26.51) calls getWorldSystem and loads `mov rsi,[rax+80h]` (26.51 `[rax+70h]`). Recipe 0: `string_xref` on the __FUNCSIG__ of _handleMultiplayerServiceJoinResult, then inside it `mov ecx,imm; add rcx,[rax+disp]; xorps xmm0,xmm0`, a `pattern_any` with the disp8 form (26.51, alternative 0) and the disp32 form (26.60 and later, alternative 1), offset 5, the displacement of operand 1. The pattern starts after the getWorldSystem call, so it does not run past a call. Recipe 1: the prologue of the second function (the same pattern as MinecraftGame.getWorldSystem recipe 1), then inside that function `mov rsi,[rax+disp]; xorps xmm0,xmm0; movaps [rbp+..],xmm0`, a `pattern_any` with the disp8 and disp32 forms, the displacement of operand 1. The two recipes are in two functions and share no bytes. Every immediate, displacement and branch byte is a wildcard. Check: range 64 to 176 (inside the 184-byte WorldSystemModules). Values: 0x70 on 6aa482fd-2658118eba0dbac5 (26.51); 0x80 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 26.60: the member moved by 10h (the members before it grew, a layout change); the meaning (an owning NetworkWorldJoiner pointer) is the same on every build."

  recipe {
    step "string_xref" { text = "Bedrock::Threading::Async<void> __cdecl MinecraftGame::_handleMultiplayerServiceJoinResult(const Social::MultiplayerServiceIdentifier, Social::JoinGameStatus, const Social::MultiplayerGameInfo &, bool)" }
    step "pattern_any" {
      patterns = [
        "B9 ?? ?? ?? ?? 48 03 48 ?? 0F 57 C0",
        "B9 ?? ?? ?? ?? 48 03 88 ?? ?? ?? ?? 0F 57 C0",
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
      bytes = "55 56 57 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 79 ?? 48 8B 07 48 8B 48 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15"
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
  notes = "Offset of the std::unique_ptr<World::LocalWorldStarter> in World::WorldSystem, the object that MinecraftGame.getWorldSystem returns: 78h on 26.63. In the LeviLamina header it is WorldSystem::mModules (a World::WorldSystemModules at offset 0) .mLocalWorldStarter, the member right before mNetworkWorldJoiner (WorldSystem.mNetworkWorldJoiner, 80h on 26.63); the header layout (bool, then unique_ptr and shared_ptr members) puts it at 78h. The code loads the LocalWorldStarter* from it (an 8-byte load); that pointer is the `this` of LocalWorldStarter.startLocalWorld. Evidence: the function that makes the Ore UI WorldStartupFacet (0x62d8240 on 26.63) calls getWorldSystem three times and loads `mov rbx,[rax+38h]`, `mov r14,[rax+50h]`, `mov r15,[rax+78h]`, then passes r15, r14 and rbx as the first three arguments of the WorldStartupFacet constructor (0x7aca290), whose header order is LocalWorldStarter&, IWorldCloudSyncer&, IWorldResourcePackHandler&; 38h, 50h and 78h are mWorldResourcePackHandler, mWorldCloudSyncer and mLocalWorldStarter in the header layout. The constructor stores the first argument at facet+18h, and the facet's start task passes [facet+18h] as `this` to LocalWorldStarter.startLocalWorld. Recipe 0: the third load and the allocation of the facet after it, `nop; mov r15,[rax+disp8]; mov rcx,[rip+..]; mov rax,[rcx]; mov rax,[rax+disp8]; mov edx,imm; call [guard]`, offset 1, the displacement of operand 1. It starts at the `nop` that follows the getWorldSystem call and ends at the allocator call; without the `nop` it has 28 matches. Every immediate, displacement and branch byte is a wildcard. One recipe and a range check, as for the other field entries found in one function. Check: range 64 to 176 (inside the 184-byte WorldSystemModules). Values: 0x78 on 6aed1906-b127418e74d2f2f9 (26.63) and 6aea437e-7eff13daabb430da (26.62). Found on 26.63; 26.60 (6ad79ee4-17d51891a1119914) was not checked, because no 26.60 exe was available."

  recipe {
    step "pattern" {
      bytes  = "90 4C 8B 78 ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? BA ?? ?? ?? ?? FF 15"
      offset = 1
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
