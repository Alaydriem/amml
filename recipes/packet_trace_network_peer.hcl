entry "PacketTraceNetworkPeer.receivePacket" {
  kind   = "function"
  hooked = true
  notes  = "Address of `NetworkPeer::DataStatus PacketTraceNetworkPeer::_receivePacket(std::string& outData, std::shared_ptr<std::chrono::steady_clock::time_point> const& timepoint)`: 0x16c6070 in 26.51, 0x226b6a0 in 26.60 (this in rcx, the out string in rdx, the time point in r8, the status in eax; 0 is HasData). It is slot 8 of the PacketTraceNetworkPeer vtable (0xe7be6e0 / 0xf1323e0). The NetworkConnection constructor (0xd926d0 / 0x2372af0) always wraps the peer chain last with PacketTraceNetworkPeer, on the remote path (PacketTrace > Batched > Compressed > [Encrypted] > RakNet or WebRTC) and on the local path (PacketTrace > LocalNetworkPeer), and nothing replaces NetworkConnection.mPeer later (encryption uses a weak pointer and does not wrap again). So every clientbound packet of a connection passes through this function once, after the decorators below it. It is DejaVu's clientbound tap: the hook is installed at enable (the loader allows hooks only in load and enable), and it records only when `this` is the open capture's NetworkConnection.mPeer. It calls the inner peer's slot 8 once. The RTTI names of the peer classes are not in the image, so there is no RTTI anchor. Recipe 0 is the function itself, from its first byte: `push rsi; push rdi; sub rsp,imm; mov rdi,rdx; mov rsi,rcx; mov rcx,[rcx+disp]; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; mov rdx,rdi; cmp qword ptr [rdi+disp],imm; mov edi,imm; cmovne edi,eax` (the same body as recipe 1 of NetworkPeer.receivePacket, with the 6 prologue bytes in front). Recipe 1 shares no bytes with it: the rest of the same function, from the second status test to the `ret` (`test eax,eax; cmovne edi,eax; test edi,edi; jne short; mov rcx,rsi; xor r8d,r8d; call; mov eax,edi; add rsp,imm; pop rdi; pop rsi; ret`; the call records the packet in the trace when the status is HasData), then `function_start_of`. A vtable route (the PacketTrace wrap in the NetworkConnection constructor, `lea rax,[rip+vtable]; mov [rsi+10h],rax; lea rcx,[rsi+28h]; mov r8d,660h; xor edx,edx; call memset` at 0xd92d59 / 0x23731df, then slot 8) also agrees, but a literal slot on a function entry is a lint finding, so it is not a recipe. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60: only the addresses. Survey: findings/plan-9/peer-survey/peer-survey.md."

  recipe {
    step "pattern" {
      bytes = "56 57 48 83 EC ?? 48 89 D7 48 89 CE 48 8B 49 ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 89 FA 48 83 7F ?? ?? BF ?? ?? ?? ?? 0F 45 F8"
    }
  }

  recipe {
    step "pattern" {
      bytes = "85 C0 0F 45 F8 85 FF 75 ?? 48 89 F1 45 31 C0 E8 ?? ?? ?? ?? 89 F8 48 83 C4 ?? 5F 5E C3"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}
