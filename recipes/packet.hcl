entry "Packet.read" {
  kind     = "vslot"
  notes    = "Slot of Bedrock::Result<void> Packet::read(ReadOnlyBinaryStream&), the one-argument overload, in the packet vtables: 8 on 26.51 (the survey's 9 is the other overload). The Result comes back through a hidden pointer: rcx the packet, rdx the result, r8 the stream. All 228 packet vtables that `inspect packets` lists hold one shared body at slot 8; slot 9 is each packet's own read(ReadOnlyBinaryStream&, cereal::ReflectionCtx const&). MSVC lays each overload pair out in the reverse of the LeviLamina header order: slot 7 is the leaf write(stream, ctx) that forwards to slot 5 (writeWithSerializationMode) with an empty optional, slot 6 is the one-argument write; slot 8 calls slot 15 (_read(stream), three arguments) and slot 9 calls slot 16 (_read(stream, ctx), four arguments). Recipe 0 is the shared body of slot 8: `mov rdi,r8; mov rsi,rdx; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; call [guard]` (the _read(stream) call), the displacement read as a slot, minus 7 (header distance from read(stream) to _read(stream)). Recipe 1 is Packet::readNoHeader(stream, ctx, subid): it stores the sub-client id (`mov rax,[rbp+..]; movzx eax,byte ptr [rax]; mov [rcx+10h],al`) and calls read(stream, ctx) (`mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp-..]; call [guard]`), the displacement read as a slot, minus 1. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 4..12. Recipe 1 is now a pattern_any, one alternative per shape of the read(stream, ctx) call after the sub-client id store, the slot load at offset 16 in both: alternative 0 (26.51) Packet::readNoHeader; alternative 1 (26.60) the same code inlined into NetworkSystem::_sortAndPacketizeEvents (`mov r9,[r15+..]; mov [rcx+10h],r12b; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]; lea r8,[rbp+..]; call [guard]`); alternative 1 starts at the last two displacement bytes of the instruction before it, inside the .pdata range, so that the slot load sits at the same offset. Value 8 on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): readNoHeader is inlined into its only caller (the function that loads NetworkSystem::_sortAndPacketizeEvents on both builds); recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "4C 89 C7 48 89 D6 48 8B 01 48 8B 40 ?? 48 8D 95 ?? ?? ?? ?? FF 15"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" { value = -7 }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 85 ?? ?? ?? ?? 0F B6 00 88 41 ?? 48 8B 01 48 8B 40 ?? 48 8D 55 ?? FF 15",
        "?? ?? 4D 8B 8F ?? ?? ?? ?? 44 88 61 ?? 48 8B 01 48 8B 40 ?? 48 8D 95 ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? FF 15",
      ]
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" { value = -1 }
  }

  check "range" {
    min = 4
    max = 12
  }
}
