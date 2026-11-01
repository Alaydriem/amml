entry "ClientNetworkHandler.vtable" {
  kind     = "global"
  notes    = "The primary vtable of ClientNetworkHandler (a NetEventCallback): 0xe7ae8d0 in 26.51. The class has no RTTI. Found from the string `virtual void __cdecl ClientNetworkHandler::handle(const NetworkIdentifier &, const CameraPacket &)`, loaded by one function (0x131b590); that function sits in two vtables (0xe7aed60 and 0xe7af5a0), and the CameraPacket dispatcher calls NetEventCallback slot 146 (see ClientNetworkHandler.handle.CameraInstructionPacket), so the vtables start at 0xe7ae8d0 and 0xe7af110. 0xe7ae8d0 is stored by the ClientNetworkHandler constructor (0x132bda0) and by its destructor (0x12f7c20); 0xe7af110 belongs to a derived class (its destructor 0x132d2e0 also stores a second base vtable at +240h), and both tables hold the same handle functions at the slots freecam uses. Recipe 0 is the constructor: after the NonOwnerReferences control block is made (`mov [rbx+8],rcx; mov [rbx+10h],rax`), `lea rax,[rip+..]; mov [rbx],rax; lea rax,[rbx+18h]; mov [rbp],rax; xorps xmm0,xmm0; movups [rbx+18h],xmm0; mov rax,[rsi+58h]; mov [rbx+18h],rax; mov rax,[rsi+60h]; mov [rbp+60h],rbx`; it reads the RIP target of the lea. Recipe 1 is the destructor: `mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; mov rbx,[rcx+168h]; lea rdi,[rbx+60h]; mov rcx,rdi; call`; it reads the RIP target of the lea. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and RIP offset is a wildcard."

  recipe {
    step "pattern" {
      bytes = "48 8D 05 ?? ?? ?? ?? 48 89 03 48 8D 43 ?? 48 89 45 ?? 0F 57 C0 0F 11 43 ?? 48 8B 46 ?? 48 89 43 ?? 48 8B 46 ?? 48 89 5D ??"
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8B 99 ?? ?? ?? ?? 48 8D 7B ?? 48 89 F9 E8"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "ClientNetworkHandler.handle.CameraInstructionPacket" {
  kind     = "vslot"
  notes    = "Slot of `void ClientNetworkHandler::handle(NetworkIdentifier const&, CameraInstructionPacket const&)` in the NetEventCallback vtable: 147 in 26.51 (this in rcx, the NetworkIdentifier in rdx, the packet in r8). The LeviLamina header order of NetEventCallback is not the vtable order (the header puts this overload at 175), so the slot comes from the packet dispatcher: MinecraftPackets::createPacket (0x16ad850) calls one function per packet id; the one for CameraInstructionPacket (0x16b7f90, the packet vtable 0xe945e00 whose getId is `mov eax,12Ch`) stores the dispatcher object 0x1185b470 at packet+20h, whose vtable slot 1 (0x1733a70) is `mov rcx,r8; mov r8,[r9]; mov rax,[rcx]; mov rax,[rax+498h]; mov r9,[guard]; jmp r9`, a call of slot 147. The override is 0x131b260 (it checks the client is the client of the level, then calls getCameraSystems through the ClientInstance at +58h and hands the packet to the camera system, 0x5679ad0). Recipe 0 is the override's body (`xorps xmm0,xmm0; movaps [rbp-10h],xmm0; mov rcx,[rax+38h]; test rcx,rcx; je; lock inc dword ptr [rcx+8]; mov r14,[rax+38h]; jmp; xor r14d,r14d; mov rbx,[rax+30h]; ...; test rbx,rbx; je; mov rax,[rdi]; mov rax,[rax+920h]; lea rdx,[rbp-50h]; mov rcx,rdi; call`, the copy of a shared_ptr and a Level call), then function_start_of and slot_of the ClientNetworkHandler vtable. Recipe 1 is the call of the camera system (`lea r9,[rbp-40h]; mov rcx,rsi; mov rdx,rax; mov r8,rbx; call`), then function_start_of and slot_of. The two anchors are in different parts of the override and share no pattern bytes; no other code calls the override (only the two vtables hold it), so both recipes end in the same vtable. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" {
      bytes = "0F 57 C0 0F 29 45 ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ?? 4C 8B 70 ?? EB ?? 45 31 F6 48 8B 58 ?? 48 89 5D ?? 4C 89 75 ?? 48 85 DB 0F 84 ?? ?? ?? ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 8D 55 ?? 48 89 F9 FF 15"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "4C 8D 4D ?? 48 89 F1 48 89 C2 49 89 D8 E8"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  check "range" {
    min = 120
    max = 180
  }
}

entry "ClientNetworkHandler.handle.CameraPresetsPacket" {
  kind     = "vslot"
  notes    = "Slot of `void ClientNetworkHandler::handle(NetworkIdentifier const&, CameraPresetsPacket const&)` in the NetEventCallback vtable: 145 in 26.51. From the dispatcher, as for CameraInstructionPacket: the createPacket function for CameraPresetsPacket (0x16b8120, packet vtable 0xe7c1030, getId `mov eax,0C6h`) stores the dispatcher object 0x1185b478, whose vtable slot 1 (0x1733a90) calls slot 145. The override is 0x131bc60: it gets the camera registry through the ClientInstance at +58h (slot 190, `[rax+5F0h]`) and hands the packet to the camera preset code (0x5679a30) with the handler's member at +158h. Recipe 0 is that hand-over (`add rbx,158h; mov [rsp+20h],rbx; mov rcx,rsi; mov rdx,rax; mov r8,r15; mov r9,r14; call`), then function_start_of and slot_of the ClientNetworkHandler vtable. Recipe 1 is the call before it (`mov r14,[rbp-8]; mov r15,[rbp-28h]; mov rax,[rdi]; mov rax,[rax+0E8h]; mov rcx,rdi; call [guard]`), then function_start_of and slot_of. The two anchors share no pattern bytes; only the two vtables hold the override. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" {
      bytes = "48 81 C3 ?? ?? ?? ?? 48 89 5C 24 ?? 48 89 F1 48 89 C2 4D 89 F8 4D 89 F1 E8"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "4C 8B 75 ?? 4C 8B 7D ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  check "range" {
    min = 120
    max = 180
  }
}

entry "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket" {
  kind     = "vslot"
  notes    = "Slot of `void ClientNetworkHandler::handle(NetworkIdentifier const&, NetworkChunkPublisherUpdatePacket const&)` in the NetEventCallback vtable: 135 in 26.51. From the dispatcher: the createPacket function for this packet (0x16b89d0, packet vtable 0xe83d920, getId `mov eax,79h`) stores the dispatcher object 0x1185b4b8, whose vtable slot 1 (0x1733b90) calls slot 135. The override is 0x1324dc0: it gets the local player (ClientInstance slot 31), then reads the packet's position (`lea rdx,[rsi+30h]; mov r8d,[rsi+3Ch]`) and radius. Recipe 0 is that read (`lea rdx,[rsi+30h]; mov r8d,[rsi+3Ch]; test al,al; je; cmp byte ptr [rdi+0B02h],1; mov eax,r8d`), then function_start_of and slot_of the ClientNetworkHandler vtable. Recipe 1 is the chunk math after it (`mov eax,r8d; shr eax,4; lea r9,[rsi+40h]; mov r10d,[rsi+30h]; mov r11d,[rsi+38h]; sar r10d,4; sar r11d,4`), then function_start_of and slot_of. The two anchors share no pattern bytes; only the two vtables hold the override. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" {
      bytes = "48 8D 56 ?? 44 8B 46 ?? 84 C0 0F 84 ?? ?? ?? ?? 80 BF ?? ?? ?? ?? ?? 44 89 C0"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "44 89 C0 C1 E8 ?? 4C 8D 4E ?? 44 8B 56 ?? 44 8B 5E ?? 41 C1 FA ?? 41 C1 FB ??"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  check "range" {
    min = 110
    max = 170
  }
}

entry "ClientNetworkHandler.handle.PrimitiveShapesPacket" {
  kind     = "vslot"
  notes    = "Slot of `void ClientNetworkHandler::handle(NetworkIdentifier const&, PrimitiveShapesPacket const&)` in the NetEventCallback vtable: 33 in 26.51 (the LeviLamina header puts it at 307). From the dispatcher: the createPacket function for PrimitiveShapesPacket (0x16bffd0, packet vtable 0xe891890, getId `mov eax,148h`) stores the dispatcher object 0x1185b7d8, whose vtable slot 1 (0x1734870) calls slot 33. The override is 0x130d5d0: it checks the level, looks up a component of the level (type id 0x451fd093, a 48-byte node), hands the packet to the shape system (0x6686330) and sets the packet's handled flag (`mov byte ptr [rsi+11h],1`). freecam calls it; it is not hooked. Recipe 0 is the component check (`cmp [rdi+58h],rcx; je; cmp byte ptr [rcx+2Ch],4; sete al; cmp dword ptr [rcx+28h],imm; setne dl; mov rcx,[rcx+10h]; test rcx,rcx; sete r8b; or r8b,al; or r8b,dl`), then function_start_of and slot_of the ClientNetworkHandler vtable. Recipe 1 is the null test before it (`mov rcx,[rdi+0C0h]; test rdx,rdx; sete al; test rcx,rcx; sete dl; or dl,al; jne`), then function_start_of and slot_of. The two anchors share no pattern bytes; only the two vtables hold the override. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot. What changed in 6ad79ee4-17d51891a1119914: the slot is 39 (26.51: 33), and two other handlers have the same game-rule check, so recipe 0 now runs to the call that follows the check (`cmp r8b,1; jne; mov rcx,[rbp-..]; call`; the other two branch to other code). Evidence for 39 on 26.60, from the 26.51 override: the two recipe anchors, drafted from its body, each match once on 26.60, in the same function, and that function sits in slot 39 of the ClientNetworkHandler vtable."

  recipe {
    step "pattern" {
      bytes = "48 39 4F ?? 74 ?? 80 79 ?? ?? 0F 94 C0 81 79 ?? ?? ?? ?? ?? 0F 95 C2 48 8B 49 ?? 48 85 C9 41 0F 94 C0 41 08 C0 41 08 D0 41 80 F8 ?? 75 ?? 48 8B 4D ?? E8"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 8B 8F ?? ?? ?? ?? 48 85 D2 0F 94 C0 48 85 C9 0F 94 C2 08 C2 0F 85"
    }
    step "function_start_of" {}
    step "slot_of" {
      vtable = "ClientNetworkHandler.vtable"
    }
  }

  check "range" {
    min = 8
    max = 60
  }
}

entry "ClientNetworkHandler.handle.ItemStackResponsePacket" {
  kind     = "vslot"
  notes    = "Slot of `void ClientNetworkHandler::handle(NetworkIdentifier const&, ItemStackResponsePacket const&)` in the NetEventCallback vtable: 194 on 26.51, the slot that `inspect packets` reads from the dispatcher of ItemStackResponsePacket (id 148). LegacyClientNetworkHandler does not override it: both the ClientNetworkHandler.vtable and the LegacyClientNetworkHandler.vtable hold the same body at 194. The body tests the level, asks the ClientInstance at +58h for the local player (slot 31) twice, calls a bool virtual of the packet (`mov rax,[rsi]; mov rax,[rax+58h]`), and tail-jumps to the item stack net manager of the player (`mov rcx,[rax+0AB8h]`) with the payload (`add rsi,30h`). Recipe 0 is that tail (`mov rcx,[rax+..]; add rsi,..; mov rdx,rsi; add rsp,..; pop rdi; pop rsi; jmp`), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the packet call before it (`mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; call [guard]; test al,al; je; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" { bytes = "48 8B 88 ?? ?? ?? ?? 48 83 C6 ?? 48 89 F2 48 83 C4 ?? 5F 5E E9" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 4F ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 186
    max = 202
  }
}

entry "ClientNetworkHandler.handle.ContainerOpenPacket" {
  kind     = "vslot"
  notes    = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, ContainerOpenPacket const&)` in the NetEventCallback vtable: 193 on 26.51, the slot that `inspect packets` reads from the dispatcher of ContainerOpenPacket (id 46). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override, which is the body to hook. The override gets the local player (ClientInstance at +58h, slot 31), tests the level at +258h and a level virtual, then copies the packet into a task (`mov ecx,[rsi+28h]; ...; movups xmm0,[rsi+30h]; movups xmm1,[rsi+3Ch]`: the payload at 30h). Recipe 0 is that copy (`mov ecx,[rsi+..]; mov [rbp+..],ecx; movups xmm0,[rsi+..]; movups xmm1,[rsi+..]; movups [rbp+..],xmm1; movups [rbp+..],xmm0; lea rcx,[rip+..]; mov [rbp+..],rcx; movups xmm0,[rsi+..]; movups xmm1,[rsi+..]`), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the level test before it (`mov rax,[rbx+..]; test rax,rax; je; cmp byte ptr [rax],..; jne; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; test r15,r15`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" { bytes = "8B 4E ?? 89 4D ?? 0F 10 46 ?? 0F 10 4E ?? 0F 11 4D ?? 0F 11 45 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 0F 10 46 ?? 0F 10 4E ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 8B 83 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 80 38 ?? 0F 85 ?? ?? ?? ?? 48 8B 8B ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 4D 85 FF" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 185
    max = 201
  }
}

entry "ClientNetworkHandler.handle.ContainerClosePacket" {
  kind     = "vslot"
  notes    = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, ContainerClosePacket const&)` in the NetEventCallback vtable: 192 on 26.51, the slot that `inspect packets` reads from the dispatcher of ContainerClosePacket (id 47). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override, which is the body to hook. The override gets the local player, tests the packet's mServerInitiatedClose (`cmp byte ptr [rsi+32h],1`), and then stores the packet's container type (`movzx r14d,byte ptr [rsi+31h]`) and a flag in the player before it calls a virtual of the player's member at +0D70h. Recipe 0 is the flag test (`cmp byte ptr [rsi+..],..; jne; test r14,r14; je; mov rbx,[rdi+..]; lock inc dword ptr [r14+..]`), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the stores (`movzx r14d,byte ptr [rsi+..]; mov byte ptr [rdi+..],..; mov [rdi+..],r14b; mov rcx,[rdi+..]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot."

  recipe {
    step "pattern" { bytes = "80 7E ?? ?? 0F 85 ?? ?? ?? ?? 4D 85 F6 0F 84 ?? ?? ?? ?? 48 8B 9F ?? ?? ?? ?? F0 41 FF 46" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "44 0F B6 76 ?? C6 87 ?? ?? ?? ?? ?? 44 88 B7 ?? ?? ?? ?? 48 8B 8F ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 184
    max = 200
  }
}

entry "ClientNetworkHandler.handle.MovePlayerPacket" {
  kind      = "vslot"
  msvc_name = "?$handle@LegacyClientNetworkHandler@@QEAAXAEBVNetworkIdentifier@@AEBVMovePlayerPacket@@@Z"
  notes     = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, MovePlayerPacket const&)` in the NetEventCallback vtable: 225 on 26.51, the slot that `inspect packets` reads from the dispatcher of MovePlayerPacket (id 19, size 0x80). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override (0x133e990 in 26.51), which is the body to hook (this in rcx, the id in rdx, the packet in r8; no return value). The override finds the actor by the packet's runtime id (+30h) and moves it; for a riding player it sets the rotation from the packet's mRot (+44h) and the head rotation from mYHeadRot (+4Ch). Recipe 0 is that riding branch (`lea rdx,[rsi+..]; mov rcx,rdi; call; movd xmm1,[rsi+..]; movd eax,xmm1`: mRot to Actor::setRotationWrapped, then the load of mYHeadRot), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the argument setup of the move call (`lea r8,[rsi+..]; xor eax,eax; cmp byte ptr [rsi+..],..; mov ecx,..; je; mov eax,[rsi+..]; mov ecx,[rsi+..]; lea r9,[rsi+..]; movzx edx,byte ptr [rsi+..]`: the teleport data, mRot and mResetPosition), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot. Values: 225 in 26.51, 233 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the NetEventCallback vtable gained slots before it, so the slot moved from 225 to 233; the override is shorter (0x36d bytes against 0x44d) but holds both anchors. Evidence for 233 on 26.60: (1) `inspect packets` reads slot 233 from the MovePlayerPacket dispatcher; (2) both anchors, drafted from the 26.51 override, match once on 26.60, in one function (0x6bd39a0), and that function sits in slot 233 of the LegacyClientNetworkHandler.vtable."

  recipe {
    step "pattern" { bytes = "48 8D 56 ?? 48 89 F9 E8 ?? ?? ?? ?? 66 0F 6E 4E ?? 66 0F 7E C8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "4C 8D 46 ?? 31 C0 80 7E ?? ?? B9 ?? ?? ?? ?? 74 ?? 8B 46 ?? 8B 4E ?? 4C 8D 4E ?? 0F B6 56 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 215
    max = 245
  }
}

entry "ClientNetworkHandler.handle.NetworkStackLatencyPacket" {
  kind      = "vslot"
  msvc_name = "?$handle@LegacyClientNetworkHandler@@QEAAXAEBVNetworkIdentifier@@AEBVNetworkStackLatencyPacket@@@Z"
  notes     = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, NetworkStackLatencyPacket const&)` in the NetEventCallback vtable: 86 on 26.51, the slot that `inspect packets` reads from the dispatcher of NetworkStackLatencyPacket (id 115, size 0x48). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override (0x134adb0 in 26.51, 0x69 bytes), which is the body to hook (this in rcx, the id in rdx, the packet in r8; no return value). The override returns at once unless mFromServer (+38h) is 1; else it copies the packet (mCreateTime +30h) into a packet on the stack and sends it back through the packet sender at this+78h (slot 4), then sets the packet's handled flag (`mov byte ptr [rsi+11h],1`). Recipe 0 is the head of the copy (`cmp byte ptr [r8+..],..; jne; mov eax,[r8+..]; mov [rsp+..],eax; movups xmm0,[r8+..]; movups xmm1,[r8+..]; movups [rsp+..],xmm1; movups [rsp+..],xmm0; lea rax,[rip+..]`, the packet vtable), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the send and the handled flag at its end (`mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rsp+..]; mov rsi,r8; call [guard]; mov byte ptr [rsi+..],..`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slot. Values: 86 in 26.51, 93 in 26.60. What changed in 6ad79ee4-17d51891a1119914: the NetEventCallback vtable gained slots before it, so the slot moved from 86 to 93; the override is the same code. Evidence for 93 on 26.60: (1) `inspect packets` reads slot 93 from the NetworkStackLatencyPacket dispatcher; (2) both anchors, drafted from the 26.51 override, match once on 26.60, in one function (0x6bdf3e0), and that function sits in slot 93 of the LegacyClientNetworkHandler.vtable."

  recipe {
    step "pattern" { bytes = "41 80 78 ?? ?? 75 ?? 41 8B 40 ?? 89 44 24 ?? 41 0F 10 40 ?? 41 0F 10 48 ?? 0F 11 4C 24 ?? 0F 11 44 24 ?? 48 8D 05 ?? ?? ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 8B 49 ?? 48 8B 01 48 8B 40 ?? 48 8D 54 24 ?? 4C 89 C6 FF 15 ?? ?? ?? ?? C6 46 ?? ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 76
    max = 104
  }
}

entry "ClientNetworkHandler.handle.PlayerListPacket" {
  kind     = "vslot"
  notes    = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, PlayerListPacket const&)` in the NetEventCallback vtable: 179 on 26.60, 171 on 26.51, the slots that `inspect packets` reads from the dispatcher of PlayerListPacket (id 63). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override (0x6bdd410 on 26.60, 0x1348a10 on 26.51), which is the body to hook (rcx the handler, rdx the NetworkIdentifier, r8 the packet). The override walks the payload's mEntries (vector at packet+30h, element PlayerListAddEntry.size, variant index PlayerListPacket.variantIndex), removes players on RemoveEntry, and on AddEntry compares the entry UUID with the local player UUID, tests the skin (PlayerListAddEntry.mSkin) and adds the entry to the player list; it loads the strings `[ClientSubId:{}]PlayerListPacket - Adding entity to player list: {} - PlayerEntityList count: {}` and `Creating skin info for: ...`. Recipe 0 is the AddEntry branch (`cmp eax,imm; jne; mov rax,[r14+..]; xor rax,[rbp+..]; mov rcx,[r14+..]; xor rcx,[rbp+..]; or rcx,rax; sete sil; lea rbx,[r14+..]; mov rcx,rbx; call`), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the loop end test and the variant index read (`cmp r14,[rbp+..]; je; movzx eax,byte ptr [r14+..]; test eax,eax; j..`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slots of both builds."

  recipe {
    step "pattern" { bytes = "83 F8 ?? 0F 85 ?? ?? ?? ?? 49 8B 46 ?? 48 33 45 ?? 49 8B 4E ?? 48 33 4D ?? 48 09 C1 40 0F 94 C6 49 8D 9E ?? ?? ?? ?? 48 89 D9 E8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "4C 3B B5 ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 41 0F B6 86 ?? ?? ?? ?? 85 C0 0F" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 167
    max = 187
  }
}

entry "ClientNetworkHandler.handle.PlayerSkinPacket" {
  kind     = "vslot"
  notes    = "Slot of `void LegacyClientNetworkHandler::handle(NetworkIdentifier const&, PlayerSkinPacket const&)` in the NetEventCallback vtable: 107 on 26.60, 100 on 26.51, the slots that `inspect packets` reads from the dispatcher of PlayerSkinPacket (id 93). The ClientNetworkHandler.vtable holds the empty default there; the LegacyClientNetworkHandler.vtable holds the override (0x6bdec10 on 26.60, 0x134a4c0 on 26.51), which is the body to hook (rcx the handler, rdx the NetworkIdentifier, r8 the packet). The packet's mUUID is at +30h and mSkin (SerializedSkinRef) at +40h on both builds. The override tests the level, tests the skin for validity (`lea rbx,[rsi+40h]; mov rcx,rbx; call`), runs the skin memory check with the UUID (`lea r14,[rsi+30h]`), then finds the player list entry by UUID (hash `imul rdx,rcx,1F1F1F1Fh; xor rdx,rax; and rdx,[r8+30h]`) and replaces its skin. Recipe 0 is the validity test (`test al,al; jne; lea rbx,[rsi+..]; mov rcx,rbx; call`), then function_start_of and slot_of the LegacyClientNetworkHandler.vtable. Recipe 1 is the UUID hash (`mov rax,[rsi+..]; mov rcx,[rsi+..]; imul rdx,rcx,imm32; xor rdx,rax; and rdx,[r8+..]`), then function_start_of and slot_of. The two anchors share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the slots of both builds."

  recipe {
    step "pattern" { bytes = "84 C0 0F 85 ?? ?? ?? ?? 48 8D 5E ?? 48 89 D9 E8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  recipe {
    step "pattern" { bytes = "48 8B 46 ?? 48 8B 4E ?? 48 69 D1 ?? ?? ?? ?? 48 31 C2 49 23 50 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "LegacyClientNetworkHandler.vtable" }
  }

  check "range" {
    min = 96
    max = 111
  }
}
