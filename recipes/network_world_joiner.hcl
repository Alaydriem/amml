entry "NetworkWorldJoiner.mServerWorldJoiner" {
  kind  = "field"
  notes = "Offset of `mServerWorldJoiner` in World::NetworkWorldJoiner: 210h on 26.51, 268h on 26.60, 26.62 and 26.63. The member is a World::ServerWorldJoiner held by value (an embedded object, not a pointer): NetworkWorldJoiner* + this offset is the `this` of ServerWorldJoiner::joinExternalServer. The LeviLamina header puts it after mGameServerShutdown (8), a PubSub::Subscription (10h) and the 250h-byte RealmWorldJoiner, so at 268h, the value on 26.60 to 26.63. Evidence: Parties::PartyTravelManager::_travelToDestination (it loads its own __FUNCSIG__) calls ServerWorldJoiner::joinExternalServer with `mov ecx,imm; add rcx,[reg+..]` (the NetworkWorldJoiner* it holds) as rcx; MinecraftGame::_handleMultiplayerServiceJoinResult adds the same immediate to [getWorldSystem()+WorldSystem.mNetworkWorldJoiner]; a NetworkWorldJoiner join method (0x2521540 on 26.63, 0x25217e0 on 26.62) does `add rdi,268h` on its `this` before a call. Recipe 0: the call set-up in _travelToDestination, `lea r9,[reg+..]; movzx r8d,word ptr [reg+..]; mov ecx,imm; add rcx,[reg+..]; mov qword ptr [rbp+..],imm; lea rax,[rbp+..]; mov [rsp+..],rax; mov rdx,reg; call`, offset 9, the immediate of operand 1. The ModRM bytes of the four register moves are wildcards, because 26.51 uses rdi and rbx the other way round. Recipe 1: `string_xref` on the __FUNCSIG__ of _handleMultiplayerServiceJoinResult, then inside it `mov ecx,imm; add rcx,[rax+disp]; xorps xmm0,xmm0`, a `pattern_any` with the disp8 form (26.51) and the disp32 form (26.60 and later), offset 0, the immediate of operand 1. The two recipes are in two functions and share no bytes. Every immediate, displacement and branch byte is a wildcard. Check: range 384 to 768. Values: 0x210 on 6aa482fd-2658118eba0dbac5 (26.51); 0x268 on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 26.60: the member moved by 58h (the members before it grew, a layout change); it is the embedded ServerWorldJoiner on every build. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the value is still 268h. Recipe 0 is a pattern_any: the second alternative has `mov [rsp+..],reg` where the old code had `lea rax,[rbp+..]; mov [rsp+..],rax` before the call. The function that carried the string anchor of recipe 1 now has another FUNCSIG text (a weak_ptr<DebugSink> argument was added) and no longer holds `mov ecx,imm; add rcx,[..]` (it holds `add rdi,imm`), so recipe 1 is a pattern_any of two global patterns, one for each build, each unique in its exe: the old alternative (the call set-up with the zeroed frame members) and the new one (`add rdi,imm; lea rdx; lea r9; mov rcx,rdi; mov r8,rbx; call; nop; mov rdi,[rbp+..]; test rdi,rdi; je`). Both read the immediate. Evidence: (1) recipe 0 is the call set-up of ServerWorldJoiner.joinExternalServer, which is a direct call of the resolved entry at the same place in the function that holds it on both builds; (2) recipe 1 is a second function (the multiplayer join result handler) that adds the same immediate before the other join call. Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 8D ?? ?? 44 0F B7 ?? ?? B9 ?? ?? ?? ?? 48 03 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 89 ?? E8",
        "4C 8D ?? ?? 44 0F B7 ?? ?? B9 ?? ?? ?? ?? 48 03 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 ?? 24 ?? 48 89 ?? E8",
      ]
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "B9 ?? ?? ?? ?? 48 03 88 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? C6 85 ?? ?? ?? ?? ?? C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? 4D 89 F0 E8",
        "48 81 C7 ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? 48 89 F9 49 89 D8 E8 ?? ?? ?? ?? 90 48 8B BD ?? ?? ?? ?? 48 85 FF 74 ??",
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
