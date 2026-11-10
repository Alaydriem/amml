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
  notes    = "Offset in Item of mFullName, the HashedString that holds the item's full name (`minecraft:white_shulker_box`): 120h on 26.60 (hash at +120h, the std::string text at +128h, so the text is 8 bytes on). ItemRenderer::renderGuiItemNew reads it twice, once for each of the two draws it can make (the foil draw and the plain draw): it loads the hash at +120h, compares it with a cached hash, and loads the address of the text. Both recipes start from ItemRenderer.renderGuiItemNew (`ref`) and scan inside it, then read the member with member_displacement (the displacement of `lea r9,[r12+disp]` on 26.60 to 26.63, the immediate of `add rcx,imm` on the Preview). Recipe 0 alternative 0 is the first inline copy of the hash compare (`mov rax,[r12+..]; cmp rax,[rip+..]; jne; lea r9,[r12+..]`), with the low byte of its jne distance kept (97h); recipe 1 alternative 0 is the second copy from its `cmp`, with its jne byte kept (0AAh). On the Preview the compare is no longer inline: the function calls an out-of-line HashedString operator== three times, each time after `mov rax,[rbx+..]; test; je; mov rcx,[rax]; test; je; add rcx,imm; lea rdx,[rip+static HashedString]`. Recipe 0 alternative 1 is the one copy with near je branches; recipe 1 alternative 1 is the short copy whose first je jumps 27h (the other short copy jumps 23h; that byte is kept). Both are unique inside the function and match many other functions outside it, so the ref is needed. Both recipes are in the same function; no other function reads this member in a form that is unique. Every other displacement, immediate and branch byte is a wildcard. The check is a window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 120h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the value is 128h (8 bytes later) and the HashedString type itself changed: its out-of-line operator== compares the last-match pointer at +30h (the 26.63 inline compare used +28h) and branches on a byte at +28h (0 or 1) before it reads the text, so HashedString grew from 30h to 38h and the text is no longer a plain std::string at +8. The gate keeps this entry off the Preview until a mirror for the new HashedString exists. Evidence for 128h on the Preview: (1) in ItemRenderer::renderGuiItemNew (found from the 26.63 function by both of its recipes) the item pointer from `[stack+8]` gets `add rcx,128h` before each HashedString compare, where 26.63 read `[r12+120h]`; (2) a separate group of item-stack helpers that compare the item name with the same static HashedString as the first renderGuiItemNew block on 26.63 (`[stack+8]`, then `[item+120h]` on 26.63) use `add ..,128h` before the same operator== on the Preview. Class: real code change (layout)."

  recipe {
    step "ref" { entry = "ItemRenderer.renderGuiItemNew" }
    step "pattern_any" {
      patterns   = ["49 8B 84 24 ?? ?? ?? ?? 48 3B 05 ?? ?? ?? ?? 0F 85 97 ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ??", "48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 08 48 85 C9 0F 84 ?? ?? ?? ?? 48 81 C1 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8"]
      offset     = 21
      literal_ok = [
        "0.17: low byte of the jne distance: the second copy of this block in the same function jumps a different distance",
      ]
    }
    step "member_displacement" {
      index    = 1
      local_at = 0
    }
  }

  recipe {
    step "ref" { entry = "ItemRenderer.renderGuiItemNew" }
    step "pattern_any" {
      patterns   = ["48 3B 05 ?? ?? ?? ?? 0F 85 AA ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ??", "48 85 C0 74 27 48 8B 08 48 85 C9 74 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8"]
      offset     = 13
      literal_ok = [
        "0.9: low byte of the jne distance: the first copy of this block in the same function jumps a different distance",
        "1.4: the je distance: the Preview function has a second short copy of this block that jumps 23h",
      ]
    }
    step "member_displacement" {
      index    = 1
      local_at = 0
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

entry "Item.useOn" {
  kind     = "function"
  hooked   = true
  notes    = "InteractionResult Item::useOn(ItemStack& stack, Actor& actor, BlockPos const& pos, uchar face, uchar, Vec3 const& clickPos, <8-byte value>) const, non-virtual (no table holds it). The InteractionResult comes back through a hidden pointer: rcx the Item, rdx the result pointer, r8 the stack, r9 the actor, then on entry [rsp+28h] the BlockPos const& (the header has int x, y, z), [rsp+30h] the face byte, [rsp+38h] a byte that GameMode::useItemOn passes on from its seventh argument, [rsp+40h] the Vec3 const& click position (relative to the block), [rsp+48h] an 8-byte value (low dword 1 from GameMode::useItemOn, bit 32 its isFirstEvent). The result is one byte written through the pointer (bit 0 success, bit 1 swing; useItemOn tests bit 0); rax returns the pointer. When the item declines (the tests at the start of the body), the body writes 0 and returns; otherwise it calls the Item virtual _useOn with the same arguments. Call order on this build, in GameMode::useItemOn (slot 13 of GameMode.vtable, 376CA20h): the block's use (3168070h, with the player, the position, the face and the click position) runs first; when it returns true, useItemOn sets the result to 3 and returns without the item; only when it returns false (or when the player sneaks with an item, which skips the block's use) does it call ItemStack::useOn (1A7CDA0h), which calls this function. So chests, doors and levers do not reach it from the player's click. Other callers: ItemStack::useOn is also called from five other functions (mob and item behaviours, such as a goal at 37D6130h), and this function from one more (8F44EA0h); a hook must test that the actor is the local player. Recipe 0 is the call in ItemStack::useOn: `mov [rsp+..],r14; mov [rsp+..],r13; mov [rsp+..],r12b; mov [rsp+..],bpl; mov [rsp+..],rax; mov rdx,rsi; mov r8,rbx; mov r9,rdi; call`, read as the call target. Recipe 1 is the head of the body: `mov rdi,r9; mov rbx,r8; mov rsi,rdx; mov r14,rcx; test byte ptr [rcx+..],..; je; mov rcx,rdi; call`, ended at the call, then function_start_of. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" {
      bytes  = "4C 89 74 24 ?? 4C 89 6C 24 ?? 44 88 64 24 ?? 40 88 6C 24 ?? 48 89 44 24 ?? 48 89 F2 49 89 D8 49 89 F9 E8"
      offset = 34
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "4C 89 CF 4C 89 C3 48 89 D6 49 89 CE F6 81 ?? ?? ?? ?? ?? 74 ?? 48 89 F9 E8" }
    step "function_start_of" {}
  }

  check "function_start" {}
}
