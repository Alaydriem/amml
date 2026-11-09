entry "FillingContainer.swapSlots" {
  kind     = "vslot"
  notes    = "Slot of void FillingContainer::swapSlots(int from, int to) in the container vtable: 44 on 26.51 (Container has 43 virtuals, 0..42, and swapSlots is the second new virtual of FillingContainer, after removeResource). In the Inventory vtable (the one that holds Inventory::setItemWithForceBalance at slot 13), slot 44 takes two ints, indexes mItems by both (`movsxd rax,r8d; imul rdi,rax,98h` and the same for edx; 98h is the ItemStack size) and swaps the two stacks through a temporary. Both recipes are calls in one PlayerInventory function on the player's Inventory (`mov rcx,[r14+..]`, PlayerInventory::mInventory) with the two slots in edx and r8d: `mov rcx,[r14+..]; mov rax,[rcx]; mov rax,[rax+disp]; (mov edi,[rbp+..];) mov edx,edi; mov r8d,r12d; call [guard]`. Recipe 0 is the call that first loads the slot from the frame; recipe 1 is the call that has it in edi already. They are two call sites of one function (no other caller passes two plain ints). Both read the displacement as a slot. Every displacement and immediate byte is a wildcard. Range window 36..52."

  recipe {
    step "pattern" {
      bytes  = "49 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 8B BD ?? ?? ?? ?? 89 FA 45 89 E0 FF 15"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 89 FA 45 89 E0 FF 15"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 36
    max = 52
  }
}

entry "FillingContainer.setItem" {
  kind     = "vslot"
  notes    = "Slot of void Container::setItem(int slot, ItemStack const&) in the container vtable: 12 on 26.51 (Container header order: getItem 7, setItem 12, setItemWithForceBalance 13). In the Inventory vtable slot 12 is the leaf `mov rax,[rcx]; mov rax,[rax+68h]; mov r10,[guard]; xor r9d,r9d; jmp r10` (setItemWithForceBalance with false), and slot 13 is the function that holds the lambda named `setItemWithForceBalance@Inventory`. Both recipes are the inline Player::setSelectedItem on the player's Inventory: `mov rax,[player+5B8h]` (Player::mInventory), `mov rcx,[rax+0B8h]` (PlayerInventory::mInventory), the selected slot in edx, the stack in r8, `mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`. Recipe 0 first tests the selected container id (`cmp byte ptr [rax+..],..; jne`) and passes the stack from rsi. Recipe 1 is a second function that passes the slot from ebx and the stack from r14. Both read the displacement as a slot. The two recipes are in two functions. Every displacement and immediate byte is a wildcard. Range window 4..20."

  recipe {
    step "pattern" {
      bytes  = "48 8B 87 ?? ?? ?? ?? 80 B8 ?? ?? ?? ?? ?? 75 ?? 48 8B 88 ?? ?? ?? ?? 8B 50 ?? 48 8B 01 48 8B 40 ?? 49 89 F0 FF 15"
      offset = 29
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 87 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 89 DA 4D 89 F0 FF 15"
      offset = 17
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 4
    max = 20
  }
}

entry "FillingContainer.mItems" {
  kind     = "field"
  notes    = "Offset of std::vector<ItemStack> FillingContainer::mItems (its first pointer; the last pointer follows 8 bytes later): 198h on 6aea437e-7eff13daabb430da (26.62). Found from the FillingContainer vtable (the tables that hold FillingContainer.swapSlots at slot 45 on this build): slot 7 (getItem, a leaf) tests the slot against the vector size, [this+198h] and [this+1A0h] divided by the ItemStack size 98h (`shr 3; imul 286BCA1Bh`, the inverse of 19 mod 2^32), and returns [this+198h] + slot * 98h; slot 45 (swapSlots, a leaf) indexes [this+198h] by both int slots with the stride 98h. Neither leaf has .pdata, so neither is used (the lint cannot decode a match in them, and five other container classes have the getItem body with another member offset). Recipe 0 is in void FillingContainer::removeResource(ItemStack const&, bool, bool, int) at slot 44 (a .pdata function): after the call that finds the next matching slot, `test eax,eax; js; mov eax,edi; mov rbx,[r15+disp]; imul r12,rax,98h; lea rdx,[rbx+r12]; mov rcx,r13; call`, ended at that call. Recipe 1 is the bounds test at the start of the slot-48 function (a .pdata function that clears one slot through setItem with the empty stack): `test edx,edx; js; mov rax,[rcx+disp]; mov r8,[rcx+disp]; sub r8,rax; shr r8,3; imul r8d,r8d,imm; cmp edx,r8d; jle`. Both read the displacement of the first load. The two recipes are in two functions, share no bytes and have different shapes (an index without a bounds test, a bounds test). Every displacement, immediate and branch byte is a wildcard, the stride 98h too; the register and opcode bytes make each pattern unique. Range window 100h..300h. The inventory chain reaches this vector through Player.playerInventory and PlayerInventory::mInventory (the Inventory is a FillingContainer). Value 198h on 6aa482fd-2658118eba0dbac5 (26.51), 6ad79ee4-17d51891a1119914 (26.60), 6aea437e-7eff13daabb430da (26.62) and 6aed1906-b127418e74d2f2f9 (26.63). On 26.51 recipe 0 does not match (a degraded row; removeResource has another shape there) and the entry resolves on recipe 1 and the range check."

  recipe {
    step "pattern" {
      bytes  = "85 C0 0F 88 ?? ?? ?? ?? 89 F8 49 8B 9F ?? ?? ?? ?? 4C 69 E0 ?? ?? ?? ?? 4A 8D 14 23 4C 89 E9 E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "85 D2 78 ?? 48 8B 81 ?? ?? ?? ?? 4C 8B 81 ?? ?? ?? ?? 49 29 C0 49 C1 E8 ?? 45 69 C0 ?? ?? ?? ?? 44 39 C2 7E ??"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 768
  }
}
