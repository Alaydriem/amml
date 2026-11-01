entry "TessellationPolicy.get" {
  kind     = "vslot"
  notes    = "Slot of mce::Color BiomeColorSampling::TessellationPolicy::get(Block const&, BlockSource&, BlockPos const&, BiomeTintCache const*) const: 2 on 26.51. The policy vtables that getTessellationPolicy stores have three slots (`inspect vtable --names` names none): 0 the destructor, 1 get(Block const&) (it reads only the policy), 2 the four-argument get. The slot 2 body takes rdx = the 16-byte Color buffer (hidden return pointer), r8 = block (it reads [r8+68h], Block::mBlockType), r9 = region and the position as the fifth argument. The tessellator call sites pass rdx = a stack buffer, r8 = block, r9 = BlockTessellator::mRegion (+8), [rsp+20h] = pos and [rsp+28h] = the tint cache at BlockTessellator+42D90h. Recipe 0 is one call site (`lea rcx,[rdi+..]; mov r9,[rdi+..]; mov rdx,[rax]; mov r10,[rdx+disp]; ..; mov rcx,rax; mov r8,r15; mov rax,r10; call [guard]`). Recipe 1 starts at another getTessellationPolicy call (after a `movups xmm0,[rbx+..]; jmp`) and takes the slot read in the same function. Both read the displacement as a slot. Range window 0..8."

  recipe {
    step "pattern" {
      bytes  = "48 8D 8F ?? ?? ?? ?? 4C 8B 4F ?? 48 8B 10 4C 8B 52 ?? 48 89 4C 24 ?? 48 89 5C 24 ?? 48 8D 54 24 ?? 48 89 C1 4D 89 F8 4C 89 D0 FF 15"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" { bytes = "0F 10 83 ?? ?? ?? ?? EB ?? 48 83 7B ?? ?? 74 ?? 48 8B 46 ?? 0F B6 88 ?? ?? ?? ?? E8" }
    step "pattern" {
      bytes  = "48 8D 8B ?? ?? ?? ?? 4C 8B 4B ?? 48 8B 10 4C 8B 52 ?? 48 89 4C 24 ?? 4C 89 74 24 ?? 48 8D 54 24 ?? 48 89 C1 49 89 F0 4C 89 D0 FF 15"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 8
  }
}
