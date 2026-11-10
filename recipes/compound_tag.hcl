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

entry "CompoundTag.vtable" {
  kind  = "global"
  notes = "The vtable of CompoundTag (no RTTI), a helper for the in_vtable check of CompoundTag.write. Its slots follow the Tag interface (LeviLamina header): 0 the deleting destructor, 1 deleteChildren, 2 write, 3 load, 4 toString, 5 getId (returns 0Ah), then equals, print, copy, hash. Recipe 0 is the vptr store in CompoundTag.dtor (`lea rax,[rip+..]; mov [rcx],rax`), read as the RIP target of the lea. Recipe 1 is the vptr store in NbtIo.read, where the function builds the empty CompoundTag it returns (`lea rax,[rip+..]; mov [rdi],rax; xorps xmm0,xmm0`), read the same way. The two recipes are in two functions. The RIP offsets are wildcards. Check section .rdata. Values: 0xE821430 on 6aa482fd-2658118eba0dbac5 (26.51), 0xF152DB0 on 6ad79ee4-17d51891a1119914 (26.60), 0xF153DA0 on 6aea437e-7eff13daabb430da (26.62), 0xF153D80 on 6aed1906-b127418e74d2f2f9 (26.63). Added on 26.63; both stores are the same code on all four builds."

  recipe {
    step "ref" { entry = "CompoundTag.dtor" }
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 01" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "ref" { entry = "NbtIo.read" }
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 07 0F 57 C0" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "CompoundTag.write" {
  kind      = "function"
  msvc_name = "?write@CompoundTag@@UEBAXAEAVIDataOutput@@@Z"
  notes     = "void CompoundTag::write(IDataOutput&) const: rcx the tag, rdx the output; slot 2 of CompoundTag.vtable (the Tag::write slot). The virtual is an 8-byte thunk without .pdata and without a direct caller: `xor r8d,r8d; jmp` into the next function, the overload CompoundTag::write(IDataOutput&, int depth) const (r8 the depth; the LeviLamina header has both). That writer walks the map (`mov r15,[rcx+8]; mov r12,[r15]`), throws bad_variant_access on a node whose index byte +68h is -1, and for each node writes the tag id (getId, Tag slot 5, then IDataOutput slot 5, writeByte), writes the key (IDataOutput slot 1, writeString), then writes the value: a ListTag (id 9) through the list writer with depth+1, a CompoundTag (id 0Ah) through a direct recursive call with depth+1, any other tag through Tag slot 2 (`call [rax+10h]` with rcx the tag, rdx the output). It ends with writeByte(0), the End tag. The value is the thunk, so a mod calls it with two arguments and needs no depth. It was found from CompoundTag.dtor: its vptr store gives the vtable; slot 2 is the thunk right after the destructor. Recipe 0 is the thunk at its first byte, its int3 padding and the head of the writer (`xor r8d,r8d; jmp; int3 x8; push r15; push r14; push r12; push rsi; push rdi; push rbx; sub rsp,..; mov rsi,rdx; mov rax,[rip+..]; xor rax,rsp; mov [rsp+..],rax; mov r15,[rcx+..]; mov r12,[r15]; cmp r12,r15; je; mov edi,r8d; inc edi`); the jmp offset and every other displacement and immediate are wildcards. Alone the thunk shape matches 52 to 58 times per build; the writer head makes it unique. Recipe 1 is the tail of CompoundTag::~CompoundTag and the padding after it (`mov rcx,[rsi+8]; mov edx,70h; add rsp,..; pop rsi; jmp; int3 x7`), offset 26, the byte after the padding. It keeps the 70h node size (other map destructors free other sizes) and the map member +8 (a second destructor with the same tail and a 70h node reads +10h; it is followed by 8 int3, so without this byte it matched twice). Both recipes depend on link order (the destructor, then the thunk, then the writer); on a new build they fail, they do not give a wrong value. They share no pattern bytes. Check in_vtable CompoundTag.vtable slots 0..6 (slot 2 on every build; Tag has 11 virtuals in the LeviLamina header). msvc_name from the LeviLamina header signature. Values: 0x2546470 on 6aa482fd-2658118eba0dbac5 (26.51), 0x2963580 on 6ad79ee4-17d51891a1119914 (26.60), 0x29634E0 on 6aea437e-7eff13daabb430da (26.62), 0x2963240 on 6aed1906-b127418e74d2f2f9 (26.63). Added on 26.63; the thunk, padding and writer head are byte-identical on all four builds."

  recipe {
    step "pattern" { bytes = "45 31 C0 E9 ?? ?? ?? ?? CC CC CC CC CC CC CC CC 41 57 41 56 41 54 56 57 53 48 83 EC ?? 48 89 D6 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 4C 8B 79 ?? 4D 8B 27 4D 39 FC 0F 84 ?? ?? ?? ?? 44 89 C7 FF C7" }
  }

  recipe {
    step "pattern" {
      bytes      = "48 8B 4E 08 BA 70 00 00 00 48 83 C4 ?? 5E E9 ?? ?? ?? ?? CC CC CC CC CC CC CC"
      literal_ok = [
        "3: the map member at +8: another destructor with the same tail frees a 70h node from +10h",
        "5: head node size 70h of map<string, CompoundTagVariant>: other map destructors free other node sizes",
        "6: upper bytes of the node size",
        "7: upper bytes of the node size",
        "8: upper bytes of the node size",
      ]
      offset     = 26
    }
  }

  check "in_vtable" {
    vtable   = "CompoundTag.vtable"
    min_slot = 0
    max_slot = 6
  }
}
