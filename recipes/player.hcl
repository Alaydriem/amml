entry "Player.getPlayerGameType" {
  kind     = "function"
  notes    = "GameType Player::getPlayerGameType() const: rcx the player, the type in eax (Survival 0, Creative 1, Adventure 2, Default 5, Spectator 6; -1 when the component is missing). The body looks up the game type component through the entity context at this+8 (type hash 88D3EDDFh), and when the type is Default it tail-calls the level's default game type (`mov rcx,[rcx+1D8h]; mov rax,[rcx]; mov rax,[rax+460h]; ... jmp rdx`, Actor::mLevel and Level slot 140). Many callers hold the same code inline; Player::isCreative and the other tests next to it inline it too. Recipe 0 is the end of the body: `mov eax,[rdx+r9*4]; cmp eax,..; jne; mov rcx,[rcx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,[guard]`, then function_start_of. Recipe 1 is the SetPlayerGameTypePacket handler (NetEventCallback slot 224): `cmp qword ptr [rbx+..],..; je; mov ebx,[rsi+..]; mov rcx,rdi; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The ModRM bytes of the compare, the load and the mov in recipe 1 are wildcards. What changed in 6ad79ee4-17d51891a1119914 (26.60): the SetPlayerGameTypePacket handler holds the player in rbx and the new type in edi (rdi and ebx on 26.51); recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "42 8B 04 8A 83 F8 ?? 75 ?? 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 ?? ?? ?? ?? ?? ?? 74 ?? 8B ?? ?? 48 89 ?? E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Player.playerInventory" {
  kind     = "field"
  notes    = "Offset of std::unique_ptr<PlayerInventory> Player::mInventory: 5B8h on 26.51, the anchor of the inventory chain (Player, then PlayerInventory, then its Inventory). LeviLamina header sum: 5B8h. Both recipes are the inline Player::setSelectedItem: load the PlayerInventory (`mov rax,[rdi+5B8h]`), then its Inventory (`mov rcx,[rax+0B8h]`) and the selected slot (`mov edx,[rax+10h]`), then call FillingContainer::setItem (slot 12). The low bytes of those two PlayerInventory displacements are kept, so the anchor fails when the PlayerInventory mirror moves (mSelected 10h, mInventory 0B8h; the class has two 8-byte listener bases). Recipe 0 also tests the selected container id (`cmp byte ptr [rax+0B0h],0; jne`), whose low byte is kept too. Recipe 1 is a second function that calls setItem with the slot from a register. Both read the displacement of the first load. Range window 580h..600h."

  recipe {
    step "pattern" {
      bytes      = "48 8B 87 ?? ?? ?? ?? 80 B8 B0 ?? ?? ?? ?? 75 ?? 48 8B 88 B8 ?? ?? ?? 8B 50 10 48 8B 01 48 8B 40 ?? 49 89 F0 FF 15"
      literal_ok = [
        "9: cmp byte ptr [rax+0B0h]: PlayerInventory::mSelectedContainerId, part of the mirror this entry anchors",
        "19: mov rcx,[rax+0B8h]: PlayerInventory::mInventory, part of the mirror this entry anchors",
        "25: mov edx,[rax+10h]: PlayerInventory::mSelected, part of the mirror this entry anchors",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "48 8B 87 ?? ?? ?? ?? 48 8B 88 B8 ?? ?? ?? 48 8B 01 48 8B 40 ?? 89 DA 4D 89 F0 FF 15"
      literal_ok = ["10: mov rcx,[rax+0B8h]: PlayerInventory::mInventory, part of the mirror this entry anchors"]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 1408
    max = 1536
  }
}
