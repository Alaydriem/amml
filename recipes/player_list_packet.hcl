entry "PlayerListAddEntry.size" {
  kind     = "field"
  notes    = "The size of one element of PlayerListPacketPayload::mEntries (std::vector<std::variant<RemoveEntry, AddEntry>>, the vector at packet+30h): D8h on 26.60, B8h on 26.51 (26.60 added std::string mPlayFabId to AddEntry). It is the loop step of LegacyClientNetworkHandler::handle(PlayerListPacket) (ClientNetworkHandler.handle.PlayerListPacket: 0x6bdd410 on 26.60, 0x1348a10 on 26.51), which walks the entries with `add r14,imm32`; the vector size in the same function is divided by the same size (`sar rcx,3; imul` by the inverse of 1Bh on 26.60, 17h on 26.51). One recipe, a pattern_any with one alternative per build, both with the add at offset 0, read as the immediate: on 26.60 the add falls through into the end test and the variant index test (`add r14,imm32; cmp r14,[rbp+..]; je; movzx eax,byte ptr [r14+..]; test eax,eax; je`); on 26.51 it jumps back to the loop head before the AddEntry branch (`add r14,imm32; jmp; nop; cmp eax,1; jne; mov rax,[r14+..]; xor rax,[rbp+..]`). Each alternative matches once on its own build and never on the other. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C6 ?? ?? ?? ?? 4C 3B B5 ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 0F B6 86 ?? ?? ?? ?? 85 C0 0F 84",
        "49 81 C6 ?? ?? ?? ?? E9 ?? ?? ?? ?? ?? ?? ?? ?? 83 F8 ?? 0F 85 ?? ?? ?? ?? 49 8B 46 ?? 48 33 45",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}

entry "PlayerListPacket.variantIndex" {
  kind     = "field"
  notes    = "The offset of the std::variant index byte inside one PlayerListPacketPayload::mEntries element: D0h on 26.60, B0h on 26.51 (the AddEntry storage comes first and grew by 20h on 26.60). Index 0 is RemoveEntry, index 1 is AddEntry. LegacyClientNetworkHandler::handle(PlayerListPacket) (0x6bdd410 on 26.60, 0x1348a10 on 26.51) reads it at the loop head and branches on 0 and 1. One recipe: the end test and the index read (`cmp r14,[rbp+..]; je; movzx eax,byte ptr [r14+..]; test eax,eax; j..`; the last jump is je on 26.60 and jne on 26.51, so only its first opcode byte is kept), read as the displacement of the movzx. Every displacement and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "4C 3B B5 ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 0F B6 86 ?? ?? ?? ?? 85 C0 0F"
      offset = 13
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 248
  }
}

