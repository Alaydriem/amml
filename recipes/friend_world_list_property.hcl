entry "FriendWorldListProperty.vtable" {
  kind  = "global"
  notes = "The vtable of the cohtml property named `friendWorlds`: the property that sends the friend world list (a std::vector at +30h of the facet object of `vanilla.friendworldlist`) to the OreUI Play screen. Found through the type description function of that facet: it is the only function that loads the string `friendWorlds` in 1.26.62.01 and 1.26.63.01. It builds one property object on the stack and passes it to the cohtml binder: `lea rax,[rip+..]; mov [rbp+..],rax` stores the vtable at the object start, then `lea rax,[rip+..]; mov [rbp+..],rax` stores the member accessor (`lea rax,[rcx+30h]; ret`, shared by many properties, so it is not an entry). The vtable has two references in both builds: this function and the clone slot (slot 5) of the same vtable. Its type-specific slots are 1 (write the name, then the value), 2 (write the value), 5 (clone) and 12 (the element type information); the other slots are shared cohtml property functions. One recipe: string_xref of `friendWorlds`, then the two stores and the virtual call that passes the object to the binder (`mov rdx,[rbp+..]; mov rax,[rsi]; mov rax,[rax+disp]; lea r8,[rbp+..]; mov rcx,rsi; call [guard]`) in that function, then the RIP target of the first `lea`. The store order is the clue: the first store is the vtable, the second the accessor. Every immediate, displacement and branch byte of the pattern is a wildcard. No second independent recipe exists: the only other code that loads the table is its own clone slot, which is reached only through the table. The section check keeps the address inside .rdata; the in_vtable checks of FriendWorldListProperty.toBinderNamed and FriendWorldListProperty.toBinder catch a wrong table. Checked on 1.26.62.01 and 1.26.63.01: the recipe is unique in both and gives the table that holds the two body-found functions at slots 1 and 2."

  recipe {
    step "string_xref" { text = "friendWorlds" }
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 55 ?? 48 8B 06 48 8B 40 ?? 4C 8D 45 ?? 48 89 F1 FF 15"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "FriendWorldListProperty.toBinderNamed" {
  kind   = "function"
  hooked = true
  notes  = "Slot 1 of FriendWorldListProperty.vtable: the cohtml property function that writes the property name `friendWorlds` and then the friend world list to the binder. rcx = the property, rdx = the cohtml binder, r8 = the facet object; it returns r8 in rax. It first calls the binder's name write (binder vtable +A8h) with the name at [rcx+8], then does what slot 2 does: gets the vector through the accessor at [rcx+18h], tries the binder's array fast path (binder vtable +158h, with four callbacks of this property), and when that returns false writes the array size (binder vtable +160h) and each element (200 bytes each; each element is described by the function that loads `friendOfFriendWorld`, a direct call). No direct call and no other reference reaches it: it is reached only through the vtable. Recipe 0 is the body after the four pushes and the frame: `mov rsi,r8; mov rdi,rdx; mov rbx,rcx; mov rdx,[rcx+disp]` (the name), the name write, the accessor call, the fast-path call with its four `lea` callbacks, `test al,al; jne`, and the element count `mov rax,[rbx+disp]; sub rax,[rbx]; sar rax,imm; mov rdx,imm64; imul rdx,rax`, then `function_start_of`. Every immediate, displacement and branch byte is a wildcard except the eight bytes of the imm64 0x8F5C28F5C28F5C29 (with the sar, the divide by 200, the element size): 24 other functions in each build have the same body (the same binder calls) with another constant, and these bytes are the only thing that tells this one apart. The name write at the head tells it apart from slot 2, which has the same body without it. The recipe is unique in 1.26.62.01 and 1.26.63.01. No second independent recipe exists: the function has no direct caller, and its body after the head is the same as slot 2, so a second body anchor that shares no bytes with recipe 0 matches both; a fixed `vtable_slot 1` route is flagged by the lint. The checks are `function_start` and `in_vtable` of FriendWorldListProperty.vtable, slots 0 to 1: the window holds slot 1 and leaves out slot 2 (toBinder), which has the same body without the name write. `function_start` cannot fail after `function_start_of`, so the window and the kept bytes are the facts besides the anchor. Checked on 1.26.62.01 and 1.26.63.01: slot 1 on both, and the code shape is the same."

  recipe {
    step "pattern" {
      bytes      = "4C 89 C6 48 89 D7 48 89 CB 48 8B 51 ?? 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 43 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 89 C3 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 4C 8D 05 ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F9 48 89 DA FF 15 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 48 8B 43 ?? 48 2B 03 48 C1 F8 ?? 48 BA 29 5C 8F C2 F5 28 5C 8F 48 0F AF D0"
      literal_ok = [
        "129: imm64 8F5C28F5C28F5C29, byte 0: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "130: imm64 8F5C28F5C28F5C29, byte 1: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "131: imm64 8F5C28F5C28F5C29, byte 2: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "132: imm64 8F5C28F5C28F5C29, byte 3: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "133: imm64 8F5C28F5C28F5C29, byte 4: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "134: imm64 8F5C28F5C28F5C29, byte 5: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "135: imm64 8F5C28F5C28F5C29, byte 6: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "136: imm64 8F5C28F5C28F5C29, byte 7: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}
  check "in_vtable" {
    vtable   = "FriendWorldListProperty.vtable"
    min_slot = 0
    max_slot = 1
  }
}

entry "FriendWorldListProperty.toBinder" {
  kind   = "function"
  hooked = true
  notes  = "Slot 2 of FriendWorldListProperty.vtable: the cohtml property function that writes the friend world list (the value only, no name) to the binder. rcx = the property, rdx = the cohtml binder, r8 = the facet object; it returns r8 in rax. It gets the vector through the accessor at [rcx+18h], tries the binder's array fast path (binder vtable +158h, with four callbacks of this property), and when that returns false writes the array size (binder vtable +160h) and each element (200 bytes each; each element is described by the function that loads `friendOfFriendWorld`, a direct call). No direct call and no other reference reaches it: it is reached only through the vtable. Recipe 0 is the body after the four pushes and the frame: `mov rsi,r8; mov rdi,rdx; mov rax,[rcx+disp]; mov rcx,r8; call [guard]` (the accessor), the fast-path call with its four `lea` callbacks, `test al,al; jne`, and the element count `mov rax,[rbx+disp]; sub rax,[rbx]; sar rax,imm; mov rdx,imm64; imul rdx,rax`, then `function_start_of`. Every immediate, displacement and branch byte is a wildcard except the eight bytes of the imm64 0x8F5C28F5C28F5C29 (with the sar, the divide by 200, the element size): 24 other functions in each build have the same body (the same binder calls) with another constant, and these bytes are the only thing that tells this one apart. The head (the accessor call straight after the argument moves) tells it apart from slot 1, which writes the name first. The recipe is unique in 1.26.62.01 and 1.26.63.01. No second independent recipe exists: the function has no direct caller, and its body after the head is the same as slot 1, so a second body anchor that shares no bytes with recipe 0 matches both; a fixed `vtable_slot 2` route is flagged by the lint. The checks are `function_start` and `in_vtable` of FriendWorldListProperty.vtable, slots 2 to 4: the window holds slot 2 and leaves out slot 1 (toBinderNamed, the same body with the name write) and slot 5 (the clone). `function_start` cannot fail after `function_start_of`, so the window and the kept bytes are the facts besides the anchor. Checked on 1.26.62.01 and 1.26.63.01: slot 2 on both, and the code shape is the same."

  recipe {
    step "pattern" {
      bytes      = "4C 89 C6 48 89 D7 48 8B 41 ?? 4C 89 C1 FF 15 ?? ?? ?? ?? 48 89 C3 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 4C 8D 05 ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F9 48 89 DA FF 15 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 48 8B 43 ?? 48 2B 03 48 C1 F8 ?? 48 BA 29 5C 8F C2 F5 28 5C 8F 48 0F AF D0"
      literal_ok = [
        "103: imm64 8F5C28F5C28F5C29, byte 0: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "104: imm64 8F5C28F5C28F5C29, byte 1: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "105: imm64 8F5C28F5C28F5C29, byte 2: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "106: imm64 8F5C28F5C28F5C29, byte 3: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "107: imm64 8F5C28F5C28F5C29, byte 4: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "108: imm64 8F5C28F5C28F5C29, byte 5: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "109: imm64 8F5C28F5C28F5C29, byte 6: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
        "110: imm64 8F5C28F5C28F5C29, byte 7: the divide by the element size 200; 24 other functions have this body with another constant (1.26.62.01, 1.26.63.01)",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}
  check "in_vtable" {
    vtable   = "FriendWorldListProperty.vtable"
    min_slot = 2
    max_slot = 4
  }
}
