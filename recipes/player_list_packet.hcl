entry "PlayerListAddEntry.size" {
  kind     = "field"
  notes    = "The size of one element of PlayerListPacketPayload::mEntries (std::vector<std::variant<RemoveEntry, AddEntry>>, the vector at packet+30h): D8h on 26.60 and later, B8h on 26.51 (26.60 added std::string mPlayFabId to AddEntry). It is the loop step of LegacyClientNetworkHandler::handle(PlayerListPacket) (ClientNetworkHandler.handle.PlayerListPacket), which walks the entries with `add reg,imm32`; the vector size in the same function is divided by the same size (`sar rcx,3; imul` by the inverse of 1Bh on 26.60 and later, 17h on 26.51). One recipe, a pattern_any with one alternative per loop shape, each with the add read as the immediate. Alternative 0 (26.60 to 26.63): the add falls through into the end test and the variant index test (`add r14,imm32; cmp r14,[rbp+..]; je; movzx eax,byte ptr [r14+..]; test eax,eax; je`). Alternative 1 (26.51): the add jumps back to the loop head before the AddEntry branch (`add r14,imm32; jmp; nop; cmp eax,1; jne; mov rax,[r14+..]; xor rax,[rbp+..]`). Alternative 2 (1.26.70.26): the loop head holds the entry pointer in a register and the add ends the entry with a jump back, the exit block (`mov rax,[rbp+..]; cmp rax,[rbp+..]; je`) follows it; the anchor is the last add, the one before the exit block: `add r15,imm32; jmp; mov rax,[rbp+..]; cmp rax,[rbp+..]; je`. Alternative 2 keeps the register byte of `add r15` (C7): the 26.63 handler has the same exit block after `add r14` (C6), and the byte is what keeps the two alternatives apart on that build. Each alternative matches once on its own build and never on the other builds. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47: the loop was rebuilt (register r14 became r15, the loop head became a jump target, the end test moved off the add); would have survived with a better recipe. Evidence on 6afa4e47: the same handler (reached by the LegacyClientNetworkHandler.vtable slot that inspect packets gives, 184 against 179) has the exit add at the end of the entry loop with the same immediate as the vector size divisor (imul by the inverse of 1Bh, so 1Bh*8 = D8h), and the exit-add shape of the 26.63 handler (`add; jmp; mov rax,[rbp+..]; cmp rax,[rbp+..]; je`, one match there) matches once in the new handler, at its last add."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C6 ?? ?? ?? ?? 4C 3B B5 ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 0F B6 86 ?? ?? ?? ?? 85 C0 0F 84",
        "49 81 C6 ?? ?? ?? ?? E9 ?? ?? ?? ?? ?? ?? ?? ?? 83 F8 ?? 0F 85 ?? ?? ?? ?? 49 8B 46 ?? 48 33 45",
        "49 81 C7 ?? ?? ?? ?? E9 ?? ?? ?? ?? 48 8B 85 ?? ?? ?? ?? 48 3B 85 ?? ?? ?? ?? 0F 84",
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
  notes    = "The offset of the std::variant index byte inside one PlayerListPacketPayload::mEntries element: D0h on 26.60 and later, B0h on 26.51 (the AddEntry storage comes first and grew by 20h on 26.60). Index 0 is RemoveEntry, index 1 is AddEntry. LegacyClientNetworkHandler::handle(PlayerListPacket) (ClientNetworkHandler.handle.PlayerListPacket) reads it at the loop head and branches on 0 and 1. One recipe: the end test and the index read (`cmp reg,[rbp+..]; je; movzx eax,byte ptr [reg+..]; test eax,eax; j..`; the last jump is je on 26.60 and later and jne on 26.51, so only its first opcode byte is kept), read as the displacement of the movzx. The register bytes of the cmp (ModRM) and of the movzx are wildcards, because the loop register changed from r14 to r15 on 1.26.70.26. Every displacement and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47: only the loop register (r14 became r15, rbp stays); would have survived with a better recipe. Evidence on 6afa4e47: the pattern drafted from the 26.63 loop head matches once in the 70.26 handler (reached by the LegacyClientNetworkHandler.vtable slot that inspect packets gives, 184 against 179), and the same value D0h is the offset that the same handler adds 8 to for the element size (the variant index byte is the last byte before the end of the D8h element)."

  recipe {
    step "pattern" {
      bytes  = "4C 3B ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 0F B6 ?? ?? ?? ?? ?? 85 C0 0F"
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
  notes    = "The offset of PlayerListPacketPayload::AddEntry::mSkin (SerializedSkinRef, a shared_ptr: element at +0, control block at +8) in one mEntries element: A8h on 26.60 and later, 88h on 26.51 (26.60 added std::string mPlayFabId before it). The stable members are mUUID at +8 and mId (ActorUniqueID) at +18h on all builds. In the AddEntry branch of LegacyClientNetworkHandler::handle(PlayerListPacket) (ClientNetworkHandler.handle.PlayerListPacket) the entry's UUID is compared with the local player's, then the skin is tested for validity (`lea rbx,[reg+mSkin]; mov rcx,rbx; call`). One recipe: the end of that compare and the lea (`or rcx,rax; sete sil; lea rbx,[reg+..]; mov rcx,rbx; call`), read as the displacement of the lea. The compare's own loads are not in the pattern: their form changed on 1.26.70.26 (the local UUID moved from a disp8 to a disp32 frame slot and the loop register from r14 to r15). The register byte of the lea is a wildcard; every displacement, immediate and branch byte is a wildcard. The same bytes match once on 26.60 to 26.63 and on 1.26.70.26, in the one handler. The range check keeps the value plausible. What changed in 6afa4e47: the loads before the lea (register and frame displacement width); would have survived with a better recipe. Evidence on 6afa4e47: the pattern drafted from the 26.63 handler tail of the UUID compare matches once in the 70.26 handler (reached by the LegacyClientNetworkHandler.vtable slot that inspect packets gives, 184 against 179), and the lea's result is passed to the same call position as on 26.63: the AddEntry branch passes rbx (entry+mSkin) in r8 to the same call that follows the skin test (`mov rcx,rdi; mov r14,rdx; mov r8,rbx; call`) on both builds."

  recipe {
    step "pattern" {
      bytes  = "48 09 C1 40 0F 94 C6 49 8D ?? ?? ?? ?? ?? 48 89 D9 E8"
      offset = 7
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

entry "PlayerListPacket.mEntries" {
  kind     = "field"
  notes    = "The offset of PlayerListPacketPayload::mEntries (std::vector<std::variant<RemoveEntry, AddEntry>>: begin at +0, end at +8) in PlayerListPacket: 30h on 26.60 and later. LegacyClientNetworkHandler::handle(PlayerListPacket) (ClientNetworkHandler.handle.PlayerListPacket) loads the begin and the end and divides the byte span by the element size (PlayerListAddEntry.size), then reloads both after it reserves its own list. Recipe 0 is the first load (`mov rax,[rbp+..] (the packet); mov r14,[rax+..]; mov rdx,[rax+..]; mov rcx,rdx; sub rcx,r14; sar rcx,imm; mov rax,imm64`), read as the displacement of `mov r14,[rax+..]`; the ModRM bytes of the load and of the sub are wildcards (r14 became r15 on 1.26.70.26). Recipe 1 is the reload after the reserve call (`nop; mov rax,[rbp+..]; mov r14,[rax+..]; mov rax,[rax+..]; mov [rbp+..],rax`), read the same way, as a pattern_any with one alternative per tail: alternative 0 (26.60 to 26.63) ends with `jmp`, alternative 1 (1.26.70.26) with the two aligned loads of the loop constants (`movaps xmm6,[rip+..]; movaps xmm7,[rip+..]`). Both are in the one handler; they share no pattern bytes beyond the packet load. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47: the loop register (r14 became r15), and the jump after the reload became a fall-through into the loop head; would have survived with a better recipe. Evidence on 6afa4e47: the pattern of recipe 0, drafted from the 26.63 first load, matches once in the 70.26 handler (reached by the LegacyClientNetworkHandler.vtable slot that inspect packets gives, 184 against 179), and the reload of recipe 1 in the 70.26 handler reads the same displacement (30h, end at 38h) at the place where the 26.63 handler reloads after the reserve call."

  recipe {
    step "pattern" {
      bytes  = "48 8B 85 ?? ?? ?? ?? 4C 8B ?? ?? 48 8B 50 ?? 48 89 D1 4C 29 ?? 48 C1 F9 ?? 48 B8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "90 48 8B 85 ?? ?? ?? ?? 4C 8B 70 ?? 48 8B 40 ?? 48 89 85 ?? ?? ?? ?? EB",
        "90 48 8B 85 ?? ?? ?? ?? 4C 8B ?? ?? 48 8B 40 ?? 48 89 85 ?? ?? ?? ?? 0F 28 35 ?? ?? ?? ?? 0F 28 3D ?? ?? ?? ??",
      ]
      offset = 8
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

entry "PlayerListAddEntry.mUUID" {
  kind     = "field"
  notes    = "The offset of the mce::UUID (16 bytes) in one PlayerListPacketPayload::mEntries element: 8h on 26.60. Both alternatives keep the UUID at the same place: the handler compares [entry+8h] and [entry+10h] with a player's UUID in the AddEntry branch and reads them in the RemoveEntry branch. Recipe 0 is the copy of the entry into the client PlayerListEntry (0x4045d50 on 26.60, called once from LegacyClientNetworkHandler::handle(PlayerListPacket)): mUUID and mId are copied first (`movups xmm0,[rsi+..]; movups [r14+..],xmm0; mov rax,[rsi+..]; mov [r14],rax; lea r9,[rsi+..]; lea rdi,[r14+..]; cmp rdi,r9`), read as the displacement of the movups load. Recipe 1 is the UUID compare in the AddEntry branch of the handler (0x6bdd410 on 26.60: `mov rdx,[r14+..]; xor rdx,[r12+..]; mov r8,[r12+..]; xor r8,[r14+..]; or r8,rdx; jne`), read as the displacement of the first load. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "0F 10 46 ?? 41 0F 11 46 ?? 48 8B 46 ?? 49 89 06 4C 8D 4E ?? 49 8D 7E ?? 4C 39 CF"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 56 ?? 49 33 94 24 ?? ?? ?? ?? 4D 8B 84 24 ?? ?? ?? ?? 4D 33 46 ?? 49 09 D0 0F 85"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 32
  }
}

entry "PlayerListAddEntry.mId" {
  kind     = "field"
  notes    = "The offset of PlayerListPacketPayload::AddEntry::mId (ActorUniqueID, an int64) in one mEntries element: 18h on 26.60, after mUUID (+8h). One recipe: the copy of the entry into the client PlayerListEntry (0x4045d50 on 26.60, called once from LegacyClientNetworkHandler::handle(PlayerListPacket)), which copies mUUID and then mId into the first word of the client entry (`movups xmm0,[rsi+..]; movups [r14+..],xmm0; mov rax,[rsi+..]; mov [r14],rax; lea r9,[rsi+..]; lea rdi,[r14+..]; cmp rdi,r9`), read as the displacement of `mov rax,[rsi+..]`. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "0F 10 46 ?? 41 0F 11 46 ?? 48 8B 46 ?? 49 89 06 4C 8D 4E ?? 49 8D 7E ?? 4C 39 CF"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}
