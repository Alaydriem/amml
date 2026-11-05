entry "ShulkerBoxBlockItem.vtable" {
  kind     = "global"
  notes    = "The vtable of ShulkerBoxBlockItem (no RTTI; the object is 218h bytes), a helper for the in_vtable window of ShulkerBoxBlockItem.appendFormattedHovertext and for the slot of the Item virtuals that peek calls (slot_of on a body that the class does not override finds the same slot in this table). It holds the Item virtuals in header order. The entry is found from the contents of the table instead: each recipe finds a function that only this class has, takes its start, and vtable_holding gives the start of the one table that holds a pointer to it (the step finds the start from the code that stores the table as a vptr; it refuses when no table, or more than one table, holds the function). No byte of the lambda and no slot number is used. Recipe 0 is the body of the second user of the shulker contents text, the function that builds the contents text from the stack's user data and the level: `xorps xmm0,xmm0; movups [rdx],xmm0; mov qword ptr [rdx+..],..; mov [rbp+..],rdx; mov qword ptr [rdx+..],..; mov rdx,[r8+..]; lea rbx,[rbp+..]; mov rcx,rbx; call <contents builder>`, ended at the call, then function_start_of. Recipe 1 is the end of the body of ShulkerBoxBlockItem.appendFormattedHovertext, where the newline is written and appended: `mov byte ptr [r14],..; lea rcx,[r14+..]; mov rdx,rdi; mov r8,rbx; call; mov byte ptr [r14+rbx+..],..; lea rdx,[rbp+..]; mov rcx,rsi; call`, ended at the second call, then function_start_of. It is the same pattern as recipe 1 of that entry, copied and not referenced; change both together. No third function that only this class has can be used: the one small override of the class has no .pdata, and the lint cannot decode a match in a function without .pdata, so the hover body is the second anchor. Slot 34 would be a fallback, but the lint cannot decode that match today, so it is not used. The two recipes share no bytes and no function, and each function has exactly one data pointer, inside this table, on every build in the data. Every immediate, displacement and branch byte is a wildcard. Because recipe 1 finds the hover function, the in_vtable check of ShulkerBoxBlockItem.appendFormattedHovertext (window of that entry) proves only the slot window of that function in this table, not that the function is in the table: the proof that it is in the table is the two agreeing recipes of this entry, and recipe 0 does not use the hover function. The check is section .rdata, and the entry_is check (LevelChunk.mSubChunks = 456) holds on 26.60 and 26.62 and refuses the entry as not for this build on 26.51. What changed in 6aea437e-7eff13daabb430da (26.62): the registration lambda was relinked and the low byte of the RIP offset of its vtable lea moved, so the old recipes, which kept that byte once per build, matched nothing. That was a code-position change and not a code change. The candidate table was confirmed with two pieces of evidence from the old build: the body of the old hover function (appendFormattedHovertext) matches once and its one data pointer is in this table, and a pattern drafted from the old second contents user matches once and its one data pointer is in this table; the two sibling lambdas have tables where neither function is. The entry now uses vtable_holding and no kept byte."

  recipe {
    step "pattern" {
      bytes = "0F 57 C0 0F 11 02 48 C7 42 ?? ?? ?? ?? ?? 48 89 55 ?? 48 C7 42 ?? ?? ?? ?? ?? 49 8B 50 ?? 48 8D 5D ?? 48 89 D9 E8 ?? ?? ?? ??"
    }
    step "function_start_of" {}
    step "vtable_holding" {}
  }

  recipe {
    step "pattern" {
      bytes = "41 C6 06 ?? 49 8D 4E ?? 48 89 FA 49 89 D8 E8 ?? ?? ?? ?? 41 C6 44 1E ?? ?? 48 8D 55 ?? 48 89 F1 E8 ?? ?? ?? ??"
    }
    step "function_start_of" {}
    step "vtable_holding" {}
  }

  check "section" {
    name = ".rdata"
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ShulkerBoxBlockItem.appendFormattedHovertext" {
  kind     = "function"
  hooked   = true
  notes    = "void ShulkerBoxBlockItem::appendFormattedHovertext(ItemStackBase const& stack, Level& level, RedactableString& hovertext, bool advanced) const: rcx the Item, rdx the stack, r8 the Level, r9 the RedactableString, the bool at [rsp+28h] on entry (the fifth argument); void. Slot 54 of ShulkerBoxBlockItem.vtable. It calls the base Item::appendFormattedHovertext first with the same five arguments (the bool is copied to [rsp+20h]), then builds the contents text from the stack's user data (`mov rdx,[rdi+10h]; lea rdi,[rbp+..]; mov rcx,rdi; call <contents builder>`, the only user of the string container.shulkerboxContains) and appends a newline and that text to the RedactableString (call 3959360h). A hook that passes all five arguments through to the original gets the vanilla text. Recipe 0 is the prologue and the base call: `push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov rsi,r9; mov rdi,rdx; movzx eax,byte ptr [rbp+..]; mov [rsp+..],al; call <base>; mov rdx,[rdi+..]; lea rdi,[rbp+..]; mov rcx,rdi; call <contents builder>`. Recipe 1 is the end of the body, where the newline is written and appended: `mov byte ptr [r14],..; lea rcx,[r14+..]; mov rdx,rdi; mov r8,rbx; call; mov byte ptr [r14+rbx+..],..; lea rdx,[rbp+..]; mov rcx,rsi; call`, then function_start_of. The two anchors are in different parts of one body and share no pattern bytes; the function has no caller. The in_vtable window proves the slot window only: recipe 1 of ShulkerBoxBlockItem.vtable finds the table from this body, and that recipe is this entry's recipe 1 copied, so change both together. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and in_vtable on ShulkerBoxBlockItem.vtable, window 50..58. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h) holds on 26.60 and 26.62 and refuses as not for this build on 26.51. Value 35802E0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 4C 89 CE 48 89 D7 0F B6 45 ?? 88 44 24 ?? E8 ?? ?? ?? ?? 48 8B 57 ?? 48 8D 7D ?? 48 89 F9 E8 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "41 C6 06 ?? 49 8D 4E ?? 48 89 FA 49 89 D8 E8 ?? ?? ?? ?? 41 C6 44 1E ?? ?? 48 8D 55 ?? 48 89 F1 E8 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "ShulkerBoxBlockItem.vtable"
    min_slot = 50
    max_slot = 58
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
