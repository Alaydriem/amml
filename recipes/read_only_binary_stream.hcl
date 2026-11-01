entry "ReadOnlyBinaryStream.dtor" {
  kind     = "function"
  notes    = "ReadOnlyBinaryStream::~ReadOnlyBinaryStream() body (not the deleting destructor in slot 0): rcx the stream. It stores the ReadOnlyBinaryStream vtable and frees mOwnedBuffer (the MSVC std::string at +8: capacity at +20h, heap block when the capacity is 10h or more, then the empty-string reset). The deleting destructor in slot 0 holds the same code inline and then frees 48h bytes. The function has 536 direct callers; all but two are exception funclets, and the shape of the body is the common std::string member destructor, so both recipes are the two direct calls on the normal path. Recipe 0 is a function that destroys three locals in a row and jumps back (`lea rcx,[rbp+..]; lea rdx,[rbp+..]; call; lea rcx,[rbp+..]; call; lea rcx,[rbp+..]; call; jmp`): the third call. Recipe 1 is a function that calls a helper with the stream's owner and then destroys the stream (`mov byte ptr [rbp+..],..; lea rdx,[rbp-..]; lea r9,[rbp+..]; mov r8d,r15d; call; lea rcx,[rbp+..]; call`): the second call. Both are read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Both recipes are now a pattern_any, one alternative per shape, aligned on the destructor call (offset 36 and 33); where the 26.60 code is longer or shorter, the alternative starts inside the instruction before (at a ModRM or displacement byte, inside the .pdata range, so the lint decodes the match). Recipe 0 alternative 1 (26.60) destroys a local after `lea rcx,[rbp+..]; lea rdx,[rbp+..]; call; nop; lea rcx,[rbp+..]; call`; recipe 1 alternative 1 passes the owner in r8 from the frame (`mov r8,[rbp+..]`) and has a nop after the helper call. What changed in 6ad79ee4-17d51891a1119914 (26.60): nops after calls, frame offsets and argument loads at both normal-path call sites (the other callers are exception funclets: 536 and 589). The 26.60 function was accepted on two facts from the 26.51 function: the same body (the vtable store, then the owned std::string release and reset), and the same two normal-path callers (found with similar on the 26.51 callers, 0.82 and 1.0). Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? 48 8D 4D ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? E9",
        "48 8D 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8D 4D ?? E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? E9",
      ]
      offset = 36
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "C6 85 ?? ?? ?? ?? ?? 48 8D 55 ?? 4C 8D 8D ?? ?? ?? ?? 45 89 F8 E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8",
        "?? ?? 48 8D 55 ?? 4C 8D 8D ?? ?? ?? ?? 4C 8B 85 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8D 8D ?? ?? ?? ?? E8",
      ]
      offset = 33
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ReadOnlyBinaryStream.vtable" {
  kind     = "global"
  notes    = "The vtable of ReadOnlyBinaryStream (MCNAPI in LeviLamina, no RTTI): slot 0 the deleting destructor, slot 1 read(void*, uint64). Recipe 0 is the store at the start of ReadOnlyBinaryStream.dtor (`lea rax,[rip+..]; mov [rcx],rax`). Recipe 1 is the inline constructor in the network receive function that calls MinecraftPackets::createPacket: `lea rcx,[rip+..]; mov [rbp+..],rcx; lea rcx,[rbp+..]; movups [rcx],xmm6; mov qword ptr [rcx+10h],0; mov qword ptr [rbp+..],0Fh; mov [rbp+..],rax; mov [rbp+..],rdx` (the vptr, an empty owned string, then the view data and size; the read pointer and the overflow flag follow). Each reads the RIP target of the lea; the two recipes are in two functions. The 72-byte layout of that constructor (view at 28h and 30h, read pointer 38h, overflow flag 40h) is the ReadOnlyBinaryStreamLayout mirror. The RIP offsets are wildcards. Recipe 1 is now a pattern_any, one alternative per shape of the inline construction in the packet loop (the size comes from rdx on 26.51 and rdi on 26.60). The layout is the same on both builds: vtable, the owned std::string at +8, the view at +28h and +30h, the read pointer at +38h, the overflow flag at +40h. What changed in 6ad79ee4-17d51891a1119914 (26.60): the source register of the size; recipe 0 (from the destructor) agrees. Would have survived with a better recipe."

  recipe {
    step "ref" { entry = "ReadOnlyBinaryStream.dtor" }
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 01" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 0F 11 31 48 C7 41 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 89 95",
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 0F 11 31 48 C7 41 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 89 BD",
      ]
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
