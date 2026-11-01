entry "ShulkerBoxBlockItem.vtable" {
  kind     = "global"
  notes    = "The vtable of ShulkerBoxBlockItem (no RTTI; the object is 218h bytes, and the deleting destructor in slot 0 frees 218h), a helper for the in_vtable window of ShulkerBoxBlockItem.appendFormattedHovertext and for the slot of the Item virtuals that peek calls (slot_of on a body that the class does not override finds the same slot in this table). It holds the Item virtuals in header order; appendFormattedHovertext is slot 54 and the second user of the shulker contents text is slot 97. Found from the item registration lambda that allocates the item (`mov edx,218h; call [alloc]`), calls the BlockItem base constructor and then stores this table at [r12]. Two sibling lambdas (other item classes of the same size) have byte-identical code and differ only in their RIP offsets, so the match needs one byte that tells the lambdas apart: the low byte of the RIP offset of the vtable lea (0A2h here, 0F2h in one sibling, another value in the third). Recipe 0 is the allocation and the store: `mov edx,..; call [alloc]; nop; mov r12,rax; test rax,rax; je; lea r14,[rbx+..]; movsx r8d,r15w; lea r9,[rip+..]; mov rcx,r12; mov rdx,r14; mov [rbp+..],r12; call <base ctor>; nop; lea rax,[rip+..]`, read as the RIP target of the last lea. Recipe 1 starts at that lea and goes on to the next allocation: `lea rax,[rip+..]; mov r12,[rbp+..]; mov [r12],rax; mov rcx,[rip+..]; mov rax,[rcx]; mov rax,[rax+8]; mov edx,..; call [alloc]`. The two recipes are in the same function and both depend on the kept byte; the independent evidence is the in_vtable check of ShulkerBoxBlockItem.appendFormattedHovertext, which refuses this table when the hover function is not in it. Every other displacement, immediate and branch byte is a wildcard. The check is section .rdata. These RIP offsets move with every relink, so the entry is expected to need a new recipe on a later build. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value F1983E0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes      = "BA ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 49 89 C4 48 85 C0 0F 84 ?? ?? ?? ?? 4C 8D 73 ?? 45 0F BF C7 4C 8D 0D ?? ?? ?? ?? 4C 89 E1 4C 89 F2 4C 89 65 ?? E8 ?? ?? ?? ?? 90 48 8D 05 A2 ?? ?? ??"
      literal_ok = [
        "58: low byte of the RIP offset of the vtable lea: two sibling lambdas of the same size have identical code and other offsets",
      ]
      offset     = 55
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 8D 05 A2 ?? ?? ?? 4C 8B 65 ?? 49 89 04 24 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? BA ?? ?? ?? ?? FF 15"
      literal_ok = [
        "3: low byte of the RIP offset of the vtable lea: four lambdas of the same shape store other tables",
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

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ShulkerBoxBlockItem.appendFormattedHovertext" {
  kind     = "function"
  hooked   = true
  notes    = "void ShulkerBoxBlockItem::appendFormattedHovertext(ItemStackBase const& stack, Level& level, RedactableString& hovertext, bool advanced) const: rcx the Item, rdx the stack, r8 the Level, r9 the RedactableString, the bool at [rsp+28h] on entry (the fifth argument); void. Slot 54 of ShulkerBoxBlockItem.vtable. It calls the base Item::appendFormattedHovertext first with the same five arguments (the bool is copied to [rsp+20h]), then builds the contents text from the stack's user data (`mov rdx,[rdi+10h]; lea rdi,[rbp+..]; mov rcx,rdi; call <contents builder>`, the only user of the string container.shulkerboxContains) and appends a newline and that text to the RedactableString (call 3959360h). A hook that passes all five arguments through to the original gets the vanilla text. Recipe 0 is the prologue and the base call: `push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; mov qword ptr [rbp+..],-2; mov rsi,r9; mov rdi,rdx; movzx eax,byte ptr [rbp+..]; mov [rsp+..],al; call <base>; mov rdx,[rdi+..]; lea rdi,[rbp+..]; mov rcx,rdi; call <contents builder>`. Recipe 1 is the end of the body, where the newline is written and appended: `mov byte ptr [r14],..; lea rcx,[r14+..]; mov rdx,rdi; mov r8,rbx; call; mov byte ptr [r14+rbx+..],..; lea rdx,[rbp+..]; mov rcx,rsi; call`, then function_start_of. The two anchors are in different parts of one body and share no pattern bytes; the function has no caller, so the in_vtable window is the independent fact. Every displacement, immediate and branch byte is a wildcard. The checks are function_start and in_vtable on ShulkerBoxBlockItem.vtable, window 50..58. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 35802E0h on 6ad79ee4-17d51891a1119914 (26.60)."

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
