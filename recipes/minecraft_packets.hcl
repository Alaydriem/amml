entry "MinecraftPackets.createPacket" {
  kind      = "function"
  msvc_name = "?createPacket@MinecraftPackets@@SA?AV?$shared_ptr@VPacket@@@std@@W4MinecraftPacketIds@@@Z"
  notes     = "Address of the static `std::shared_ptr<Packet> MinecraftPackets::createPacket(MinecraftPacketIds)`: 0x16ad850 in 26.51. A static function that returns a class, so the shared_ptr buffer is in rcx and the id in edx; rax holds the buffer on return. The body is a jump table over the id (`mov rsi,rcx; cmp edx,161h; ja; mov eax,edx; lea rcx,[rip+table]; movsxd rax,[rcx+rax*4]; add rax,rcx; jmp rax`, an empty pointer for an unknown id: `xorps xmm0,xmm0; movups [rsi],xmm0`); each case calls a make_shared function for one packet type (CameraInstructionPacket 0x16b7f90, PrimitiveShapesPacket 0x16bffd0), which also stores the packet's dispatcher at packet+20h. Recipe 0 is that head, then function_start_of. Recipe 1 is a caller that range-checks an id and creates the packet (`cmp rbx,161h; je; lea rcx,[rbp+30h]; mov edx,ebx; call`), read as a branch target. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" {
      bytes = "48 89 CE 81 FA ?? ?? ?? ?? 77 ?? 89 D0 48 8D 0D ?? ?? ?? ?? 48 63 04 81 48 01 C8 FF E0 0F 57 C0 0F 11 06"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 81 FB ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 8D 4D ?? 89 DA E8"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
