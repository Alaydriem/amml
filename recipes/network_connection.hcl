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

entry "NetworkConnection.mType" {
  kind  = "field"
  notes = "Offset of mType (NetworkConnection::Type, a dword: 0 remote, 1 local) in NetworkConnection: B0h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26), right after mId (the B0h-byte NetworkIdentifier at 0). The constructor (26.63 0x2372710, 70.26 0x301ca70; called by NetworkSystem's onNewIncomingConnection and onNewOutgoingConnection, 26.63 0x2381ad0 / 0x2382400, 70.26 0x302bef0 / 0x302c710) takes rcx this, rdx the NetworkIdentifier, r8 the peer and r9b the local flag: it copies the id with `mov r8d,0B0h; call`, clears the weak_ptr peers from B8h, and later writes `mov dword ptr [this+0B0h],0; ...; test bl,bl; je; mov dword ptr [this+0B0h],1` (26.63 0x23729f9 / 0x2372a11, 70.26 0x301cd65 / 0x301cd7e). Recipe 0 is that store pair, a pattern_any with one alternative per build (26.63 stores through rdi after a 49 BE movabs; 70.26 stores through r14 with a 41 REX after a 48 BF movabs; each alternative matches once on its build and not on the other); offset 0, the displacement of operand 0 of the first store. Recipe 1 is the head of the constructor: `mov ebx,r9d; mov [rbp+..],r8; mov rdi,rcx; mov r8d,imm32; call` (26.63 0x237272d, 70.26 0x301ca8d), unique in the image and ending at the copy call; offset 10, the immediate of operand 1 of `mov r8d,0B0h`: the size of the id copy to this+0, so the end of mId and the start of the next member. Both recipes are in the constructor: on 26.63 a ConnectionCallbacks override also reads the type (`cmp dword ptr [r10+0B0h],1`, 0x238190a), but that override is not in the 70.26 callback tables (0x12e89b88, 0x12f457b0, 0x12f48c60), and a decode of every .pdata function in 0x2f00000..0x3100000 on 70.26 finds no other 4-byte access at [reg+0B0h] in a function that also touches the peer (+118h), the close flag (+199h) or the id type with mConnections (+0A8h, +0C0h); so the second recipe uses a second fact of the same function (search: sd-batch survey row N7). The two recipes share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 80h..200h. Class: new entry; the same value on both builds."

  recipe {
    step "pattern_any" {
      patterns = [
        "C7 87 ?? ?? ?? ?? ?? ?? ?? ?? 49 BE ?? ?? ?? ?? ?? ?? ?? ?? 84 DB 74 ?? C7 87 ?? ?? ?? ?? ?? ?? ?? ?? E9",
        "41 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 BF ?? ?? ?? ?? ?? ?? ?? ?? 84 DB 74 ?? 41 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? E9",
      ]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "44 89 CB 4C 89 45 ?? 48 89 CF 41 B8 ?? ?? ?? ?? E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 128
    max = 512
  }
}
