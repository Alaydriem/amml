entry "CommandRequestPacket.mCommand" {
  kind     = "field"
  notes    = "Offset of `mCommand` (std::string, the command line with its leading slash) in CommandRequestPacket: 30h on 26.51, 26.60, 26.62 and 26.63. CommandRequestPacket is ll::PayloadPacket<CommandRequestPacketPayload> (LeviLamina header): the 30h-byte Packet base, then the payload (mCommand 30h, mOrigin CommandOriginData 50h: mType u8 50h, mUUID 58h, mRequestId std::string 68h, mPlayerId i64 88h; mVersion i32 90h, mInternalSource bool 94h), then mSerializationMode 98h; the packet is A0h bytes (CommandRequestPacket.size). The packet id is 4Dh (vtable slot 1 is `mov eax,4Dh; ret`, slot 2 returns the string_view \"CommandRequestPacket\", length 14h). Found from MinecraftPackets::createPacket: case 4Dh of its jump table calls the make_shared function, which builds an empty payload (version 37h) on the stack and calls the packet constructor CommandRequestPacket(CommandRequestPacketPayload) (0x41988d0 on 26.63, 0x4198b70 on 26.62, 0x41989a0 on 26.60, 0x38e6990 on 26.51). The client sends the packet from MinecraftCommands::requestCommandExecution (0x577ac10 on 26.63): when the level is client side it builds the payload from the CommandContext, constructs the packet on its stack, sends it with PacketSender::send (vtable slot 2), stores MCRESULT_CommandRequestInitiated and destroys the packet inline. Recipe 0 is the packet constructor: the vptr store, the two zeroing stores of mCommand, then the move of the payload string into it (`lea rax,[rip+vtable]; mov [rcx],rax; movups [rcx+..],xmm0; movups [rcx+..],xmm0; movups xmm1,[rdx]; movups xmm2,[rdx+..]; movups [rcx+..],xmm2; movups [rcx+mCommand],xmm1; mov qword [rdx+..],..; mov qword [rdx+..],..; mov byte [rdx],..; lea r14,[rcx+..]`), reading the displacement of operand 0 of the fourth movups (offset 29). Recipe 1 starts in MinecraftCommands::requestCommandExecution, after the send: `mov eax,[rip+MCRESULT]; mov [rsi],eax; lea rax,[rip+vtable]; mov [rbp+..],rax; mov rax,[rbp+..]; cmp rax,..; jb` (the inline destructor of the local packet); the RIP target of the lea is the packet vtable, read_pointer gives slot 0 (the deleting destructor), and in that function the free of mCommand (`mov rax,[rsi+cap]; cmp rax,..; jb; mov rcx,[rsi+mCommand]`) gives the displacement of the second mov (offset 10). The free of mRequestId reads its capacity through rcx with a disp32 (`mov rax,[rcx+80h]`), so the rsi-based pattern matches once in the destructor. The two recipes start in two functions and share no pattern bytes. Every displacement, immediate, RIP offset and branch byte is a wildcard. Check: range 30h..60h (the Packet base is 30h bytes). The derive gives 30h on 26.51, 26.60, 26.62 and 26.63; the constructor and the destructor have the same code on all four."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 0F 11 41 ?? 0F 11 41 ?? 0F 10 0A 0F 10 52 ?? 0F 11 51 ?? 0F 11 49 ?? 48 C7 42 ?? ?? ?? ?? ?? 48 C7 42 ?? ?? ?? ?? ?? C6 02 ?? 4C 8D 71 ??"
      offset = 29
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "8B 05 ?? ?? ?? ?? 89 06 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 85 ?? ?? ?? ?? 48 83 F8 ?? 72"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "read_pointer" {}
    step "pattern" {
      bytes  = "48 8B 46 ?? 48 83 F8 ?? 72 ?? 48 8B 4E ??"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 48
    max = 96
  }
}

entry "CommandRequestPacket.size" {
  kind     = "field"
  notes    = "sizeof(CommandRequestPacket): A0h on 26.51, 26.60, 26.62 and 26.63. It is the size check of the back of the CommandRequestPacketLayout mirror: the 30h-byte Packet base + CommandRequestPacketPayload (mCommand 30h, mOrigin 50h: mType 50h, mUUID 58h, mRequestId 68h, mPlayerId 88h; mVersion 90h, mInternalSource 94h) + mSerializationMode (98h), aligned to 8 = A0h (LeviLamina header sum). The make_shared case of MinecraftPackets::createPacket allocates B0h (the packet and a 10h control block) on all four builds. Both recipes reach the packet vtable, read slot 0 (the deleting destructor, the first virtual of Packet) with read_pointer, and in that function read the immediate of the size passed to operator delete (`mov edx,A0h; mov rcx,rsi; call`). Recipe 0 reaches the vtable from the vptr store in the packet constructor (the anchor of CommandRequestPacket.mCommand recipe 0, offset 0, the RIP target of the lea). Recipe 1 reaches it from the inline destructor of the local packet in MinecraftCommands::requestCommandExecution (the anchor of CommandRequestPacket.mCommand recipe 1). The two recipes start in two functions and end in the same destructor bytes, so they are one fact about the size, checked from two places that must name the same table. Every displacement, immediate and RIP offset is a wildcard. Check: range 60h..200h. The derive gives A0h on 26.51, 26.60, 26.62 and 26.63."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 0F 11 41 ?? 0F 11 41 ?? 0F 10 0A 0F 10 52 ?? 0F 11 51 ?? 0F 11 49 ?? 48 C7 42 ?? ?? ?? ?? ?? 48 C7 42 ?? ?? ?? ?? ?? C6 02 ?? 4C 8D 71 ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "read_pointer" {}
    step "pattern" {
      bytes = "BA ?? ?? ?? ?? 48 89 F1 E8"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "8B 05 ?? ?? ?? ?? 89 06 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 85 ?? ?? ?? ?? 48 83 F8 ?? 72"
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "read_pointer" {}
    step "pattern" {
      bytes = "BA ?? ?? ?? ?? 48 89 F1 E8"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 96
    max = 512
  }
}
