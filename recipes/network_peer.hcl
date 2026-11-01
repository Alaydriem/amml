entry "NetworkPeer.receivePacket" {
  kind  = "vslot"
  notes = "Slot of `NetworkPeer::DataStatus NetworkPeer::_receivePacket(std::string& outData, std::shared_ptr<std::chrono::steady_clock::time_point> const& timepoint)` in the NetworkPeer vtable: 8 in 26.51 (this in rcx, the out string in rdx, the time point in r8, the status in eax). LeviLamina header order: the destructor, sendPacket, getNetworkStatus, update, flush, isLocal, isEncrypted, isLan, then _receivePacket, so 8 (NetworkSystem.sendInternal calls slot 1, sendPacket). It is the clientbound tap: on 26.60 NetworkConnection::receivePacket is inlined, so the mod hooks the slot of the connection's peer (NetworkConnection.mPeer). Recipe 0 is the call of the peer by the connection (`mov rcx,[conn+..]; mov rax,[rcx]; mov rax,[rax+..]; ...; call [guard]`), read as a displacement (offset 10, operand 1) and turned into a slot. It is a pattern_any, one alternative per shape: alternative 0 is the 26.51 NetworkConnection::receivePacket body (0xd935e0; the connection in r13, the arguments from rdi and rbx, then `mov ecx,eax; mov rdx,[rdi+..]; test rdx,rdx`); alternative 1 is the same call inlined in the 26.60 NetworkSystem::_sortAndPacketizeEvents (0x237e790; the connection in rdi, the time point from the frame, then `nop; mov rdi,[r15+..]; test rdi,rdi; mov ecx,eax`). Each alternative matches once on its own build and not on the other. Recipe 1 is a decorator peer's own _receivePacket (0x16c6070 in 26.51, 0x226b6a0 in 26.60; it is slot 8 of its own vtable, 0xe7be6e0 / 0xf1323e0): it forwards to the peer it wraps (`mov rdi,rdx; mov rsi,rcx; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; mov rdx,rdi; cmp qword ptr [rdi+..],..; mov edi,..; cmovne edi,eax`); offset 13 is the slot load, read the same way. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 4..12. Values: 8 in 26.51 and in 26.60. What changed in 6ad79ee4-17d51891a1119914: NetworkConnection::receivePacket was inlined into its one caller (a real code change), so recipe 0 has a second alternative; the slot did not move. Evidence for 8 on 26.60: (1) the inlined call in NetworkSystem::_sortAndPacketizeEvents (the only function that loads its own name) reads `[rax+40h]` through `[rdi+118h]`; (2) the decorator's _receivePacket, drafted from 26.51, matches once on 26.60, reads `[rax+40h]` of the peer it wraps, and sits in slot 8 of its own vtable."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 8B 8D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 89 FA 49 89 D8 FF 15 ?? ?? ?? ?? 89 C1 48 8B 57 ?? 48 85 D2",
        "48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 89 DA 4C 8D 85 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 49 8B BF ?? ?? ?? ?? 48 85 FF 89 C1",
      ]
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 D7 48 89 CE 48 8B 49 ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 89 FA 48 83 7F ?? ?? BF ?? ?? ?? ?? 0F 45 F8"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 4
    max = 12
  }
}
