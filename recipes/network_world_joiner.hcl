entry "NetworkWorldJoiner.mServerWorldJoiner" {
  kind  = "field"
  notes = "Offset of `mServerWorldJoiner` in World::NetworkWorldJoiner: 210h on 26.51, 268h on 26.60, 26.62 and 26.63. The member is a World::ServerWorldJoiner held by value (an embedded object, not a pointer): NetworkWorldJoiner* + this offset is the `this` of ServerWorldJoiner::joinExternalServer. The LeviLamina header puts it after mGameServerShutdown (8), a PubSub::Subscription (10h) and the 250h-byte RealmWorldJoiner, so at 268h, the value on 26.60 to 26.63. Evidence: Parties::PartyTravelManager::_travelToDestination (it loads its own __FUNCSIG__) calls ServerWorldJoiner::joinExternalServer with `mov ecx,imm; add rcx,[reg+..]` (the NetworkWorldJoiner* it holds) as rcx; MinecraftGame::_handleMultiplayerServiceJoinResult adds the same immediate to [getWorldSystem()+WorldSystem.mNetworkWorldJoiner]; a NetworkWorldJoiner join method (0x2521540 on 26.63, 0x25217e0 on 26.62) does `add rdi,268h` on its `this` before a call. Recipe 0: the call set-up in _travelToDestination, `lea r9,[reg+..]; movzx r8d,word ptr [reg+..]; mov ecx,imm; add rcx,[reg+..]; mov qword ptr [rbp+..],imm; lea rax,[rbp+..]; mov [rsp+..],rax; mov rdx,reg; call`, offset 9, the immediate of operand 1. The ModRM bytes of the four register moves are wildcards, because 26.51 uses rdi and rbx the other way round. Recipe 1: `string_xref` on the __FUNCSIG__ of _handleMultiplayerServiceJoinResult, then inside it `mov ecx,imm; add rcx,[rax+disp]; xorps xmm0,xmm0`, a `pattern_any` with the disp8 form (26.51) and the disp32 form (26.60 and later), offset 0, the immediate of operand 1. The two recipes are in two functions and share no bytes. Every immediate, displacement and branch byte is a wildcard. Check: range 384 to 768. Values: 0x210 on 6aa482fd-2658118eba0dbac5 (26.51); 0x268 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 26.60: the member moved by 58h (the members before it grew, a layout change); it is the embedded ServerWorldJoiner on every build."

  recipe {
    step "pattern" {
      bytes  = "4C 8D ?? ?? 44 0F B7 ?? ?? B9 ?? ?? ?? ?? 48 03 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 89 ?? E8"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "string_xref" { text = "Bedrock::Threading::Async<void> __cdecl MinecraftGame::_handleMultiplayerServiceJoinResult(const Social::MultiplayerServiceIdentifier, Social::JoinGameStatus, const Social::MultiplayerGameInfo &, bool)" }
    step "pattern_any" {
      patterns = [
        "B9 ?? ?? ?? ?? 48 03 48 ?? 0F 57 C0",
        "B9 ?? ?? ?? ?? 48 03 88 ?? ?? ?? ?? 0F 57 C0",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 384
    max = 768
  }
}
