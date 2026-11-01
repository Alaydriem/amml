entry "LevelData.mLevelName" {
  kind     = "field"
  notes    = "Offset of std::string LevelData::mLevelName: 2A8h on 26.51, the anchor of LevelDataLayout. LevelData has no vtable; the header sum is 680 (2A8h). The LevelData tag reader is the only function that loads the \"LevelName\" string; it looks the key up (length 9) and assigns the string to [rsi+2A8h] (`cmove r9,rax; lea r14,[rsi+2A8h]; cmp r14,r9`). It stores mLastSaved at [rsi+360h] just before, which also matches the header sum. The recipe reads the lea displacement. Range window 2A0h..2B8h."

  recipe {
    step "string_xref" { text = "LevelName" }
    step "pattern" {
      bytes  = "4C 0F 44 C8 4C 8D B6 ?? ?? ?? ?? 4D 39 CE"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 672
    max = 696
  }
}
