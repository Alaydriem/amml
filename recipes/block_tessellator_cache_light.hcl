entry "BlockTessellatorCache.getLightColor" {
  kind      = "function"
  hooked    = true
  notes     = "PoC (dynamic-lighting). BrightnessPair BlockTessellatorCache::getLightColor(BlockPos const&, Block const*): rcx = the cache (BlockTessellator + BlockTessellator.mBlockCache), rdx = the 2-byte result (sky first, block second), r8 = pos, r9 = the block or null; returns rdx in rax. It reads a 20x20x20 window (mLightColors at cache+20h, index (x-px)*400 + (y-py)*20 + (z-pz)); a stored value of FF00h or more is a miss, filled from the region at cache+18h. Outside the window it writes 000Fh. Every terrain shape (flat and smooth light) reads light through it. Recipe 0 is the window test at the body head (`mov eax,[r8]; sub eax,[rcx+8]; jl; mov edi,[r8+4]; mov esi,edi; sub esi,[rcx+0Ch]; jl`), then function_start_of. Recipe 1 is the first call in BlockTessellator.tessellateInWorld (`lea rcx,[rsi+mBlockCache]; lea rdx,[rsp+..]; mov r8,rdi; xor r9d,r9d; call`), read as a branch target. 1.26.62.01: 0x68b0ac0."

  recipe {
    step "pattern" { bytes = "41 8B 00 2B 41 ?? 0F 8C ?? ?? ?? ?? 41 8B 78 ?? 89 FE 2B 71 ?? 0F 8C" }
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
