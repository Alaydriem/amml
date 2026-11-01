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
