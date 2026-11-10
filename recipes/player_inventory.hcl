entry "PlayerInventory.mInventory" {
  kind     = "field"
  notes    = "Offset of std::unique_ptr<Inventory> PlayerInventory::mInventory (the Inventory is a FillingContainer): 0B8h on 26.51, 26.60, 26.62 and 26.63. Found from the inline Player::setSelectedItem, the two call sites that FillingContainer.setItem uses: load the PlayerInventory (`mov rax,[player+disp]`, Player.playerInventory), then its Inventory (`mov rcx,[rax+disp]`), then call setItem through the Inventory vtable. The same bytes are the Player.playerInventory anchors, which keep this displacement as a literal; here every displacement, immediate and branch byte is a wildcard. Recipe 0 is the site that tests the selected container id first (`cmp byte ptr [rax+..],..; jne`), passes the slot from [rax+mSelected] and the stack from rsi; it reads the displacement of the `mov rcx,[rax+disp]` at offset 16. Recipe 1 is a second function that passes the slot from ebx and the stack from r14; it reads the same load at offset 7. The two recipes are in two functions. Range window 80h..140h."

  recipe {
    step "pattern" {
      bytes  = "48 8B 87 ?? ?? ?? ?? 80 B8 ?? ?? ?? ?? ?? 75 ?? 48 8B 88 ?? ?? ?? ?? 8B 50 ?? 48 8B 01 48 8B 40 ?? 49 89 F0 FF 15"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 87 ?? ?? ?? ?? 48 8B 88 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 89 DA 4D 89 F0 FF 15"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 320
  }
}

entry "PlayerInventory.mSelected" {
  kind     = "field"
  notes    = "Offset of int PlayerInventory::mSelected, the selected hotbar slot: 10h on 26.51, 26.60, 26.62 and 26.63 (after the two 8-byte listener bases). Found from the inline Player::setSelectedItem that tests the selected container id, loads the Inventory and passes the selected slot to FillingContainer::setItem in edx: `mov edx,[rax+disp8]` right after `mov rcx,[rax+PlayerInventory::mInventory]`. One recipe: the same pattern as recipe 0 of PlayerInventory.mInventory and FillingContainer.setItem, with every displacement, immediate and branch byte a wildcard, read at offset 23. The Player.playerInventory anchor keeps this byte as a literal, so that entry fails if the member moves. Range window 8h..40h."

  recipe {
    step "pattern" {
      bytes  = "48 8B 87 ?? ?? ?? ?? 80 B8 ?? ?? ?? ?? ?? 75 ?? 48 8B 88 ?? ?? ?? ?? 8B 50 ?? 48 8B 01 48 8B 40 ?? 49 89 F0 FF 15"
      offset = 23
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}
