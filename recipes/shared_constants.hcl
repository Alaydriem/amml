entry "SharedConstants.NetworkProtocolVersion" {
  kind  = "field"
  notes = "The network protocol of the game build (SharedConstants::NetworkProtocolVersion), as a number: 2225 (8B1h) on 26.62 and 26.63. The game has no symbol and no global for it: the compiler folds it into immediates. Requested by DejaVu, which needs the protocol before the first world join. Found by a scan for the immediate 8B1h (the value in RequestNetworkSettingsPacket that DejaVu sees on the wire, and RELEASED_LATEST of bedrock-protocol for the 26.60 line), then by reading each hit. Three sites carry it: (a) slot 16 of the RequestNetworkSettingsPacket vtable (the read of the packet; the vtable comes from `inspect packets`, id 193) compares the protocol it read with the constant: `mov eax,[rbp+..]; cmp eax,8B1h; je; mov rcx,[rbx+..]; mov [rbx+..],rcx; mov byte ptr [rbp+..],1; mov [rbp+..],eax`; (b) a compare of a peer's protocol with the build's, which also holds the constant minus one: `cmp eax,8B0h; jg; mov rcx,[rsi+..]; mov rax,[rcx]; mov rax,[rax+..]; mov r8,[rip+..]; mov edx,imm; jmp; cmp eax,8B1h; jne` (edx 0Dh and 0Eh are the two outdated results); (c) LegacyClientNetworkHandler.vtable slot 2, which builds the RequestNetworkSettingsPacket it sends: `mov rax,1000008B1h; mov [rsp+58h],rax` writes the protocol and the next dword in one store, so its immediate is not the protocol alone and no recipe uses it; it confirms the value. The settings screen does not read a number: it formats `options.protocolversion.format` with the string literal \"2225\" built inline (`mov [rbp..],35323232h`), so it is no source. Recipe 0 is site (a), operand 1 (the immediate) of the `cmp`, offset 6. Recipe 1 is site (b), operand 1 of the second `cmp`, offset 35. The two recipes are in two functions and share no anchor bytes. Every displacement, immediate and branch byte of both patterns is a wildcard, the compared constants too, so a new protocol number does not break them. Both patterns are unique on 26.62 (6aea437e-7eff13daabb430da) and 26.63 (6aed1906-b127418e74d2f2f9). Check: range 1..=10000. Values: 26.62 2225, 26.63 2225. 26.51, 26.60 and the Preview 1.26.70.23 not looked at (no exe here)."

  recipe {
    step "pattern" {
      bytes  = "8B 85 ?? ?? ?? ?? 3D ?? ?? ?? ?? 74 ?? 48 8B 4B ?? 48 89 4B ?? C6 85 ?? ?? ?? ?? ?? 89 85"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "3D ?? ?? ?? ?? 7F ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 4C 8B 05 ?? ?? ?? ?? BA ?? ?? ?? ?? EB ?? 3D ?? ?? ?? ?? 75"
      offset = 35
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 10000
  }
}
