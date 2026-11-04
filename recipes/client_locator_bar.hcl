entry "ClientLocatorBar.listHead" {
  kind     = "field"
  notes    = "The offset of the list head pointer of the waypoint map in ClientLocatorBar: 10h on 26.60 (the unordered_map<WaypointHandle, unique_ptr<ClientWaypoint>> at +8, its std::list head pointer 8 bytes into it; the bucket vector at +20h and the mask at +38h). The ClientLocatorBar packet function (0x1e8a280 on 26.60, called by ClientNetworkHandler::handle(LocatorBarPacket) with rcx = player+LocalPlayer.mClientLocatorBar) looks a waypoint handle up in the map (`imul r9,rcx,imm32; xor r9,rax; and r9,[rsi+..]; mov rdx,[rsi+..]; mov r10,[rsi+..]; shl r9,imm; mov r8,[r10+r9+..]; cmp r8,rdx; je short`). The function does the same lookup a second time, which ends with a near je; the short je keeps the pattern to the first. The same lookup shape is in two other functions, so the recipe first finds the packet function by the ClientEntityWaypoint factory inside it (the anchor of ClientEntityWaypoint.vtable recipe 0), then scans that function for the lookup, read as the displacement of `mov rdx,[rsi+..]` (the list head, the end sentinel of the lookup). One recipe. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "C6 40 ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 08 49 8B 4C 24 ?? 48 89 48 ?? 0F 11 ?? ?? ?? ?? ?? C7 80 ?? ?? ?? ?? ?? ?? ?? ??"
    }
    step "pattern" {
      bytes  = "4C 23 4E ?? 48 8B 56 ?? 4C 8B 56 ?? 49 C1 E1 ?? 4F 8B 44 0A ?? 49 39 D0 74"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 48
  }
}

entry "ClientLocatorBar.nodeWaypoint" {
  kind     = "field"
  notes    = "The offset of the unique_ptr<ClientWaypoint> in one node of the ClientLocatorBar waypoint list: 20h on 26.60 (next +0, prev +8, the 16-byte WaypointHandle key +10h, then the value). Recipe 0 is the map lookup in the ClientLocatorBar packet function (0x1e8a280 on 26.60): after the key compare it tests the value (`cmp rcx,[r8+..]; je; cmp r8,r9; je; mov r8,[r8+..]; cmp rax,[r8+..]; je; jmp; cmp r8,rdx; je; cmp qword ptr [r8+..],imm; jne`), read as the displacement of the last cmp. Recipe 1 is the list walk in HudLocatorBarRenderer.render (0x5745690 on 26.60: `mov r12,[r12]; cmp r12,r13; je; mov rax,[r12+..]; test rax,rax; je`), read as the displacement of the load of the value. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "49 3B 48 ?? 74 ?? 4D 39 C8 74 ?? 4D 8B 40 ?? 49 3B 40 ?? 74 ?? EB ?? 49 39 D0 74 ?? 49 83 78 ?? ?? 0F 85"
      offset = 28
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 8B 24 24 4D 39 EC 74 ?? 49 8B 44 24 ?? 48 85 C0 74"
      offset = 9
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
