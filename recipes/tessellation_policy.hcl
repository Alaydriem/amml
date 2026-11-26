entry "TessellationPolicy.get" {
  kind     = "vslot"
  notes    = "Slot of mce::Color BiomeColorSampling::TessellationPolicy::get(Block const&, BlockSource&, BlockPos const&, BiomeTintCache const*) const: 2 on 26.51. The policy vtables that getTessellationPolicy stores have three slots (`inspect vtable --names` names none): 0 the destructor, 1 get(Block const&) (it reads only the policy), 2 the four-argument get. The slot 2 body takes rdx = the 16-byte Color buffer (hidden return pointer), r8 = block (it reads [r8+68h], Block::mBlockType), r9 = region and the position as the fifth argument. The tessellator call sites pass rdx = a stack buffer, r8 = block, r9 = BlockTessellator::mRegion (+8), [rsp+20h] = pos and [rsp+28h] = the tint cache at BlockTessellator+42D90h. Recipe 0 is one call site (`lea rcx,[rdi+..]; mov r9,[rdi+..]; mov rdx,[rax]; mov r10,[rdx+disp]; ..; mov rcx,rax; mov r8,reg; mov rax,r10; call [guard]`), told apart from the sibling sites of the same shape by what follows it: the second call (`mov r8,[rdi+..]; mov rcx,[r14+..]; mov rax,[rax+disp]; mov rdx,r14; mov r9,rbx; call [guard]; test al,al; je short; mov dword ptr [rsp+..],imm; mov rcx,[rdi+..]`); the register of `mov r8` is a wildcard and the float immediate is a wildcard. Recipe 1 starts at another getTessellationPolicy call (after a `movups xmm0,[rbx+..]; jmp`) and takes the slot read in the same function, a pattern_any with one alternative per encoding of `lea rdx,[rsp+..]` (disp8 on 26.60 to 26.63, disp32 on 1.26.70.26). Both read the displacement as a slot. Range window 0..8. What changed in 6afa4e47-dadc375797a3d31a: the frame of the second function grew, so `lea rdx,[rsp+..]` has a 32-bit displacement; the sibling site that recipe 0 used on 26.63 is no longer the only one with its register shape (three sites match its bytes), so recipe 0 now uses the tail after the call; class: would have survived with a better recipe. Evidence on 6afa4e47: (1) the call-site body drafted from the 26.63 getTessellationPolicy user matches once in the new exe (recipe 0) and, in the new function that recipe 1 finds, the same shape with the disp32 lea (recipe 1), both read displacement 10h, slot 2; (2) both sites call the new getTessellationPolicy result (a call that follows `movzx ecx,byte ptr [rax+17Ch]`, as on 26.63) and the slot-2 call is the four-argument get: r9 = the region at [rbx+8], [rsp+20h] and [rsp+28h] = position and the tint cache at +42D90h, the same arguments as the old notes describe."

  recipe {
    step "pattern" {
      bytes  = "48 8D 8F ?? ?? ?? ?? 4C 8B 4F ?? 48 8B 10 4C 8B 52 ?? 48 89 4C 24 ?? 48 89 5C 24 ?? 48 8D 54 24 ?? 48 89 C1 4D 89 ?? 4C 89 D0 FF 15 ?? ?? ?? ?? 4C 8B 47 ?? 49 8B 4E ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 4C 89 F2 49 89 D9 FF 15 ?? ?? ?? ?? 84 C0 74 ?? C7 44 24 ?? ?? ?? ?? ?? 48 8B 4F ??"
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
    step "pattern_any" {
      patterns = [
        "48 8D 8B ?? ?? ?? ?? 4C 8B 4B ?? 48 8B 10 4C 8B 52 ?? 48 89 4C 24 ?? 4C 89 74 24 ?? 48 8D 54 24 ?? 48 89 C1 49 89 F0 4C 89 D0 FF 15",
        "48 8D 8B ?? ?? ?? ?? 4C 8B 4B ?? 48 8B 10 4C 8B 52 ?? 48 89 4C 24 ?? 4C 89 74 24 ?? 48 8D 94 24 ?? ?? ?? ?? 48 89 C1 49 89 F0 4C 89 D0 FF 15",
      ]
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
