entry "BlockTessellatorCache.getBlock" {
  kind      = "function"
  msvc_name = "?getBlock@BlockTessellatorCache@@QEAAAEBVBlock@@AEBVBlockPos@@@Z"
  notes     = "Block const& BlockTessellatorCache::getBlock(BlockPos const&): rcx = the cache (BlockTessellator+6B0h), rdx = pos. It sets the air block at +0 on first use, then reads mBlocks (+3EA0h) inside the 20x20x20 window at mPos (+8): index (x-px)*400 + (y-py)*20 + (z-pz); outside the window it asks the region at +18h. Recipe 0 is the window test (`setl bpl; cmp r9d,..; setl r14b; and r14b,bpl; cmp r10d,..; setl r10b; and r10b,r11b; and r10b,r14b`), then function_start_of. Recipe 1 is a call in a cache helper (`mov r15,r9; mov ebp,r8d; mov r12,rcx; mov rcx,[rcx+..]; mov r14,rdx; call`). Check function_start. The ModRM byte of `mov r32,r8d` in recipe 1 is a wildcard. What changed in 6ad79ee4-17d51891a1119914 (26.60): the helper keeps r8d in ebx instead of ebp, so recipe 1 did not match. The helper is the same on both builds (the face tests, then the three calls to this function); recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "40 0F 9C C5 41 83 F9 ?? 41 0F 9C C6 41 20 EE 41 83 FA ?? 41 0F 9C C2 45 20 DA 45 20 F2" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "4D 89 CF 44 89 ?? 49 89 CC 48 8B 49 ?? 49 89 D6 E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
