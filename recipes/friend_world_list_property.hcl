entry "FriendWorldListProperty.vtable" {
  kind  = "global"
  notes = "The vtable of the cohtml property named `friendWorlds`: the property that sends the friend world list (a std::vector at +30h of the facet object of `vanilla.friendworldlist`) to the OreUI Play screen. Found through the type description function of that facet: it is the only function that loads the string `friendWorlds` in 1.26.62.01 and 1.26.63.01. It builds one property object on the stack and passes it to the cohtml binder: `lea rax,[rip+..]; mov [rbp+..],rax` stores the vtable at the object start, then `lea rax,[rip+..]; mov [rbp+..],rax` stores the member accessor (`lea rax,[rcx+30h]; ret`, shared by many properties, so it is not an entry). The vtable has two references in both builds: this function and the clone slot (slot 5) of the same vtable. Its type-specific slots are 1 (write the name, then the value), 2 (write the value), 5 (clone) and 12 (the element type information); the other slots are shared cohtml property functions. One recipe: string_xref of `friendWorlds`, then the two stores and the virtual call that passes the object to the binder (`mov rdx,[rbp+..]; mov rax,[rsi]; mov rax,[rax+disp]; lea r8,[rbp+..]; mov rcx,rsi; call [guard]`) in that function, then the RIP target of the first `lea`. The store order is the clue: the first store is the vtable, the second the accessor. Every immediate, displacement and branch byte of the pattern is a wildcard. No second independent recipe exists: the only other code that loads the table is its own clone slot, which is reached only through the table. The section check keeps the address inside .rdata; the in_vtable checks of FriendWorldListProperty.toBinderNamed and FriendWorldListProperty.toBinder catch a wrong table. Checked on 1.26.62.01 and 1.26.63.01: the recipe is unique in both and gives the table that holds the two body-found functions at slots 1 and 2. What changed in 6afa4e47-dadc375797a3d31a (70.26, 1.26.70.26): the compiler stores the accessor first and the vtable second, so the first-store rule read the accessor (a code address, refused by the section check). The recipe is now a pattern_any with one alternative per store order; the cursor (offset 11 in both) is the store of the vtable, and the vtable store is the one whose frame slot is the slot that `lea r8` passes (the object start holds the vtable), kept as four literal bytes with literal_ok; each alternative matches on one build only. Would have survived with a better recipe. Evidence on 70.26: the string `friendWorlds` is loaded by one function on both builds, the same function whose body has the call that passes the object; and the table that the recipe reads is the one that the in_vtable checks of the two body-found functions (FriendWorldListProperty.toBinder and toBinderNamed) must hold."

  recipe {
    step "string_xref" { text = "friendWorlds" }
    step "pattern_any" {
      patterns = [
        "?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 D0 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8B 55 ?? 48 8B 06 48 8B 40 ?? 4C 8D 45 D0 48 89 F1 FF 15",
        "48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 D0 48 8B 55 ?? 48 8B 06 48 8B 40 ?? 4C 8D 45 D0 48 89 F1 FF 15",
      ]
      literal_ok = [
        "0.21: frame slot of the first store (the vtable on 26.63): the store whose slot equals the slot that lea r8 passes is the vtable; the other store is the accessor",
        "0.47: the slot passed in lea r8 (the object, whose first qword is the vtable)",
        "1.21: frame slot of the second store (the vtable on 70.26): it equals the slot that lea r8 passes",
        "1.36: the slot passed in lea r8 (the object, whose first qword is the vtable)",
      ]
      offset     = 11
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
  notes  = "Slot 2 of FriendWorldListProperty.vtable: the cohtml property function that writes the friend world list (the value only, no name) to the binder. rcx = the property, rdx = the cohtml binder, r8 = the facet object; it returns r8 in rax. It gets the vector through the accessor at [this+18h], tries the binder's array fast path (binder vtable +158h, with four callbacks of this property), and when that returns false writes the array size (binder vtable +160h) and each element (200 bytes each; each element is described by the function that loads `friendOfFriendWorld`, a direct call). No direct call and no other reference reaches it: it is reached only through the vtable. Recipe 0 is the end of FriendWorldListProperty.toBinderNamed (the function before it in the image: `mov rax,rsi; add rsp,..; pop rbx; pop rdi; pop rsi; pop r14; ret`, six int3 pad bytes), then this function from its prologue (`push r14; push rsi; push rdi; push rbx; sub rsp,..`) to the element count `mov rax,[rbx+disp]; sub rax,[rbx]; sar rax,imm; mov rdx,imm64; imul rdx,rax`, as a pattern_any with one alternative per code shape, and `offset = 19` (the first byte of this function), then function_start_of. Alternative 0 (26.63) is the body straight after the argument moves: `mov rsi,r8; mov rdi,rdx; mov rax,[rcx+disp]; mov rcx,r8; call [guard]` (the accessor), the fast-path call with its four `lea` callbacks, `test al,al; jne`, the element count. Alternative 1 (1.26.70.26) has a stack cookie and a thread-safe static init guard with a debug assert block before the accessor call (`mov rbx,rcx` kept in rbx, `mov rax,[rbx+disp]; mov rcx,rsi; call [guard]`), then the same fast-path call and element count. Every immediate, displacement and branch byte is a wildcard except the eight bytes of the imm64 0x8F5C28F5C28F5C29 (the divide by 200, the element size): 24 other functions in each build have the same body with another constant. Why the prefix is there: on 1.26.70.26 a sibling class has a function with the 26.63 body shape and the same constant (a byte-identical twin of the 26.63 function apart from displacements), so a body pattern of the 26.63 shape alone finds two functions. The sibling function is not next to a toBinderNamed tail (its predecessor ends with 13 pad bytes), so the prefix tells the two apart. The recipe therefore depends on the link order: this function directly after toBinderNamed. On a build where that order changes it fails, it does not give a wrong value. Each alternative matches once on its own build and zero times on the other. The slot 1 head (the name write) is not in the match. No second independent recipe exists: the function has no direct caller, and a fixed `vtable_slot 2` route is flagged by the lint. The checks are `function_start` and `in_vtable` of FriendWorldListProperty.vtable, slots 2 to 4: the window holds slot 2 and leaves out slot 1 (toBinderNamed) and slot 5 (the clone). `function_start` cannot fail after `function_start_of`, so the window and the kept bytes are the facts besides the anchor. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the function gained a stack cookie and a static init guard, and a sibling class got a function with the old body. Class: real code change in the function (a new shape) with a recipe that needed a second shape. Evidence for 70.26: (1) the 26.63 body pattern with its toBinderNamed prefix matches once on 26.63 and the 70.26 alternative, built from the same prefix and the new body, matches once on 70.26, directly after the toBinderNamed that resolves on both builds (so the link neighbour of the old function is the new function); (2) the vtable neighbours of the old slot: on 26.63 slot 1 is toBinderNamed, slot 2 this function, slots 3 and 4 one pointer, slot 5 the clone; on 70.26 the table has the same shape and slot 2 holds the function that the pattern found. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns   = [
        "48 89 F0 48 83 C4 ?? 5B 5F 5E 41 5E C3 CC CC CC CC CC CC 41 56 56 57 53 48 83 EC ?? 4C 89 C6 48 89 D7 48 8B 41 ?? 4C 89 C1 FF 15 ?? ?? ?? ?? 48 89 C3 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 4C 8D 05 ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F9 48 89 DA FF 15 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 48 8B 43 ?? 48 2B 03 48 C1 F8 ?? 48 BA 29 5C 8F C2 F5 28 5C 8F 48 0F AF D0",
        "48 89 F0 48 83 C4 ?? 5B 5F 5E 41 5E C3 CC CC CC CC CC CC 41 56 56 57 53 48 83 EC ?? 4C 89 C6 48 89 D7 48 89 CB 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 ?? ?? ?? ?? 48 8B 0C CA 3B 81 ?? ?? ?? ?? 0F 8F ?? ?? ?? ?? 48 83 3D ?? ?? ?? ?? ?? 75 ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 8D 0D ?? ?? ?? ?? 48 8D 54 24 ?? 4C 8D 4C 24 ?? 41 B8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 05 ?? ?? ?? ?? C7 00 ?? ?? ?? ?? 48 8B 43 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 89 C3 48 8B 07 48 8B 80 ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 48 8D 0D ?? ?? ?? ?? 48 89 4C 24 ?? 4C 8D 05 ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F9 48 89 DA FF 15 ?? ?? ?? ?? 84 C0 0F 85 ?? ?? ?? ?? 48 8B 43 ?? 48 2B 03 48 C1 F8 ?? 48 BA 29 5C 8F C2 F5 28 5C 8F 48 0F AF D0",
      ]
      offset     = 19
      literal_ok = [
        "0.131: imm64 8F5C28F5C28F5C29, byte 0: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.132: imm64 8F5C28F5C28F5C29, byte 1: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.133: imm64 8F5C28F5C28F5C29, byte 2: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.134: imm64 8F5C28F5C28F5C29, byte 3: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.135: imm64 8F5C28F5C28F5C29, byte 4: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.136: imm64 8F5C28F5C28F5C29, byte 5: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.137: imm64 8F5C28F5C28F5C29, byte 6: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "0.138: imm64 8F5C28F5C28F5C29, byte 7: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.314: imm64 8F5C28F5C28F5C29, byte 0: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.315: imm64 8F5C28F5C28F5C29, byte 1: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.316: imm64 8F5C28F5C28F5C29, byte 2: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.317: imm64 8F5C28F5C28F5C29, byte 3: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.318: imm64 8F5C28F5C28F5C29, byte 4: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.319: imm64 8F5C28F5C28F5C29, byte 5: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.320: imm64 8F5C28F5C28F5C29, byte 6: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
        "1.321: imm64 8F5C28F5C28F5C29, byte 7: the divide by the element size 200; 24 other functions have this body with another constant (26.63, 1.26.70.26)",
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
