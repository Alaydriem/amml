entry "Dimension.vtable" {
  kind     = "global"
  notes    = "The vtable that the 26.51 value names: the primary vtable of one class derived from Dimension (the Dimension base constructor is a separate function with four callers, one per derived class, and the base destructor stores the base table; this class is the one whose constructor and deleting destructor sit next to the Dimension code). It holds every Dimension virtual at the Dimension slot (the base table and this one both hold the getDimensionId leaf at the same slot), so the slot recipes of the dimension virtuals can use it. The derived constructor calls the base constructor, then stores four vtable pointers (`lea rax,[rip+..]; mov [this],rax; lea; mov [this+8],rax; lea; mov [this+10h],rax; lea; mov [this+40h],rax`), then sets up its own std::function member (`lea rax,[this+4F0h]; mov [rbp+..],rax; mov qword ptr [this+528h],0`); the first `lea` is this vtable. Its deleting destructor stores the same four pointers into [rcx], then destroys that std::function member (`mov rcx,[rcx+528h]; test rcx,rcx; je; lea rax,[rsi+4F0h]; cmp rcx,rax; setne dl`) and calls the base destructor. Recipe 0 is the constructor run plus the member set-up; recipe 1 is the destructor run plus the member clean-up. They are two functions and their anchors differ after the run (the constructor keeps `this` in rbx or rdi, the destructor in rcx). Every REX, ModRM, displacement and immediate byte of the constructor run is a wildcard; the destructor keeps its rcx/rsi ModRM bytes (registers, not fields). The section check keeps the address inside .rdata. What changed in 6ad79ee4-17d51891a1119914: the old single recipe anchored on the three calls before the run (`mov rcx,rdi; call; mov rcx,rbx; mov rdx,rdi; call; mov rcx,rdi; call`); on 26.60 the derived constructor inlines more set-up, calls the base constructor with other registers and puts a `nop` after the call, so that anchor matched nothing. Class: would have survived with a better recipe. Evidence that 26.60 names the same class: (1) the member set-up after the run (std::function at this+4F0h/528h, then the 48h allocation stored at this+140h) is found once on 26.60, in the function that calls the 26.60 base constructor and then stores the run; (2) the deleting destructor of the old class (same members, size 538h, calls the base destructor) is found once on 26.60 and stores the same table; and the old slot contents line up: the getDimensionId leaf and its two following neighbours, and the getBlockSourceFromMainChunkSource slot that Actor.getDimensionBlockSource tail-jumps through, all sit one slot later on 26.60."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 ?? ?? 48 8D ?? ?? ?? ?? ?? 48 89 45 ?? 48 C7 ?? ?? ?? ?? ?? ?? ?? ?? ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 05 ?? ?? ?? ?? 48 89 41 ?? 48 8D 05 ?? ?? ?? ?? 48 89 41 ?? 48 8D 05 ?? ?? ?? ?? 48 89 41 ?? 48 8B 89 ?? ?? ?? ?? 48 85 C9 74 ?? 48 8D 86 ?? ?? ?? ?? 48 39 C1 0F 95 C2"
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

entry "Dimension.getDimensionId" {
  kind     = "vslot"
  notes    = "Slot of DimensionType Dimension::getDimensionId() const in the Dimension vtable: slot 2 in 26.51 (LeviLamina header order: IDimension has the destructor, isNaturalDimension, getDimensionId), slot 3 in 26.60. It returns the id through a hidden out pointer (`mov rax,rdx; mov ecx,[rcx+disp]; mov [rdx],ecx; ret`), a leaf with no unwind data. Recipe 0 is a call-site route that does not use Dimension.vtable: the end of IClientInstance::getRegion, which locks the weak reference to the player's dimension and calls the getBlockSourceFromMainChunkSource slot of that dimension; the recipe reads that displacement as a slot and goes 10 slots back (getDimensionId sits 10 slots before getBlockSourceFromMainChunkSource: slots 2 and 12 in 26.51, 3 and 13 in 26.60 and later). It is a pattern_any with one alternative per body shape, both with the displacement load at index 21 of their bytes: alternative 0 (26.60 to 26.63) is the tail after the weak-reference release (`jne; mov rax,[rdi]; mov rax,[rax+disp]; mov rcx,rdi; call [guard]; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; jmp; xor eax,eax; add rsp; pop rdi; pop rsi; ret`), alternative 1 (1.26.70.26) is the lock call, the call that gets the dimension and the slot call (`call; test rax,rax; je; mov rcx,rax; call; mov rcx,[rax]; mov rdx,[rcx+disp]; mov rcx,rax; mov rax,rdx; call [guard]; jmp; xor eax,eax; mov rcx,[rsp+..]`). This is a fixed slot distance inside one interface: a virtual added between the two makes the recipes disagree and the entry refuse; it never gives a value on its own. Recipe 1 is the leaf body itself, matched together with the next function in link order (the getBiomeRegistry forwarder: `mov rcx,[rcx+disp]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[guard]; jmp rdx`), and read as its slot in Dimension.vtable; it depends on that neighbour following the getter in the image. Every displacement and the guard address are wildcards. The check is a range window around the known slots. What changed in 6afa4e47-dadc375797a3d31a: Actor.getDimensionBlockSource is inlined into its callers (no out-of-line function), so the old recipe 0 (a tail jump in that function) has nothing to anchor on; recipe 0 now starts from IClientInstance::getRegion, which has the same call through the slot, and Dimension.vtable and the leaf body of recipe 1 are unchanged (slot 3); class: would have survived with a better recipe. Evidence on 6afa4e47: (1) the getRegion tail drafted from the 26.63 function matches once on 26.63, and the new alternative matches once in the 70.26 function that sits in ClientInstance.getRegion's slot (12), which calls the slot at displacement 68h, slot 13, ten after the getter; (2) the leaf body of recipe 1, drafted from the 26.60 getter, matches once on 70.26 and sits at slot 3 of the 70.26 Dimension.vtable, and the slot of the call in recipe 0 minus 10 gives the same slot. What changed in 6ad79ee4-17d51891a1119914: the table has one more slot before getDimensionId (every slot from the getter on is one later); the old recipe 0 used the Dimension::init source-location string, which 26.60 no longer has, and a fixed distance of 13 slots from init. Class: real code change. Evidence for slot 3 on 26.60: (1) the leaf body with its neighbour, drafted from the 26.51 getter, is found once on 26.60 and sits at slot 3 of the 26.60 table, with the old slot 3 and 4 functions at 4 and 5; (2) Actor.getDimensionBlockSource tail-jumps through slot 13 on 26.60 (slot 12 on 26.51), so the shift of one starts before the getter."

  recipe {
    step "pattern_any" {
      patterns = [
        "75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? EB ?? 31 C0 48 83 C4 ?? 5F 5E C3",
        "E8 ?? ?? ?? ?? 48 85 C0 74 ?? 48 89 C1 E8 ?? ?? ?? ?? 48 8B 08 48 8B 51 ?? 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? EB ?? 31 C0 48 8B 4C 24 ??",
      ]
      offset = 21
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
    step "add" {
      value = -10
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 D0 8B 89 ?? ?? ?? ?? 89 0A C3 CC CC CC CC 48 8B 89 ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ??"
      offset = 0
    }
    step "slot_of" {
      vtable = "Dimension.vtable"
    }
  }

  check "range" {
    min = 0
    max = 8
  }
}

entry "Dimension.getBlockSourceFromMainChunkSource" {
  kind     = "vslot"
  notes    = "Slot of BlockSource& Dimension::getBlockSourceFromMainChunkSource() const in the Dimension vtable: 12 in 26.51, 13 (0xd) in 26.63 and in 6afa4e47-dadc375797a3d31a (70.26). The function returns the member at Dimension+0F0h: a leaf `mov rax,[rcx+0F0h]; ret` on 26.63; on 70.26 it has a frame, asserts that the member is not null and loads its own `virtual BlockSource &Dimension::getBlockSourceFromMainChunkSource() const` text. Recipe 0 is the end of IClientInstance::getRegion (the ClientInstance.getRegion slot body), which calls the slot on the player's dimension. It is one pattern_any with one alternative per code shape, both with the slot instruction at offset 21: alternative 0 (26.63) is the release of the locked dimension weak reference and the call (`jne; mov rax,[rdi]; mov rax,[rax+disp8]; mov rcx,rdi; call [guard]; mov rax,[rsi]; mov rax,[rax+disp8]; mov rcx,rsi; call [guard]; jmp; xor eax,eax; add rsp; pop rdi; pop rsi; ret`); alternative 1 (70.26) is the call of the dimension getter and the call (`call; test rax,rax; je; mov rcx,rax; call; mov rcx,[rax]; mov rdx,[rcx+disp8]; mov rcx,rax; mov rax,rdx; call [guard]; jmp; xor eax,eax`). Recipe 1 is a virtual function that reads the player's dimension block source and then calls BlockSource slot 90h (BlockSource.getDimensionId) into a local (`lea rbx,[rsp+..]`) before three direct calls; on 26.63 it inlines the weak-reference lock, on 70.26 it calls the dimension getter. It is one pattern_any, slot instruction at offset 3: alternative 0 (26.63) `mov rax,[rbx]; mov rax,[rax+disp8]; mov rcx,rbx; call [guard]; mov rcx,[rax]; mov r8,[rcx+disp32]; lea rbx,[rsp+..]; mov rcx,rax; mov rdx,rbx; mov rax,r8; call [guard]`; alternative 1 (70.26) `mov rcx,[rax]; mov rdx,[rcx+disp8]; mov rcx,rax; mov rax,rdx; call [guard]` followed by the same BlockSource call. Both recipes read the displacement and turn it into a slot. They are in two functions and share no bytes. Each alternative matches once on its own build and not on the other. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slots. The former recipe from the end of Actor::getDimensionBlockSource is removed: that function is inlined into its callers on 70.26, so the entry no longer depends on Actor.getDimensionBlockSource. Evidence on 70.26 for slot 13: (1) vtable neighbours: slots 0..23 of the Dimension vtable have the same function and non-function layout on both builds, slots 4, 5 and 16 name themselves (sendPacketForPosition, sendPacketForEntity, init), and slot 13 names itself getBlockSourceFromMainChunkSource and reads the same member +0F0h as the 26.63 slot 13 leaf; (2) callers: the ClientInstance.getRegion body (a resolved slot on both builds) calls displacement 68h on the player's dimension on both builds, and the virtual function of recipe 1, found on 70.26 by a pattern drafted from its 26.63 body (the BlockSource slot 90h call and the three direct calls, one match), calls displacement 68h on the dimension before it. No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a: Actor::getDimensionBlockSource and the inlined dimension weak-reference lock became a call to the Actor dimension getter, and the slot function gained an assert; the slot number did not change. Class: real code change (call-site shapes)."

  recipe {
    step "pattern_any" {
      patterns = [
        "75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? EB ?? 31 C0 48 83 C4 ?? 5F 5E C3",
        "E8 ?? ?? ?? ?? 48 85 C0 74 ?? 48 89 C1 E8 ?? ?? ?? ?? 48 8B 08 48 8B 51 ?? 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? EB ?? 31 C0",
      ]
      offset   = 21
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
        "48 8B 03 48 8B 40 ?? 48 89 D9 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 81 ?? ?? ?? ?? 48 8D 5C 24 ?? 48 89 C1 48 89 DA 4C 89 C0 FF 15",
        "48 8B 08 48 8B 51 ?? 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 81 ?? ?? ?? ?? 48 8D 5C 24 ?? 48 89 C1 48 89 DA 4C 89 C0 FF 15",
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
    min = 6
    max = 20
  }
}

entry "Dimension.mSkyDarken" {
  kind  = "field"
  notes = "draft"

  recipe {
    step "pattern" {
      bytes  = "F3 0F 59 ?? ?? ?? ?? ?? F3 0F 2C ?? 38 ?? ?? ?? ?? ?? 74 ?? 88 ?? ?? ?? ?? ??"
      offset = 20
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 31 E0 48 89 44 24 ?? 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 89 C3 48 8B 47 ?? 44 0F B6 88 ?? ?? ?? ??",
        "48 89 44 24 ?? 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 89 C6 48 8B 03 48 8B 4B ?? 0F B6 A9 ?? ?? ?? ??",
      ]
      offset = 35
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 384
    max = 512
  }
}
