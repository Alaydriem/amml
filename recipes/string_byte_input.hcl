entry "StringByteInput.vtable" {
  kind     = "global"
  notes    = "The StringByteInput vtable on 26.51 (MCNAPI in LeviLamina, so there is no name to grade). It has the 11 IDataInput slots: slot 9 readBytesResult copies from this+10h (the string_view data) plus this+8 (the index) after a slot-10 bounds check, and slot 10 numBytesLeft is the leaf `mov rax,[rcx+18h]; sub rax,[rcx+8]; ret`; BigEndianStringByteInput shares its string reads and numBytesLeft. The game builds the 20h-byte object inline at 47 sites: `lea rcx,[rip+vtable]; mov [rbp+..],rcx; mov qword ptr [rbp+..],0; mov [rbp+..],data; mov [rbp+..],size`, then calls a reader with its address. Recipe 0 is a site with disp8 frame slots that calls NbtIo.read (`lea rcx,[rip+vtable]; mov [rbp+..],rcx; mov qword ptr [rbp+..],0; mov [rbp+..],rax; mov [rbp+..],rbx; lea r14,[rbp+..]; lea rdx,[rbp+..]; mov rcx,r14; call`). Recipe 1 is another site that calls NbtIo.read, with disp32 frame slots (`lea rcx,[rip+vtable]; ..; lea rcx,[rbp+..]; lea rdx,[rbp+..]; call`), one pattern_any with one alternative per shape of the code after the call: alternative 0 (26.51) `lea rcx,[rbp+..]; xorps xmm6,xmm6`; alternative 1 (26.60) `nop; mov rax,[rbp+..]`. Both read the lea target. Check section .rdata. The object layout is the same on both builds: vtable, +8 index (0), +10h data, +18h size. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site of recipe 0 (a reader other than NbtIo.read) no longer has that shape, and the 26.51 site of recipe 1 lost its `mov r14d,[rbx]` prefix (the disp32 shape alone has 2 matches on 26.51, so the code after the call picks the site). Both new sites were found by drafting every 26.60 site of the vtable and keeping the ones whose pattern matches once on each build and loads the 26.51 vtable there. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 8D 0D ?? ?? ?? ?? 48 89 4D ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 45 ?? 48 89 5D ?? 4C 8D 75 ?? 48 8D 55 ?? 4C 89 F1 E8" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 0F 57 F6",
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8B 85 ?? ?? ?? ??",
      ]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" { name = ".rdata" }
}
