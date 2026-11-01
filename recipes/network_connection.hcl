entry "NetworkConnection.mPeer" {
  kind  = "field"
  notes = "Offset of `mPeer` (std::shared_ptr<NetworkPeer>: the pointer, then the control block) in NetworkConnection: 0x118 in 26.51. LeviLamina header (network/NetworkConnection.h): mId (NetworkIdentifier, 0xB0 bytes) at 0, mType at +B0h, six weak_ptr peers from +B8h, then mPeer at +118h. Recipe 0 is NetworkSystem.sendInternal, scanned inside that function: after it finds the connection it tests a flag of the connection, then loads the peer and tests it for null before the tail-jump to the peer's slot 1 (`jne; mov rcx,[conn+..]; test rcx,rcx; je; mov r8d,[rdi+..]; mov r9d,[rdi+..]`); offset 2 is the load, and the recipe reads its displacement (operand 1). The REX and ModRM bytes of the load are wildcards (26.51 `mov rcx,[rax+..]`, 26.60 `mov rcx,[r14+..]`). Recipe 1 is the NetworkConnection destructor (0x1725dd0 in 26.51): after the two paused-packet arrays (`lea rdi,[rsi+..]; lea rcx,[rsi+..]; call; mov rcx,rdi; call; lea rcx,[rsi+..]; call`) and the sized delete of a 10h-byte member, it releases a shared_ptr (`mov rdi,[rsi+..]; test rdi,rdi; je; lock dec dword ptr [rdi+8]; ...`, both counts), then the weak_ptr control blocks at +110h, +100h, .. +C0h. That shared_ptr is mPeer, and the load reads its control block, 8 bytes after the pointer: offset 67 is that load, the recipe reads its displacement (0x120) and adds -8. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 128..512. Values: 0x118 in 26.51 and in 26.60. What changed in 6ad79ee4-17d51891a1119914: the class grew from 0x1B0 to 0x1E0 bytes at its end (the destructor first frees a std::string at +1B8h); the prefix to +1A8h, mPeer with it, did not move. Evidence for 0x118 on 26.60: (1) NetworkSystem.sendInternal loads the peer at +118h before the peer call, as on 26.51; (2) the destructor, found by the same anchor, releases the control block at +120h, and the inlined receive in NetworkSystem::_sortAndPacketizeEvents calls slot 8 through `mov rcx,[rdi+118h]` (see NetworkPeer.receivePacket)."

  recipe {
    step "ref" { entry = "NetworkSystem.sendInternal" }
    step "pattern" {
      bytes  = "75 ?? ?? 8B ?? ?? ?? ?? ?? 48 85 C9 74 ?? 44 8B 47 ?? 44 8B 4F ??"
      offset = 2
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D BE ?? ?? ?? ?? 48 8D 8E ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 F9 E8 ?? ?? ?? ?? 48 8D 8E ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B BE ?? ?? ?? ?? 48 85 FF 74 ?? F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 00 48 89 F9 FF 15"
      offset = 67
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -8 }
  }

  check "range" {
    min = 128
    max = 512
  }
}
