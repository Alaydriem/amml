entry "CompoundTag.dtor" {
  kind      = "function"
  msvc_name = "??1CompoundTag@@UEAA@XZ"
  notes     = "CompoundTag::~CompoundTag(): rcx = this (357 callers). It stores the CompoundTag vtable (the table whose slot 5, getId, returns 0Ah), destroys the tag map tree and frees the 70h-byte head node; the constructor before it allocates that head node. The deleting destructor (vtable slot 0) inlines the same code. Recipe 0 is the whole body at its first byte (`push rsi; sub rsp,..; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; add rcx,8; mov rax,[rsi+..]; mov r8,[rax+..]; mov rdx,rcx; call; mov rcx,[rsi+..]; mov edx,70h; add rsp,..; pop rsi; jmp`), so function_start is a real check; the head node size 70h is kept, because two other map-owning destructors have the same code with other node sizes. Recipe 1 is a call site in a function that frees a 70h-byte map head node (`mov rcx,[rdi+8]; mov edx,imm32; call delete; mov rdx,rbx; jmp`) and next to it destroys the CompoundTag in rdi (`mov rcx,rdi; call`); the pattern runs past the delete call and the jmp (no nop on either build). Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site of recipe 1 (`or [r14+rax*8+..],rdx; lea rcx,[rbp+..]; call`) is gone; the new site was found by drafting every 26.60 call site of this function and keeping one whose pattern matches once on each build and calls the 26.51 value there. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes      = "56 48 83 EC ?? 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 83 C1 ?? 48 8B 46 ?? 4C 8B 40 ?? 48 89 CA E8 ?? ?? ?? ?? 48 8B 4E ?? BA 70 00 00 00 48 83 C4 ?? 5E E9"
      literal_ok = [
        "43: head node size 70h of map<string, CompoundTagVariant>: two other destructors free other node sizes",
        "44: upper bytes of the node size",
        "45: upper bytes of the node size",
        "46: upper bytes of the node size",
      ]
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 DA E9 ?? ?? ?? ?? 48 89 F9 E8"
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "CompoundTag.nodeVariantIndex" {
  kind     = "field"
  notes    = "Offset of the variant index byte in a node of the CompoundTag map (std::map<std::string, CompoundTagVariant>): 68h on 26.60. A node is: left +0, parent +8, right +10h, isnil byte +19h, the key std::string +20h, the value (the Tag object itself, with its vtable at +0) +40h, and this index byte at +68h (the index equals Tag::Type: Byte 1, Int 3, List 9, Compound 10; -1 is valueless). The typed accessors read it after the tree search: CompoundTag::getList `cmp byte ptr [rbx+68h],9` and the Byte accessor `cmp byte ptr [rbx+68h],1`. Together with the node layout in the amml-game mirror it lets a mod walk the map without a game call. Recipe 0 is the tail of the List accessor (`cmp rbx,r14; je; test cl,cl; jne; lea rcx,[rbx+40h]; xor eax,eax; cmp byte ptr [rbx+68h],9; cmovne..; add rsp,..; pop` seven registers; ret), read as the displacement of the compare. Recipe 1 is the same tail of the Byte accessor (`... cmp byte ptr [rbx+68h],1 ...`). The two accessors are the same code with other type indices, so the immediate is kept in each (9 and 1) to tell them apart. The recipes are in two functions. Every other displacement, immediate and branch byte is a wildcard. The check is a window around the known value. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 68h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes      = "4C 39 F3 74 ?? 84 C9 75 ?? 48 8D 4B ?? 31 C0 80 7B ?? 09 48 0F 44 C1 48 83 C4 ?? 5B 5F 5E 41 5C 41 5D 41 5E 41 5F C3"
      literal_ok = [
        "18: the type index 9 (List): the Byte and Int accessors have the same tail with other indices",
      ]
      offset     = 15
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes      = "4C 39 F3 74 ?? 84 C9 75 ?? 48 8D 4B ?? 31 C0 80 7B ?? 01 48 0F 44 C1 48 83 C4 ?? 5B 5F 5E 41 5C 41 5D 41 5E 41 5F C3"
      literal_ok = [
        "18: the type index 1 (Byte): the List and Int accessors have the same tail with other indices",
      ]
      offset     = 15
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 256
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}
