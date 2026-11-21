entry "CreativeItemRegistry.mCreativeItems" {
  kind     = "field"
  notes    = "Offset of std::vector<CreativeItemEntry> CreativeItemRegistry::mCreativeItems: 18h on 26.51, the anchor of the CreativeItemEntry mirror. The base Bedrock::EnableNonOwnerReferences is 18h bytes (vptr and a shared_ptr control block), so the vector is at 18h. An ItemRegistryRef function that searches the creative items walks the vector: `mov rdi,[rbx+18h]; mov r14,[rbx+20h]; cmp rdi,r14; je; lea rcx,[rdi+28h]; mov rdx,rsi; call; test al,al; jne; add rdi,0B0h`. The recipe reads the displacement of the first load. It keeps the low byte of `lea rcx,[rdi+28h]` (CreativeItemEntry::mItemInstance) and of `add rdi,0B0h` (the entry stride, 176), so the anchor fails when the CreativeItemEntryLayout mirror (176 bytes, the item instance at 28h) no longer matches the exe. Range window 8..30h. The recipe is now a pattern_any, one alternative per shape of the walk: alternative 1 (26.60) walks with rbx (rdi on 26.51), has a one-byte nop before the loop and a nop after the call. Value 18h on both builds; the item instance at +28h and the stride 0B0h are the same, so the CreativeItemEntryLayout mirror holds on 26.60 (mirror check passed). What changed in 6ad79ee4-17d51891a1119914 (26.60): register allocation and nops only. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the walk keeps the 26.60 form (a vector walk with a nop before and after the call) but the loop uses r14 and r15 and a multi-byte nop (`nop word ptr [rax+rax]`), and the registry is loaded from another member of the caller's object, so alternative 2 is the 1.26.70.26 shape with the registers wildcarded; the member (18h), the item instance (28h) and the stride (0B0h) are the same, so the CreativeItemEntryLayout mirror holds. Class: would have survived with a better recipe. Evidence on 1.26.70.26: (1) alternative 2, drafted from the 26.63 walk, matches once; (2) the one direct caller of the function that holds it has the same call shape as the one direct caller of the 26.63 function (`lea rcx,[rbp+..]; lea rdx,[rbp-..]; call; nop; mov r14d,eax`), and each function has exactly one caller."

  recipe {
    step "pattern_any" {
      patterns   = [
        "48 8B 7B ?? 4C 8B 73 ?? 4C 39 F7 74 ?? 0F 1F 40 ?? 48 8D 4F 28 48 89 F2 E8 ?? ?? ?? ?? 84 C0 75 ?? 48 81 C7 B0 ?? ?? ??",
        "48 8B 5F ?? 4C 8B 77 ?? 4C 39 F3 74 ?? 90 48 8D 4B 28 48 89 F2 E8 ?? ?? ?? ?? 90 84 C0 75 ?? 48 81 C3 B0 ?? ?? ??",
        "4C 8B ?? ?? 4C 8B ?? ?? 4D 39 ?? 74 ?? 66 0F 1F 44 00 ?? 49 8D ?? 28 48 89 F2 E8 ?? ?? ?? ?? 90 84 C0 75 ?? 49 81 ?? B0 ?? ?? ??",
      ]
      literal_ok = [
        "0.20: lea rcx,[rdi+28h]: CreativeItemEntry::mItemInstance, held by the CreativeItemEntryLayout mirror this entry anchors",
        "0.36: add rdi,0B0h: the CreativeItemEntry stride (176), held by the CreativeItemEntryLayout mirror this entry anchors",
        "1.17: lea rcx,[rbx+28h]: the same member in the 26.60 shape",
        "1.34: add rbx,0B0h: the same stride in the 26.60 shape",
        "2.22: lea rcx,[r14+28h]: the same member in the 1.26.70.26 shape",
        "2.39: add r14,0B0h: the same stride in the 1.26.70.26 shape",
      ]
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