entry "PlayerListAddEntry.mSkin" {
  kind     = "field"
  notes    = "The offset of PlayerListPacketPayload::AddEntry::mSkin (SerializedSkinRef, a shared_ptr: element at +0, control block at +8) in one mEntries element: A8h on 26.60, 88h on 26.51 (26.60 added std::string mPlayFabId before it). The stable members are mUUID at +8 and mId (ActorUniqueID) at +18h on both builds. In the AddEntry branch of LegacyClientNetworkHandler::handle(PlayerListPacket) (0x6bdd410 on 26.60, 0x1348a10 on 26.51) the entry's UUID is compared with the local player's, then the skin is tested for validity (`lea rbx,[r14+mSkin]; mov rcx,rbx; call`). One recipe: `cmp eax,1; jne; mov rax,[r14+..]; xor rax,[rbp+..]; mov rcx,[r14+..]; xor rcx,[rbp+..]; or rcx,rax; sete sil; lea rbx,[r14+..]; mov rcx,rbx; call`, read as the displacement of the lea. The same bytes match once on both builds. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible."

  recipe {
    step "pattern" {
      bytes  = "83 F8 ?? 0F 85 ?? ?? ?? ?? 49 8B 46 ?? 48 33 45 ?? 49 8B 4E ?? 48 33 4D ?? 48 09 C1 40 0F 94 C6 49 8D 9E ?? ?? ?? ?? 48 89 D9 E8"
      offset = 32
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 96
    max = 192
  }
}

entry "PlayerListAddEntry.mName" {
  kind     = "field"
  notes    = "The offset of PlayerListPacketPayload::AddEntry::mName (std::string, the player name) in one mEntries element: 20h on 26.60 (after mUUID +8h and mId +18h). Proof: in LegacyClientNetworkHandler::handle(PlayerListPacket) (0x6bdd410 on 26.60) the local player's entry passes [entry+20h] to the setter 0x2bf940, which copies it into Player+BE8h, the name that the join and leave messages print. Recipe 0 is that call in the handler (`mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; nop; lea rdx,[r14+..]; mov rcx,rax; call`), read as the displacement of the lea. Recipe 1 is the copy of the entry into the client PlayerListEntry (0x4045d50 on 26.60, called from the handler): mUUID and mId are copied first, then the first string (`movups xmm0,[rsi+..]; movups [r14+..],xmm0; mov rax,[rsi+..]; mov [r14],rax; lea r9,[rsi+..]; lea rdi,[r14+..]; cmp rdi,r9; mov [rbp+..],r14; je`), read as the displacement of `lea r9,[rsi+..]`. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 49 8D 56 ?? 48 89 C1 E8"
      offset = 21
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 10 46 ?? 41 0F 11 46 ?? 48 8B 46 ?? 49 89 06 4C 8D 4E ?? 49 8D 7E ?? 4C 39 CF 4C 89 75 ?? 74"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "PlayerListAddEntry.mXUID" {
  kind     = "field"
  notes    = "The offset of PlayerListPacketPayload::AddEntry::mXUID (std::string) in one mEntries element: 40h on 26.60. This is an inference, not proven: it is the string member right after mName (+20h), which is the header order (mId, mUUID, mName, mXUID, mPlatformOnlineId, ...), and the handler collects [entry+40h] of each AddEntry into a list of strings that it passes on after the loop, which fits a list of XUIDs. To confirm, log [entry+40h] in game: an Xbox XUID is a decimal number of about 16 digits. Recipe 0 is that collection in LegacyClientNetworkHandler::handle(PlayerListPacket) (0x6bdd410 on 26.60): `mov rcx,[rdi+..]; lea r8,[r14+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r12; call [guard]; nop; lea rdx,[r14+..]; lea rcx,[rbp+..]; call`, read as the displacement of `lea rdx,[r14+..]`. Recipe 1 is the copy of the entry into the client PlayerListEntry (0x4045d50 on 26.60): the whole copy of the first string (mName), then the source of the second string (`... mov byte ptr [rdi+rbx],0; lea r9,[rsi+..]`), read as the displacement of that lea; the pattern starts at the mId store so that it matches only the second string of the four. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 8F ?? ?? ?? ?? 4D 8D 46 ?? 48 8B 01 48 8B 40 ?? 4C 89 E2 FF 15 ?? ?? ?? ?? 90 49 8D 56 ?? 48 8D 8D ?? ?? ?? ?? E8"
      offset = 28
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 06 4C 8D 4E ?? 49 8D 7E ?? 4C 39 CF 4C 89 75 ?? 74 ?? 48 8B 5E ?? 48 83 7E ?? ?? 72 ?? 4C 8B 4E ?? 49 8B 46 ?? 48 39 C3 76 ?? 48 89 F9 48 89 DA E8 ?? ?? ?? ?? 90 EB ?? 4D 89 F7 48 83 F8 ?? 72 ?? 49 8B 7F ?? 49 89 5F ?? 48 89 F9 4C 89 CA 49 89 D8 E8 ?? ?? ?? ?? C6 04 1F ?? 4C 8D 4E ??"
      offset = 93
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
