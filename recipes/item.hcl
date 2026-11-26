entry "Item.appendFormattedHovertext" {
  kind     = "function"
  notes    = "void Item::appendFormattedHovertext(ItemStackBase const& stack, Level& level, RedactableString& hovertext, bool advanced) const, the base implementation: rcx the Item, rdx the stack, r8 the Level, r9 the RedactableString, the bool at [rsp+28h] on entry. Slot 54 of the Item vtable; 74 item vtables hold it at that slot and 23 functions call it directly (the overrides call it first with their own five arguments). A caller copies the bool to [rsp+20h] before the call (`movzx eax,byte ptr [rbp+..]; mov [rsp+20h],al; call`). The ShulkerBoxBlockItem override is one of those callers. Recipe 0 is that call inside ShulkerBoxBlockItem.appendFormattedHovertext: the entry of the override, then `mov [rsp+..],al; call`, read as the call target. Recipe 1 is the prologue of the base function itself (`push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; mov rsi,r9; mov r14,r8; mov rbx,rdx; mov r15,rcx`). The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. No build gate: builds before 26.63 are not supported. Value 1A4EB60h on 6ad79ee4-17d51891a1119914 (26.60), 1A4E780h on 6aed1906-b127418e74d2f2f9 (26.63). What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): nothing in the code; only the old build gate refused the entry. Evidence on 26.70.26: (1) recipe 1, a prologue pattern drafted from the old base function, matches once; (2) recipe 0, the call in the ShulkerBoxBlockItem override (a pattern from the old override), gives the same function, which has 24 direct callers where the old function had 23; its slot in the ShulkerBoxBlockItem table is 53 where it was 54 on 26.63, one slot earlier like the other Item virtuals in front of it. Class: gate only."

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
}

entry "Item.getAnimationFrameFor" {
  kind     = "vslot"
  notes    = "Slot of int Item::getAnimationFrameFor(Mob* holder, bool, ItemStack const* stack, bool) const in the Item vtable: 119 on 26.60 (the header order of the Item virtuals breaks between 54 and 119, so the number is not the header's). rcx the Item, rdx the Mob*, r8b the bool, r9 the stack pointer, the second bool at [rsp+20h]; the int frame is in eax. ShulkerBoxBlockItem.vtable holds the default in this slot (`xor eax,eax; ret`, shared by many tiny functions), so the body cannot be matched; both recipes avoid it. Recipe 0 is the call site in the helper that draws one inventory item (`mov rcx,[rbx]; mov r10,[rcx+..]; mov byte ptr [rsp+20h],1; mov rcx,rbx; mov rdx,rax; xor r8d,r8d; mov r9,rdi; mov rax,r10; call [guard]`, after the call to getLocalPlayer for the holder), read as the displacement of the load and turned into a slot. Recipe 1 is the neighbour: the Item function in the next slot (120; it calls the virtual at +3C8h with the byte in edx: `sub rsp,..; mov r8d,edx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rax,[rcx]; mov rax,[rax+..]; lea rdx,[rsp+..]`), then function_start_of, slot_of ShulkerBoxBlockItem.vtable and add -1. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. No build gate: builds before 26.63 are not supported. Value 119 on 6ad79ee4-17d51891a1119914 (26.60) and 119 (77h) on 6aed1906-b127418e74d2f2f9 (26.63). What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): the slot is 118 (76h), one earlier: an Item virtual in front of it was removed. Evidence on 26.70.26: (1) recipe 0, the call site in the item draw helper, gives the displacement 3B0h, slot 118; (2) recipe 1, the neighbour function in the next slot (found from its old body) turned into a slot of the ShulkerBoxBlockItem table minus one, gives 118; the two recipes are in two functions. Neighbours agree: in the ShulkerBoxBlockItem table the functions that stood at slots 40, 44 and 54 on 26.63 stand at 39, 43 and 53 on 26.70.26, with the same runs of shared stubs between them. Class: real code change (one Item virtual removed), the recipes survive."

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
}

