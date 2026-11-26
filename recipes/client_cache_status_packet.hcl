entry "ClientCacheStatusPacket.vtable" {
  kind  = "global"
  notes = "The vtable of ClientCacheStatusPacket (packet id 81h, 38h bytes): 0xf360e10 on 6aed1906-b127418e74d2f2f9 (26.63), 0x13108df0 on 6afa4e47-dadc375797a3d31a (70.26), the same tables that `inspect packets` lists for the packet. The exe has no RTTI for the class. Slot 1 (getId) is `mov eax,81h; ret` and slot 2 (getName) returns the text `ClientCacheStatusPacket` (17h bytes) on both builds. Slot 5 is the write function: it takes the payload at this+30h and writes it as a bool under the field name `Is cache supported?`. Recipe 0 is the client send site that builds the packet on the stack and sends it: `lea rax,[rip+vtable]; mov [rbp+..],rax; setne byte ptr [rbp+..]; mov dword ptr [rbp+..],imm` (26.63 0x6b9dfdb, 70.26 0x8baf815; the flag is set from a test of the client's blob cache, then the packet goes to the packet sender, slot 2); offset 0, the RIP target of the lea. Recipe 1 is the write function: string_xref `Is cache supported?` with nth 0 (the functions that load the text, in address order: 26.63 the write 0x85fa000 and the schema builder 0x85fa7c0; 70.26 the write 0x7ec54e0, the schema builder 0x7ec5d10 and three serializer templates), then vtable_holding (the one table that holds the write). The nth 0 choice depends on link order: when another loader of the text comes first, vtable_holding refuses, because the schema builder and the templates are in no vtable. The two recipes share no anchor and no function. Every displacement, immediate and branch byte is a wildcard. Check: section .rdata. Class: new entry."

  recipe {
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 0F 95 45 ?? C7 45 ?? ?? ?? ?? ??" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "string_xref" {
      text = "Is cache supported?"
      nth  = 0
    }
    step "vtable_holding" {}
  }

  check "section" { name = ".rdata" }
}

entry "ClientCacheStatusPacket.mEnabled" {
  kind  = "field"
  notes = "Offset of mEnabled (bool, the client has a blob cache) in ClientCacheStatusPacket: 30h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26), the first payload byte after the 30h-byte Packet. The packet is 38h bytes; mSerializationMode (a dword, 1 when the game builds the packet) follows at +34h: the client send site stores `setne byte ptr [pkt+30h]; mov dword ptr [pkt+34h],1` and the createPacket factory (26.63 0x2383ba0, 70.26 0x302db60) stores `mov byte ptr [ctrl+40h],1; mov dword ptr [ctrl+44h],1` with the packet at control block+10h. Recipe 0 is the write function (slot 5), reached as in ClientCacheStatusPacket.vtable recipe 1 (string_xref `Is cache supported?`, nth 0): after the serialization-mode call it forms the payload address `bt rbx,imm; cmovb r14d,ebx; lea rbx,[r15+..]` (26.63 0x85fa030, 70.26 0x7ec5510); offset 9, the displacement of operand 1 of the lea. Recipe 1 is the read function, slot 16 of ClientCacheStatusPacket.vtable on both builds (26.63 0x85fac30, 70.26 0x7ec6180): after the same serialization-mode call it forms the payload address `mov r15d,eax; lea r?,[r12+..]; mov rax,[r12]; mov rax,[rax+..]; mov rcx,r12; call [guard]` (26.63 `lea rdi,[r12+30h]` 0x85fac8e, 70.26 `lea r14,[r12+30h]` 0x7ec61a9, so the REX and ModRM bytes are wildcards); vtable_slot 16, read_pointer, the pattern inside that function at offset 3, and the displacement of operand 1. Recipe 1 alone would be refused (vtable-only rule); it counts only as agreement with recipe 0. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 30h..40h. Class: new entry; the same value on both builds."

  recipe {
    step "string_xref" {
      text = "Is cache supported?"
      nth  = 0
    }
    step "pattern" {
      bytes  = "48 0F BA E3 ?? 44 0F 42 F3 49 8D 5F ??"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ClientCacheStatusPacket.vtable" }
    step "vtable_slot" { slot = 16 }
    step "read_pointer" {}
    step "pattern" {
      bytes  = "41 89 C7 ?? 8D ?? 24 ?? 49 8B 04 24 48 8B 40 ?? 4C 89 E1 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 48
    max = 64
  }
}
