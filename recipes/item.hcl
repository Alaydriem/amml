entry "Item.appendFormattedHovertext" {
  kind     = "function"
  notes    = "void Item::appendFormattedHovertext(ItemStackBase const& stack, Level& level, RedactableString& hovertext, bool advanced) const, the base implementation: rcx the Item, rdx the stack, r8 the Level, r9 the RedactableString, the bool at [rsp+28h] on entry. Slot 54 of the Item vtable; 74 item vtables hold it at that slot and 23 functions call it directly (the overrides call it first with their own five arguments). A caller copies the bool to [rsp+20h] before the call (`movzx eax,byte ptr [rbp+..]; mov [rsp+20h],al; call`). The ShulkerBoxBlockItem override is one of those callers. Recipe 0 is that call inside ShulkerBoxBlockItem.appendFormattedHovertext: the entry of the override, then `mov [rsp+..],al; call`, read as the call target. Recipe 1 is the prologue of the base function itself (`push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; mov rsi,r9; mov r14,r8; mov rbx,rdx; mov r15,rcx`). The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 1A4EB60h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "ref" { entry = "ShulkerBoxBlockItem.appendFormattedHovertext" }
    step "pattern" {
      bytes  = "88 44 24 ?? E8 ?? ?? ?? ??"
      offset = 4
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 75 ?? 48 C7 45 ?? ?? ?? ?? ?? 4C 89 CE 4D 89 C6 48 89 D3 49 89 CF" }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Item.getAnimationFrameFor" {
  kind     = "vslot"
  notes    = "Slot of int Item::getAnimationFrameFor(Mob* holder, bool, ItemStack const* stack, bool) const in the Item vtable: 119 on 26.60 (the header order of the Item virtuals breaks between 54 and 119, so the number is not the header's). rcx the Item, rdx the Mob*, r8b the bool, r9 the stack pointer, the second bool at [rsp+20h]; the int frame is in eax. ShulkerBoxBlockItem.vtable holds the default in this slot (`xor eax,eax; ret`, shared by many tiny functions), so the body cannot be matched; both recipes avoid it. Recipe 0 is the call site in the helper that draws one inventory item (`mov rcx,[rbx]; mov r10,[rcx+..]; mov byte ptr [rsp+20h],1; mov rcx,rbx; mov rdx,rax; xor r8d,r8d; mov r9,rdi; mov rax,r10; call [guard]`, after the call to getLocalPlayer for the holder), read as the displacement of the load and turned into a slot. Recipe 1 is the neighbour: the Item function in the next slot (120; it calls the virtual at +3C8h with the byte in edx: `sub rsp,..; mov r8d,edx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rsp+..]`), then function_start_of, slot_of ShulkerBoxBlockItem.vtable and add -1. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 119 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 0B 4C 8B 91 ?? ?? ?? ?? C6 44 24 ?? ?? 48 89 D9 48 89 C2 45 31 C0 49 89 F9 4C 89 D0 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" { bytes = "48 83 EC ?? 41 89 D0 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ??" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ShulkerBoxBlockItem.vtable" }
    step "add" { value = -1 }
  }

  check "range" {
    min = 90
    max = 140
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Item.isGlint" {
  kind     = "vslot"
  notes    = "Slot of bool Item::isGlint(ItemStackBase const& stack) const in the Item vtable: 40 on 26.60. rcx the Item, rdx the stack; the bool in al. The base implementation (the one ShulkerBoxBlockItem keeps) reads the stack's user data, looks up the tag `ench` with CompoundTag::get, tests the tag id for 9 (a list) and ors the result with bit 0 of the byte at Item+152h (the foil flag). Recipe 0 is the head of that body (`push rsi; sub rsp,..; mov rsi,rcx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rcx,[rdx+..]; test rcx,rcx; je; lea rax,[rip+..]; mov [rsp+..],rax; mov qword ptr [rsp+..],..; lea rdx,[rsp+..]; call; test rax,rax; je; mov rcx`), then function_start_of and slot_of ShulkerBoxBlockItem.vtable. Recipe 1 is the end of the same body (`call [guard]; cmp al,9; sete al; jmp; xor eax,eax; movzx ecx,byte ptr [rsi+..]; mov rdx,[rsp+..]; xor rdx,rsp; mov r8,[rip+..]; cmp r8,rdx; jne; or al,cl`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes; every call site of this virtual sits in a function with a dozen other virtual calls at the same displacement, so no call-site recipe is unique. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 40 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "56 48 83 EC ?? 48 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 4A ?? 48 85 C9 74 ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 54 24 ?? E8 ?? ?? ?? ?? 48 85 C0 74 ?? 48 8B" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ShulkerBoxBlockItem.vtable" }
  }

  recipe {
    step "pattern" { bytes = "FF 15 ?? ?? ?? ?? 3C ?? 0F 94 C0 EB ?? 31 C0 0F B6 8E ?? ?? ?? ?? 48 8B 54 24 ?? 48 31 E2 4C 8B 05 ?? ?? ?? ?? 49 39 D0 75 ?? 08 C8" }
    step "function_start_of" {}
    step "slot_of" { vtable = "ShulkerBoxBlockItem.vtable" }
  }

  check "range" {
    min = 20
    max = 70
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Item.getMaxDamage" {
  kind     = "vslot"
  notes    = "Slot of short Item::getMaxDamage() const in the Item vtable: 36 on 26.60. rcx the Item; the short in ax. The base body is `movzx eax,word ptr [rcx+150h]` (a leaf without .pdata that the tables share), so the body cannot be matched; both recipes are call sites. Recipe 0 is Item::canBeDepleted-like code in the Item table: `mov rax,[rcx]; mov rax,[rax+..]; call [guard]; test ax,ax; jle; mov rcx,[rsi+..]; test rcx,rcx; je`, read as the displacement of the virtual load. Recipe 1 is an ItemStackBase member that compares a damage value with the maximum: `mov rax,[rcx]; mov rax,[rax+..]; call [guard]; mov ecx,eax; jmp; xor ecx,ecx; mov al,1; cmp bx,cx; jle`, read the same way. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 36 on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 66 85 C0 7E ?? 48 8B 4E ?? 48 85 C9 74 ??"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 89 C1 EB ?? 31 C9 B0 ?? 66 39 CB 0F 8E"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 20
    max = 70
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "Item.mFullName" {
  kind     = "field"
  notes    = "Offset in Item of mFullName, the HashedString that holds the item's full name (`minecraft:white_shulker_box`): 120h on 26.60 (hash at +120h, the std::string text at +128h, so the text is 8 bytes on). ItemRenderer::renderGuiItemNew reads it twice, once for each of the two draws it can make (the foil draw and the plain draw): it loads the hash at +120h, compares it with a cached hash, and loads the address of the text. Recipe 0 is the first copy of that block (`mov rax,[r12+..]; cmp rax,[rip+..]; jne; lea r9,[r12+..]`) and recipe 1 the second copy; the two blocks are the same code with other jump distances, so one byte of the jump distance is kept in each (the first jumps 97h, the second 0AAh). Both recipes are in the same function; no other function reads this member in a form that is unique. Every other displacement, immediate and branch byte is a wildcard. The check is a window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 120h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes      = "49 8B 84 24 ?? ?? ?? ?? 48 3B 05 ?? ?? ?? ?? 0F 85 97 ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ??"
      literal_ok = [
        "17: low byte of the jne distance: the second copy of this block in the same function jumps a different distance",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "49 8B 84 24 ?? ?? ?? ?? 48 3B 05 ?? ?? ?? ?? 0F 85 AA ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ??"
      literal_ok = [
        "17: low byte of the jne distance: the first copy of this block in the same function jumps a different distance",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 64
    max = 512
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