entry "Item.isGlint" {
  kind     = "vslot"
  notes    = "Slot of bool Item::isGlint(ItemStackBase const& stack) const in the Item vtable: 40 on 26.60. rcx the Item, rdx the stack; the bool in al. The base implementation (the one ShulkerBoxBlockItem keeps) reads the stack's user data, looks up the tag `ench` with CompoundTag::get, tests the tag id for 9 (a list) and ors the result with bit 0 of the byte at Item+152h (the foil flag). Recipe 0 is the head of that body (`push rsi; sub rsp,..; mov rsi,rcx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rcx,[rdx+..]; test rcx,rcx; je; lea rax,[rip+..]; mov [rsp+..],rax; mov qword ptr [rsp+..],..; lea rdx,[rsp+..]; call; test rax,rax; je; mov rcx`), then function_start_of and slot_of ShulkerBoxBlockItem.vtable. Recipe 1 is the end of the same body (`call [guard]; cmp al,9; sete al; jmp; xor eax,eax; movzx ecx,byte ptr [rsi+..]; mov rdx,[rsp+..]; xor rdx,rsp; mov r8,[rip+..]; cmp r8,rdx; jne; or al,cl`), then function_start_of and slot_of. The two anchors are in different parts of one body and share no pattern bytes; every call site of this virtual sits in a function with a dozen other virtual calls at the same displacement, so no call-site recipe is unique. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slot. No build gate: builds before 26.63 are not supported. Value 40 on 6ad79ee4-17d51891a1119914 (26.60) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): the slot is 39 (27h), one earlier: an Item virtual in front of it was removed. Evidence on 26.70.26: (1) recipe 0, the head of the old base body, and (2) recipe 1, the end of the same body (two parts of one body that share no bytes), both find the same function and give slot 39 in the ShulkerBoxBlockItem table; (3) neighbours: in that table the functions that stood at slots 40, 44 and 54 on 26.63 stand at 39, 43 and 53, with the same runs of shared stubs between them. Class: real code change (one Item virtual removed), the recipes survive."

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
}

