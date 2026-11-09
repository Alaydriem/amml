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
