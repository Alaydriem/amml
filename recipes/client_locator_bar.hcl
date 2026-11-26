entry "ClientLocatorBar.listHead" {
  kind     = "field"
  notes    = "The offset of the list head pointer of the waypoint map in ClientLocatorBar: 10h on 26.60 (the unordered_map<WaypointHandle, unique_ptr<ClientWaypoint>> at +8, its std::list head pointer 8 bytes into it; the bucket vector at +20h and the mask at +38h). The ClientLocatorBar packet function (0x1e8a280 on 26.60, called by ClientNetworkHandler::handle(LocatorBarPacket) with rcx = player+LocalPlayer.mClientLocatorBar) looks a waypoint handle up in the map (`imul r9,rcx,imm32; xor r9,rax; and r9,[rsi+..]; mov rdx,[rsi+..]; mov r10,[rsi+..]; shl r9,imm; mov r8,[r10+r9+..]; cmp r8,rdx; je short`). The function does the same lookup a second time; the first one is told apart by the packet type test before it (`cmp dword ptr [r12],imm8; jne short; mov rax,[rsp+..]; mov rcx,[rsp+..]; imul r9,rcx,imm32`), which the second lookup does not have. The je after the first compare is short on 26.60 to 26.63 and near on 1.26.70.26, so the pattern stops before it. The same lookup shape is in two other functions, so the recipe first finds the packet function by the ClientEntityWaypoint factory inside it (the anchor of ClientEntityWaypoint.vtable recipe 0), then scans that function for the lookup, read as the displacement of `mov rdx,[rsi+..]` (the list head, the end sentinel of the lookup). One recipe. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47: the first je of the lookup became a near jump; would have survived with a better recipe. Evidence on 6afa4e47: (1) the lookup drafted from the 26.63 packet function (anchor: the ClientEntityWaypoint factory, one match on each build) matches once in the 70.26 function, in the first of its two lookups; (2) the second lookup of the same function, a different place of the old function, loads the same displacement as its end sentinel (`mov rdx,[rsi+10h]; ...; cmp r8,rdx`) on both builds. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "C6 40 ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 08 49 8B 4C 24 ?? 48 89 48 ?? 0F 11 ?? ?? ?? ?? ?? C7 80 ?? ?? ?? ?? ?? ?? ?? ??"
    }
    step "pattern" {
      bytes  = "41 83 3C 24 ?? 75 ?? 48 8B 44 24 ?? 48 8B 4C 24 ?? 4C 69 C9 ?? ?? ?? ?? 49 31 C1 4C 23 4E ?? 48 8B 56 ?? 4C 8B 56 ?? 49 C1 E1 ?? 4F 8B 44 0A ?? 49 39 D0"
      offset = 31
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
  notes    = "The offset of the unique_ptr<ClientWaypoint> in one node of the ClientLocatorBar waypoint list: 20h (next +0, prev +8, the 16-byte WaypointHandle key +10h, then the value). Both recipes are in the ClientLocatorBar packet function (26.60 to 26.63: 0x1e89ea0 on 26.63; called by ClientNetworkHandler::handle(LocatorBarPacket)); it looks a waypoint handle up in the map twice. Recipe 0 is the end of the first lookup, after the key compare, as a pattern_any with one alternative per jump shape: alternative 0 (26.60 to 26.63) `je short; jmp short; cmp r8,rdx; je short; cmp qword ptr [r8+..],imm; jne` (the compares are short jumps), alternative 1 (1.26.70.26) `cmp r8,rdx; je near; cmp qword ptr [r8+..],imm; je short; mov [rsp+..],r14; mov qword ptr [rsp+..],imm` (the compares are near jumps and the value test is followed by the frame stores of the error path). Both read the displacement of the memory operand of the cmp (operand 0). Each alternative starts so that the cmp is at index 9 of its bytes. Recipe 1 is the end of the second lookup, which is the same in 26.63 and 1.26.70.26: `cmp r8,rdx; je near; mov r15,[r8+..]; test r15,r15; je near`, read as the displacement of the load. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47-dadc375797a3d31a: the jumps of the first lookup became near jumps; the old recipe 1 (the list walk of HudLocatorBarRenderer.render: `mov r12,[r12]; cmp r12,r13; je; mov rax,[r12+..]; test rax,rax; je`) is gone, because the render function walks the waypoint list through helper calls now (see HudLocatorBarRenderer.render), so recipe 1 now reads the second lookup of the packet function. On 70.26 the two recipes are therefore in one function (the old pair was in two); the two lookups share no pattern bytes. Class: would have survived with a better recipe (alternative for the jump shape), the second function anchor was lost (real code change in the render function). Evidence on 6afa4e47: (1) the first-lookup end drafted from the 26.63 function matches once in the 70.26 packet function (found as the function of the ClientEntityWaypoint factory, as in ClientLocatorBar.listHead); (2) the 26.63 second lookup, a different place of the old function, matches once in the 70.26 function and reads the same displacement. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern_any" {
      patterns = [
        "74 ?? EB ?? 49 39 D0 74 ?? 49 83 78 ?? ?? 0F 85 ?? ?? ?? ?? 41 80 7C 24",
        "49 39 D0 0F 84 ?? ?? ?? ?? 49 83 78 ?? ?? 74 ?? 4C 89 B4 24 ?? ?? ?? ?? 48 C7 84 24",
      ]
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 39 D0 0F 84 ?? ?? ?? ?? 4D 8B 78 ?? 4D 85 FF 0F 84 ?? ?? ?? ??"
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
