entry "PlayerSkinPacket.mUUID" {
  kind     = "field"
  notes    = "The offset of PlayerSkinPacketPayload::mUUID (mce::UUID, 16 bytes) in PlayerSkinPacket: 30h on 26.60. LegacyClientNetworkHandler::handle(PlayerSkinPacket) (ClientNetworkHandler.handle.PlayerSkinPacket, 0x6bdec10 on 26.60, r8 = the packet, kept in rsi) checks the skin, then passes the UUID and the skin to the skin memory check (`test al,al; je; lea r14,[rsi+..]; mov rcx,rdi; mov rdx,r14; mov r8,rbx; call`), then hashes the UUID to find the player list entry (`mov rax,[rsi+..]; mov rcx,[rsi+..]; imul rdx,rcx,imm32; xor rdx,rax; and rdx,[r8+..]`). Recipe 0 is the first, read as the displacement of the lea. Recipe 1 is the second, read as the displacement of `mov rax,[rsi+..]`. Both are in the one handler; they share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "84 C0 0F 84 ?? ?? ?? ?? 4C 8D 76 ?? 48 89 F9 4C 89 F2 49 89 D8 E8"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 46 ?? 48 8B 4E ?? 48 69 D1 ?? ?? ?? ?? 48 31 C2 49 23 50 ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 96
  }
}

entry "PlayerSkinPacket.mSkin" {
  kind     = "field"
  notes    = "The offset of PlayerSkinPacketPayload::mSkin (SerializedSkinRef, a shared_ptr: element at +0, control block at +8) in PlayerSkinPacket: 40h on 26.60. LegacyClientNetworkHandler::handle(PlayerSkinPacket) (0x6bdec10 on 26.60, r8 = the packet, kept in rsi) returns early when a virtual check of the client is true, then tests the skin for validity with the same function the PlayerListPacket handler uses (0x6bdee80 on 26.60): `mov rax,[rax+..]; call [guard]; test al,al; jne; lea rbx,[rsi+..]; mov rcx,rbx; call`. One recipe: that pattern, read as the displacement of the lea. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 48 8D 5E ?? 48 89 D9 E8"
      offset = 21
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 128
  }
}
