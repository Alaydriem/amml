entry "HashedString.layout" {
  kind     = "field"
  notes    = "Which layout the game's HashedString has: 0 for the 30h layout (the hash at +0, the text as a plain std::string at +8, the last-match pointer at +28h; 26.63, 6aed1906-b127418e74d2f2f9), 1 for the 38h layout (the hash at +0, the text as a std::variant<std::string, std::string_view> at +8 whose index byte is at +28h, 0 the string and 1 the view {pointer +8, length +10h}, and the last-match pointer at +30h; 26.70.26, 6afa4e47-dadc375797a3d31a). The loader's name reader (Item.mFullName, BlockType.mNameInfo.mFullName) selects its mirror by this value; any other value refuses. Both recipes scan ItemRenderer::renderGuiItemNew (`ref`), which compares the item name with static HashedStrings: on the 30h layout the compare is inline, on the 38h layout it is a call of an out-of-line HashedString operator==. Each is a pattern_any whose alternative 0 is the inline shape and alternative 1 the call shape, then the `alternative` step. Recipe 0 alternative 0 is the inline text compare of a plain std::string: the size at +18h, the capacity tested against 10h (the MSVC small-string limit; kept as a literal, because it is what shows a plain std::string), the pointer at +8 (`mov rax,[r12+..]; cmp qword ptr [r12+..],10h; jb; mov rcx,[r12+..]; cmp rax,r8; je; jmp; lea rcx,[r12+..]; cmp rax,r8; jne`); alternative 1 is the fourth call copy, with near je branches (`test rax,rax; je near; mov rcx,[rax]; test rcx,rcx; je near; add rcx,imm32; lea rdx,[rip+..]; call`). Recipe 1 alternative 0 is the inline last-match compare of the second copy (`cmp rax,[rip+..]; jne near; lea r9,[r12+..]; lea r10,[rip+..]; cmp [r12+..],r10`, the low byte of the jne distance kept, as in Item.mFullName, because the first copy jumps another distance); alternative 1 is the second call copy (`test rax,rax; je short 23h; mov rcx,[rax]; test rcx,rcx; je short; add rcx,imm32; lea rdx,[rip+..]; call`); the first copy has the same bytes with a je of 4Fh, so the distance byte is kept. Each alternative matches once in its own build and zero times in the other. The patterns end at the call. Evidence for the 38h layout on 26.70.26: the operator== those calls reach (the only function with `cmp [rcx+30h],rdx; jne; cmp [rdx+30h],rcx` after the hash compare) reads the index byte at +28h of each side (`movzx r8d,byte ptr [rdx+28h]`), takes the text as a pointer and a length at +8 / +10h for index 1, or as a std::string (capacity +20h against 10h, size +18h) for index 0, refuses any other index, and stores each side's address into the other's +30h on a match. Evidence for the 30h layout on 26.63: the inline compare reads the item name text as a std::string at item+128h (the HashedString at +120h: size +18h, capacity +20h, pointer +8) and the last match at +28h (item+148h). The check is range 0..1. Class: new entry."

  recipe {
    step "ref" { entry = "ItemRenderer.renderGuiItemNew" }
    step "pattern_any" {
      patterns   = ["49 8B 84 24 ?? ?? ?? ?? 49 83 BC 24 ?? ?? ?? ?? 10 72 ?? 49 8B 8C 24 ?? ?? ?? ?? 4C 39 C0 74 ?? EB ?? 49 8D 8C 24 ?? ?? ?? ?? 4C 39 C0 75 ??", "48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 08 48 85 C9 0F 84 ?? ?? ?? ?? 48 81 C1 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8"]
      literal_ok = [
        "0.16: the small-string capacity 10h of the MSVC std::string: the compare against it is what shows a plain std::string",
      ]
    }
    step "alternative" {}
  }

  recipe {
    step "ref" { entry = "ItemRenderer.renderGuiItemNew" }
    step "pattern_any" {
      patterns   = ["48 3B 05 ?? ?? ?? ?? 0F 85 AA ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ?? 4C 8D 15 ?? ?? ?? ?? 4D 39 94 24", "48 85 C0 74 23 48 8B 08 48 85 C9 74 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8"]
      literal_ok = [
        "0.9: low byte of the jne distance: the first copy of this block in the same function jumps a different distance",
        "1.4: the je distance of the second call copy (23h): the first copy, the same bytes otherwise, jumps 4Fh",
      ]
    }
    step "alternative" {}
  }

  check "range" {
    min = 0
    max = 1
  }
}
