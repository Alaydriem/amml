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

entry "LevelData.mCommandsEnabled" {
  kind     = "field"
  notes    = "Offset of bool LevelData::mCommandsEnabled: 4C9h on 26.62 and 26.63. The name is ours: the exe has no symbols, so Ghidra shows no member names. The LevelData tag reader (26.63 0x106b470, 26.62 0x106b710) is the only function that loads the string \"commandsEnabled\"; both recipes start there with string_xref. The reader reads one tag after another with the same code: a key build (`lea rbx,[key]; mov [rbp-30h],rbx; mov qword [rbp-28h],<key length>`), a lookup call (`call; nop; test rax,rax; je`), the value byte (`cmp byte [rax+8],0; setne al`) and a store `mov [rsi+<member>],al`. Recipe 0 reads the store after the key of length 15 (the length of \"commandsEnabled\"), up to the next key of length 13 (the length of \"cheatsEnabled\"); the two key lengths are kept, because the tag blocks have the same code and only the key lengths tell them apart. Recipe 1 is the block of the next key (length 13): after its lookup the bytes are `cmp byte [rax+8],0; setne al; jmp; movzx eax,byte [rsi+4C9h]; jmp; xor eax,eax; mov [rsi+4C8h],al`; the movzx is reached from the `je` after the first lookup call of that block, so when that call returns null the reader stores the byte at 4C9h into 4C8h. The recipe reads operand 1 (the displacement) of the movzx. The pattern starts at the cmp so that it has one match in the whole exe. The two recipes share no bytes. Every REX, ModRM and SIB byte is a wildcard (so a change of the LevelData register, rsi on 26.62 and 26.63, does not break them); the patterns assume the disp32 forms that 26.62 and 26.63 use, and the operand step decodes the real instruction. Range window 4A0h..4F0h. 26.51 and 26.60 were not derived: their exes are not on this disk. Which of the two flags the \"Activate Cheats\" toggle sets is not established; read both."

  recipe {
    step "string_xref" { text = "commandsEnabled" }
    step "pattern" {
      bytes      = "?? C7 ?? ?? 0F 00 00 00 ?? 8D ?? ?? ?? 89 ?? E8 ?? ?? ?? ?? 90 ?? 85 ?? 74 ?? 80 ?? ?? ?? 0F 95 ?? EB ?? 31 ?? 88 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? 0D 00 00 00"
      offset     = 37
      literal_ok = [
        "4: key length 15 (\"commandsEnabled\"): every bool tag of the reader has the same code, only the key length tells them apart (26.62, 26.63)",
        "5: upper bytes of the key length 15",
        "6: upper bytes of the key length 15",
        "7: upper bytes of the key length 15",
        "58: key length 13 (\"cheatsEnabled\"), the next key: makes the match unique (26.62, 26.63)",
        "59: upper bytes of the key length 13",
        "60: upper bytes of the key length 13",
        "61: upper bytes of the key length 13",
      ]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "string_xref" { text = "commandsEnabled" }
    step "pattern" {
      bytes  = "80 ?? ?? ?? 0F 95 ?? EB ?? 0F B6 ?? ?? ?? ?? ?? EB ?? 31 ?? 88 ?? ?? ?? ?? ??"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1184
    max = 1264
  }
}

entry "LevelData.mCheatsEnabled" {
  kind     = "field"
  notes    = "Offset of bool LevelData::mCheatsEnabled: 4C8h on 26.62 and 26.63, one byte before LevelData.mCommandsEnabled. The name is ours: the exe has no symbols, so Ghidra shows no member names. Both recipes start in the LevelData tag reader (26.63 0x106b470, 26.62 0x106b710) with string_xref \"commandsEnabled\" (the only function that loads it; \"cheatsEnabled\" is loaded by two functions). The member is the store of the tag block whose key has length 13 (the length of \"cheatsEnabled\"). Recipe 0 is the end of that block: `cmp byte [rax+8],0; setne al; jmp; movzx eax,byte [rsi+4C9h]; jmp; xor eax,eax; mov [rsi+4C8h],al`; when the first lookup call of the block returns null, its `je` goes to the movzx, so the byte at 4C9h is stored here. The recipe reads operand 0 (the displacement) of the store. Recipe 1 reads the same store from the code after it: the key of length 20 and its tag block, then the next store and the key of length 21; the two key lengths are kept because the tag blocks have the same code and only the key lengths tell them apart. The two recipes share only the store instruction that both read, and anchor on different neighbours (the movzx before it, the next two tag blocks after it). Every REX, ModRM and SIB byte is a wildcard (so a change of the LevelData register, rsi on 26.62 and 26.63, does not break them); the patterns assume the disp32 forms that 26.62 and 26.63 use, and the operand step decodes the real instruction. Each pattern has one match in the whole exe on 26.62 and 26.63. Range window 4A0h..4F0h. 26.51 and 26.60 were not derived: their exes are not on this disk. Which of the two flags the \"Activate Cheats\" toggle sets is not established; read both."

  recipe {
    step "string_xref" { text = "commandsEnabled" }
    step "pattern" {
      bytes  = "80 ?? ?? ?? 0F 95 ?? EB ?? 0F B6 ?? ?? ?? ?? ?? EB ?? 31 ?? 88 ?? ?? ?? ?? ??"
      offset = 20
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "string_xref" { text = "commandsEnabled" }
    step "pattern" {
      bytes      = "88 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? 14 00 00 00 ?? 8D ?? ?? ?? 89 ?? E8 ?? ?? ?? ?? 90 ?? 85 ?? 74 ?? ?? 89 ?? ?? ?? C7 ?? ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? 89 ?? E8 ?? ?? ?? ?? 90 ?? 85 ?? 74 ?? 80 ?? ?? ?? 0F 95 ?? EB ?? 31 ?? 88 ?? ?? ?? ?? ?? ?? 8D ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? C7 ?? ?? 15 00 00 00"
      literal_ok = [
        "21: key length 20, the key after \"cheatsEnabled\": every bool tag of the reader has the same code, only the key lengths tell them apart (26.62, 26.63)",
        "22: upper bytes of the key length 20",
        "23: upper bytes of the key length 20",
        "24: upper bytes of the key length 20",
        "105: key length 21, the key after that: makes the match unique (26.62, 26.63)",
        "106: upper bytes of the key length 21",
        "107: upper bytes of the key length 21",
        "108: upper bytes of the key length 21",
      ]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1184
    max = 1264
  }
}
