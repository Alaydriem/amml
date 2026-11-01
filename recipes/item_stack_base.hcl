entry "ItemStackBase.dtor" {
  kind     = "function"
  notes    = "ItemStackBase::~ItemStackBase() body (not a deleting destructor): rcx the stack or item instance. It stores the ItemStackBase vtable, releases mChargedItem (+78h, a deleting destructor call), frees the two block vectors, mUserData and mBlock, and drops the WeakPtr<Item> count (+8). ItemStack::~ItemStack (its own function, 1000 callers) stores the ItemStack vtable, destroys mNetIdVariant when it holds a request id (index 1 or 2) and tail-jumps here; ItemInstance and the other bases tail-jump here too (3324 direct callers and jumps). Recipe 0 is the WeakPtr release in the body: `mov rax,[rsi+..]; test rax,rax; je; lock dec dword ptr [rax+..]; cmp dword ptr [rax+..],0; jg; cmp qword ptr [rax],0`, then function_start_of. Recipe 1 is a caller that destroys a local after moving a member out: `mov rax,[rbp+..]; mov [r15+..],rax; lea rcx,[rbp+..]; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 0 is now a pattern_any, one alternative per shape of the weak count test (26.51 `cmp dword ptr [rax+0Ch],0`, 26.60 `mov ecx,[rax+0Ch]; test ecx,ecx`). What changed in 6ad79ee4-17d51891a1119914 (26.60): that compare form only; recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 46 ?? 48 85 C0 74 ?? F0 FF 48 ?? 83 78 ?? ?? 7F ?? 48 83 38 ??",
        "48 8B 46 ?? 48 85 C0 74 ?? F0 FF 48 ?? 8B 48 ?? 85 C9 7F ?? 48 83 38 ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 85 ?? ?? ?? ?? 49 89 87 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ItemStackBase.getMaxStackSize" {
  kind     = "function"
  notes    = "uchar ItemStackBase::getMaxStackSize() const: rcx the stack, the size in al. It reads the WeakPtr<Item> counter (`mov rdx,[rcx+8]`; 0FFh when the counter or the item is null), builds an ItemDescriptor of this stack on the stack, calls Item::getMaxStackSize(ItemDescriptor const&) (Item slot 100, `mov rax,[rax+320h]`) and destroys the descriptor. It sits next to ItemStackBase::~ItemStackBase. Recipe 0 is the body up to the Item call: `mov rdx,[rcx+..]; mov al,..; test rdx,rdx; je; mov rsi,[rdx]; test rsi,rsi; je; lea rdi,[rbp-..]; mov rdx,rdi; call; mov rax,[rsi]; mov rax,[rax+..]; mov rcx,rsi; mov rdx,rdi; call [guard]`, then function_start_of. Recipe 1 is a caller that compares the result: `mov byte ptr [rbp+..],..; movzx edi,byte ptr [rbp+..]; lea rcx,[rbp+..]; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 1 is now another call site: `cmp [rbp+..],dl; jae; lea rcx,[rbp+..]; call`. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site of recipe 1 changed shape; the new site was found by drafting each 26.60 call of this function and keeping one whose pattern matches once on each build and calls the 26.51 value there. Recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 8B 51 ?? B0 ?? 48 85 D2 74 ?? 48 8B 32 48 85 F6 74 ?? 48 8D 7D ?? 48 89 FA E8 ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 48 89 FA FF 15" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "38 95 ?? ?? ?? ?? 0F 83 ?? ?? ?? ?? 48 8D 4D ?? E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ItemStackBase.getDamageValue" {
  kind     = "function"
  notes    = "short ItemStackBase::getDamageValue() const: rcx the stack, the damage in ax. It needs the WeakPtr<Item> at +8 and the user data at +10h, looks up the IntTag `Damage` in the user data (CompoundTag::get, then the Int accessor) and returns its low 16 bits; 0 without user data or tag. 26 direct callers. Recipe 0 is the prologue (`push rsi; push rdi; sub rsp,..; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rax,[rcx+8]; test rax,rax; je; cmp qword ptr [rax],0; je; mov rsi,[rcx+..]`). Recipe 1 is a call site after a null test of a local (`test edi,edi; je; mov rcx,[rbp+..]; call`), read as the call target. Recipe 2 is a call site in a function that returns zero on the other branch (`jmp; xor r15d,r15d; mov rcx,r14; call`), read the same way. The three recipes are in three functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 1ACF510h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "56 57 48 83 EC ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 41 ?? 48 85 C0 74 ?? 48 83 38 ?? 74 ?? 48 8B 71 ??" }
  }

  recipe {
    step "pattern" {
      bytes  = "85 FF 0F 84 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? E8 ?? ?? ?? ??"
      offset = 15
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "E9 ?? ?? ?? ?? 45 31 FF 4C 89 F1 E8 ?? ?? ?? ??"
      offset = 11
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "ItemStackBase.isDamaged" {
  kind     = "function"
  notes    = "bool ItemStackBase::isDamaged() const: rcx the stack, the bool in al. The name is from behaviour: it asks the Item whether it is damageable (Item virtual at +70h, slot 14), returns false for a stack whose user data has the ByteTag `Unbreakable`, and otherwise tests that the damage value is above 0. It replaces the plan's isDamageableItem, which is not a function of this build. 9 callers (one is a tail jump). Recipe 0 is the prologue (`push rsi; push rdi; sub rsp,..; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov rax,[rcx+8]; test rax,rax; je; mov rdx,[rax]; test rdx,rdx`). Recipe 1 is a call site that follows the damageable test on an Item (`mov rsi,rax; mov rax,[rax]; mov rax,[rax+..]; mov rcx,rsi; call [guard]; test al,al; je; mov rcx,rsi; call`), read as the call target. Recipe 2 is a second call site with the same test in a function that returns early (`mov rdi,rcx; mov rax,[rdx]; mov rax,[rax+..]; mov rcx,rdx; call [guard]; mov ecx,eax; mov al,..; test cl,cl; je; mov rcx,rsi; call`), read the same way. The three recipes are in three functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 1AD1050h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "56 57 48 83 EC ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 41 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 10 48 85 D2" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 C6 48 8B 00 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 89 F1 E8 ?? ?? ?? ??"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 02 48 8B 40 ?? 48 89 D1 FF 15 ?? ?? ?? ?? 89 C1 B0 ?? 84 C9 0F 84 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ??"
      offset = 34
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