entry "Item.getMaxDamage" {
  kind     = "vslot"
  notes    = "Slot of short Item::getMaxDamage() const in the Item vtable: 36 on 26.60 and 26.63, 35 on 70.26. rcx the Item; the short in ax. The base body is `movzx eax,word ptr [rcx+disp]; ret` (a leaf without .pdata that about 100 item vtables share; the member is 150h on 26.63 and 160h on 70.26), so the body cannot be matched; both recipes are virtual call sites, read as the displacement of the virtual load and turned into a slot. Recipe 0 is a small Item member that returns whether the item has a maximum damage: `mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test ax,ax; setg al; add rsp,..; ret`. Recipe 1 is the Item member that reads the `Damage` tag only when the item has a maximum damage: `mov rax,[rcx]; mov rax,[rax+disp]; call [guard]; test ax,ax; jle; mov rcx,[rsi+10h]; test rcx,rcx; je`; a pattern_any with one alternative per shape: alternative 0 has a short `jle` (26.63), alternative 1 a near `jle` (70.26); each matches once on its own build and 0 times on the other. The two recipes are in two functions and share only the shape of the virtual call. Every displacement, immediate and branch byte is a wildcard. The check is a range window around the known slots. No build gate: builds before 26.63 are not supported. The old recipe 1 (an ItemStackBase member that compared a damage value with the maximum through the slot) was removed: on 70.26 that member no longer calls the slot. What changed in 6afa4e47-dadc375797a3d31a: the slot moved from 36 to 35 (Item lost one virtual before it; Item.getAnimationFrameFor moved by one too), the `Damage` tag function got a near branch, and some callers read the member directly (`cmp word ptr [rcx+160h],0`). Class: would have survived with a better recipe. Evidence on 70.26, from the 26.63 slot: (1) the 26.63 function of recipe 1 (the `Damage` tag read after the slot call, tag pointer at [rsi+10h], the text length 6) is found once on 70.26 with the same order and reads the virtual at 118h (slot 35); (2) the 26.63 base body (`movzx eax,word ptr [rcx+150h]; ret`, held by 101 vtables) has a counterpart on 70.26 (`movzx eax,word ptr [rcx+160h]; ret`, held by 101 vtables), and in ShulkerBoxBlockItem.vtable it is at slot 36 on 26.63 and at slot 35 on 70.26; the recipe 0 wrapper is the same function on both builds and reads 120h and 118h."

  recipe {
    step "pattern" {
      bytes  = "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 66 85 C0 0F 9F C0 48 83 C4 ?? C3"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 66 85 C0 7E ?? 48 8B 4E ?? 48 85 C9 74 ??",
        "48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 66 85 C0 0F 8E ?? ?? ?? ?? 48 8B 4E ?? 48 85 C9 74 ??",
      ]
      offset   = 3
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
}


entry "Item.mFullName" {
  kind     = "field"
  notes    = "Offset in Item of mFullName, the HashedString that holds the item's full name (`minecraft:white_shulker_box`): 120h on 26.60 (hash at +120h, the std::string text at +128h, so the text is 8 bytes on). ItemRenderer::renderGuiItemNew reads it twice, once for each of the two draws it can make (the foil draw and the plain draw): it loads the hash at +120h, compares it with a cached hash, and loads the address of the text. Both recipes start from ItemRenderer.renderGuiItemNew (`ref`) and scan inside it, then read the member with member_displacement (the displacement of `lea r9,[r12+disp]` on 26.60 to 26.63, the immediate of `add rcx,imm` on the Preview). Recipe 0 alternative 0 is the first inline copy of the hash compare (`mov rax,[r12+..]; cmp rax,[rip+..]; jne; lea r9,[r12+..]`), with the low byte of its jne distance kept (97h); recipe 1 alternative 0 is the second copy from its `cmp`, with its jne byte kept (0AAh). On the Preview the compare is no longer inline: the function calls an out-of-line HashedString operator== three times, each time after `mov rax,[rbx+..]; test; je; mov rcx,[rax]; test; je; add rcx,imm; lea rdx,[rip+static HashedString]`. Recipe 0 alternative 1 is the one copy with near je branches; recipe 1 alternative 1 is the short copy whose first je jumps 27h (the other short copy jumps 23h; that byte is kept). Both are unique inside the function and match many other functions outside it, so the ref is needed. Both recipes are in the same function; no other function reads this member in a form that is unique. Every other displacement, immediate and branch byte is a wildcard. The check is a window around the known value. No build gate: builds before 26.63 are not supported. Value 120h on 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the value is 128h (8 bytes later) and the HashedString type itself changed: its out-of-line operator== compares the last-match pointer at +30h (the 26.63 inline compare used +28h) and branches on a byte at +28h (0 or 1) before it reads the text, so HashedString grew from 30h to 38h and the text is no longer a plain std::string at +8. Evidence for 128h on the Preview: (1) in ItemRenderer::renderGuiItemNew (found from the 26.63 function by both of its recipes) the item pointer from `[stack+8]` gets `add rcx,128h` before each HashedString compare, where 26.63 read `[r12+120h]`; (2) a separate group of item-stack helpers that compare the item name with the same static HashedString as the first renderGuiItemNew block on 26.63 (`[stack+8]`, then `[item+120h]` on 26.63) use `add ..,128h` before the same operator== on the Preview. Class: real code change (layout). 26.70.26 has the same HashedString change. ItemRenderer::renderGuiItemNew holds four copies of `mov rax,[rbx+8]; test; je; mov rcx,[rax]; test; je; add rcx,128h; lea rdx,[static HashedString]; call; test al,al; jne`: the first has short jumps of 4Fh and 47h and is followed by the second copy, the second has short jumps of 23h and 1Bh and a `test r13b,r13b` after its jne, the third has a `mov r14d,[rbp+..]` between its first test and its je, the fourth has near jumps. Recipe 0 alternative 1 is the fourth copy; recipe 1 alternative 1 is the second copy, told from the first by the `test r13b,r13b` after its jne and from the third by the plain `test; je` pair, so it keeps no jump byte. That alternative runs past its call (a direct call, with the test after it on this build). Evidence on 26.70.26 for 128h: (1) the old function (found by both of its recipes) reads the HashedString hash at [item+120h] before each compare with the static HashedString, and the two recipes here read the same compare in two different copies in the 26.70.26 function, both at +128h; (2) other functions that compare the item name with the same static HashedString (three further users of that static take `add rcx,128h` after loading the item from [stack+8]) agree. Preview 70.23 had the same value."

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
      patterns   = ["48 3B 05 ?? ?? ?? ?? 0F 85 AA ?? ?? ?? 4D 8D 8C 24 ?? ?? ?? ??", "48 85 C0 74 ?? 48 8B 08 48 85 C9 74 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 45 84 ED"]
      offset     = 13
      literal_ok = [
        "0.9: low byte of the jne distance: the first copy of this block in the same function jumps a different distance",
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
