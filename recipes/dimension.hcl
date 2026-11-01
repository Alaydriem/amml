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
  notes    = "Slot of DimensionType Dimension::getDimensionId() const in the Dimension vtable: slot 2 in 26.51 (LeviLamina header order: IDimension has the destructor, isNaturalDimension, getDimensionId), slot 3 in 26.60. It returns the id through a hidden out pointer (`mov rax,rdx; mov ecx,[rcx+disp]; mov [rdx],ecx; ret`), a leaf with no unwind data. Recipe 0 is a call-site route that does not use Dimension.vtable: Actor.getDimensionBlockSource ends in a tail jump through the getBlockSourceFromMainChunkSource slot of the dimension (`mov rax,[rsi]; mov rax,[rax+disp]; mov rdx,[guard]; mov rcx,rsi; add rsp; pop; pop; jmp rdx`); the recipe reads that displacement as a slot and goes 10 slots back (getDimensionId sits 10 slots before getBlockSourceFromMainChunkSource: slots 2 and 12 in 26.51, 3 and 13 in 26.60). This is a fixed slot distance inside one interface: a virtual added between the two makes the recipes disagree and the entry refuse; it never gives a value on its own. Recipe 1 is the leaf body itself, matched together with the next function in link order (the getBiomeRegistry forwarder: `mov rcx,[rcx+disp]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[guard]; jmp rdx`), and read as its slot in Dimension.vtable; it depends on that neighbour following the getter in the image. Every displacement and the guard address are wildcards. The check is a range window around the known slots. What changed in 6ad79ee4-17d51891a1119914: the table has one more slot before getDimensionId (every slot from the getter on is one later); the old recipe 0 used the Dimension::init source-location string, which 26.60 no longer has, and a fixed distance of 13 slots from init. Class: real code change. Evidence for slot 3 on 26.60: (1) the leaf body with its neighbour, drafted from the 26.51 getter, is found once on 26.60 and sits at slot 3 of the 26.60 table, with the old slot 3 and 4 functions at 4 and 5; (2) Actor.getDimensionBlockSource tail-jumps through slot 13 on 26.60 (slot 12 on 26.51), so the shift of one starts before the getter."

  recipe {
    step "ref" {
      entry = "Actor.getDimensionBlockSource"
    }
    step "pattern" {
      bytes  = "48 8B ?? 48 8B 40 ?? 48 8B 15 ?? ?? ?? ?? 48 89 ?? 48 83 C4 ?? 5F 5E 48 FF E2"
      offset = 3
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
  notes    = "Slot of BlockSource& Dimension::getBlockSourceFromMainChunkSource() const in the Dimension vtable: 12 in 26.51 (the slot is `mov rax,[rcx+0F0h]; ret`, a leaf; Dimension.getDimensionId recipe 0 uses the same fact). Recipe 0 is the end of Actor.getDimensionBlockSource, a tail jump through the slot (`mov rax,[rsi]; mov rax,[rax+disp]; mov rdx,[guard]; mov rcx,rsi; add rsp; pop rdi; pop rsi; jmp rdx`). Recipe 1 is a call site on a locked dimension weak reference whose result is used as a BlockSource (BlockSource.getDimensionId recipe 1). Recipe 2 is the end of IClientInstance::getRegion (ClientInstance.getRegion recipe 0), which calls the slot on the player's dimension. All three read the displacement and turn it into a slot. They are in three functions. Every displacement, immediate and branch byte is a wildcard. The range check is a window around the known slot."

  recipe {
    step "ref" { entry = "Actor.getDimensionBlockSource" }
    step "pattern" {
      bytes  = "48 8B ?? 48 8B 40 ?? 48 8B 15 ?? ?? ?? ?? 48 89 ?? 48 83 C4 ?? 5F 5E 48 FF E2"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 03 48 8B 40 ?? 48 89 D9 FF 15 ?? ?? ?? ?? 48 8B 08 4C 8B 81 ?? ?? ?? ?? 48 8D 5C 24 ?? 48 89 C1 48 89 DA 4C 89 C0 FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F0 FF 4F ?? 75 ?? 48 8B 07 48 8B 40 ?? 48 89 F9 FF 15 ?? ?? ?? ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? EB ?? 31 C0 48 83 C4 ?? 5F 5E C3"
      offset = 25
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
