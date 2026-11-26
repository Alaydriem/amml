entry "BlockTessellatorCache.getLightColor" {
  kind      = "function"
  hooked    = true
  notes     = "PoC (dynamic-lighting). BrightnessPair BlockTessellatorCache::getLightColor(BlockPos const&, Block const*): rcx = the cache (BlockTessellator + BlockTessellator.mBlockCache), rdx = the 2-byte result (sky first, block second), r8 = pos, r9 = the block or null; returns rdx in rax. It reads a 20x20x20 window (mLightColors at cache+20h, index (x-px)*400 + (y-py)*20 + (z-pz)); a stored value of FF00h or more is a miss, filled from the region at cache+18h. Outside the window it writes 000Fh. Every terrain shape (flat and smooth light) reads light through it. Recipe 0 is the first read of the window array in the body: `movzx eax,word ptr [rcx+idx*2+20h]; cmp eax,FEFFh; jbe; mov reg,[rcx+18h]; test reg,reg; je` (the miss test and the load of the region pointer), then function_start_of. It is a pattern_any with one alternative for each build shape: 26.63 reads the index from rbx and the region into r10; 70.26 moves the arguments into rbx, rsi and rdi first, so the index is in r15, the region in rcx and a 32-bit load of the position follows. Every displacement, immediate and branch byte is a wildcard. No build gate: builds before 26.63 are not supported. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the window test at the body head is a branchless compare (`setl; or; jne`) and the index math is regrouped, so the old head pattern did not match; the value did not move (recipe 1 still gave it). Class: would have survived with a better recipe. Evidence: (1) the old-build array read drafted from 26.63 matches once there and the 70.26 shape matches once on 70.26, in the function that recipe 1 reaches; (2) recipe 1 (the call in tessellateInWorld) gives the same function. Recipe 1 is the first call in BlockTessellator.tessellateInWorld (`lea rcx,[rsi+mBlockCache]; lea rdx,[rsp+..]; mov r8,rdi; xor r9d,r9d; call`), read as a branch target. 1.26.62.01: 0x68b0ac0."

  recipe {
    step "pattern_any" {
      patterns = [
        "0F B7 44 59 ?? 3D ?? ?? ?? ?? 0F 86 ?? ?? ?? ?? 4C 8B 51 ?? 4D 85 D2 0F 84 ?? ?? ?? ?? 49 8B 02",
        "42 0F B7 44 7F ?? 3D ?? ?? ?? ?? 0F 86 ?? ?? ?? ?? 48 8B 4F ?? 48 85 C9 0F 84 ?? ?? ?? ?? 8B 6B ??",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "BlockTessellator.tessellateInWorld" }
    step "pattern" {
      bytes  = "48 8D 8E ?? ?? ?? ?? 48 8D 54 24 ?? 49 89 F8 45 31 C9 E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
