entry "BlockTessellator.ctor" {
  kind      = "function"
  msvc_name = "??0BlockTessellator@@QEAA@PEAVBlockSource@@@Z"
  notes     = "BlockTessellator::BlockTessellator(BlockSource*): rcx = this (293344 = 479E0h bytes, the header size, from the game allocator), rdx = the region (may be null). The ClientInstance init allocates 479E0h, calls it with a null region and stores the object at ClientInstance+610h (what getBlockTessellator, slot 349, returns); the LevelRenderer function that rebuilds its own tessellator at LevelRenderer+390h calls it with the region, after BlockTessellator.setRegion on the old one. The body stores rdx at this+8 (mRegion) and memsets the block cache at this+6B0h (426A0h bytes). Recipe 0 is the body head (`mov rsi,rdx; mov r15,rcx; mov word ptr [rcx],..; mov byte ptr [rcx+..],..; mov [rcx+..],rdx; mov dword ptr [rcx+..],..` x2), then function_start_of. Recipe 1 starts at the setRegion call of that LevelRenderer function (`cmp [rcx+8],rdi; je; mov rdx,rdi; call`) and takes the call after the allocation in the same function (`mov rbx,rax; test rax,rax; je; mov rcx,rbx; mov rdx,rdi; mov [rbp+..],rbx; call`). Check function_start."

  recipe {
    step "pattern" { bytes = "48 89 D6 49 89 CF 66 C7 01 ?? ?? C6 41 ?? ?? 48 89 51 ?? C7 41 ?? ?? ?? ?? ?? C7 41 ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "48 39 79 ?? 0F 84 ?? ?? ?? ?? 48 89 FA E8" }
    step "pattern" {
      bytes  = "48 89 C3 48 85 C0 74 ?? 48 89 D9 48 89 FA 48 89 5D ?? E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockTessellator.dtor" {
  kind      = "function"
  msvc_name = "??1BlockTessellator@@QEAA@XZ"
  notes     = "BlockTessellator::~BlockTessellator(): rcx = this; the caller frees the 479E0h bytes after it. It frees the vector at this+479B8h and destroys the members near the end of the object. The ClientInstance init and the LevelRenderer rebuild function both destroy the old tessellator with it before a sized delete of 479E0h. Recipe 0 starts at the setRegion call of the LevelRenderer rebuild function and takes `test rdi,rdi; je; mov rcx,rdi; call` in the same function. Recipe 1 is the LevelRenderer destructor: `lea rax,[rsi+..]; mov qword ptr [rax],0; mov rdi,[rsi+..]; test rdi,rdi; je; mov rcx,rdi; call` (the LevelRenderer tessellator at +390h). Check function_start. The ModRM bytes of the load, the test and the mov in recipe 1 are wildcards. What changed in 6ad79ee4-17d51891a1119914 (26.60): the LevelRenderer destructor holds the tessellator in rbx instead of rdi, so recipe 1 did not match. The new site was accepted on two facts from the 26.51 site: the same sequence (clear a qword, load the tessellator, destroy it, sized delete of 479E0h), and the same callers count of the destructor (7 on each build). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 39 79 ?? 0F 84 ?? ?? ?? ?? 48 89 FA E8" }
    step "pattern" {
      bytes  = "48 85 FF 74 ?? 48 89 F9 E8"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 86 ?? ?? ?? ?? 48 C7 00 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 48 85 ?? 74 ?? 48 89 ?? E8"
      offset = 29
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockTessellator.setRegion" {
  kind      = "function"
  msvc_name = "?setRegion@BlockTessellator@@QEAAXAEAVBlockSource@@@Z"
  notes     = "void BlockTessellator::setRegion(BlockSource&): rcx = this, rdx = the region. It stores the region at this+8 (mRegion) and this+6C8h (the cache mRegion, cache+18h), resets the cache position (6B8h) to -2, fills mLightColors (6D0h, 3E80h bytes) with FFh and fills mBlocks (4550h) with the air block. Recipe 0 is the store run of the body (`mov [rcx+8],rdx; mov [rcx+6C8h],rdx; mov rax,imm64; mov [rcx+6B8h],rax; mov dword ptr [rcx+6C0h],imm32`), then function_start_of; every displacement and immediate is a wildcard. Recipe 1 is the call in the LevelRenderer rebuild function, which calls it on the old tessellator when the region changed: `cmp [rcx+8],rdi; je; mov rdx,rdi; call`. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): the body sets up a stack cookie between `mov rsi,rcx` and the stores, so the 26.51 head pattern did not match; the stores and their offsets are the same. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 51 ?? 48 89 91 ?? ?? ?? ?? 48 B8 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 81 ?? ?? ?? ?? C7 81 ?? ?? ?? ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 39 79 ?? 0F 84 ?? ?? ?? ?? 48 89 FA E8"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockTessellator.tessellateInWorld" {
  kind      = "function"
  msvc_name = "?tessellateInWorld@BlockTessellator@@QEAA_NAEAVTessellator@@AEBVBlock@@AEBVBlockPos@@_N@Z"
  notes     = "bool BlockTessellator::tessellateInWorld(Tessellator&, Block const&, BlockPos const&, bool useCalcWithCache): rcx = this, rdx = tessellator, r8 = block, r9 = pos, [rsp+20h] at the call = the bool. It calls 54 of the per-shape tessellate functions next to it and itself; the item renderers call it and then Tessellator.end. Recipe 0 is the body head (`mov rdi,r9; mov r14,r8; mov rbx,rdx; mov rsi,rcx; movzx eax,byte ptr [rcx+..]; xorps xmm6,xmm6; cmp al,..; jne`), then function_start_of; it ends at `cmp al,imm8`, before the jne (near on 26.51, short on 26.60). Recipe 1 is the call in a function that tessellates over a region (`mov rcx,[r15+..]; mov rdx,[r15+..]; mov byte ptr [rsp+20h],1; mov r9,rdi; call`). Check function_start. Recipe 1 is now a pattern_any from `mov byte ptr [rsp+20h],1; mov r9,r64; call` on to the three RIP loads after the call, one alternative per shape: alternative 1 has the nop that 26.60 adds after the call. Both have the call at offset 8. What changed in 6ad79ee4-17d51891a1119914 (26.60): the jne after the head became short, and the region function keeps its object in r12 (an extra SIB byte) and has a nop after the call. The new function was accepted on two facts from the 26.51 function: the same body head and table walk (`movzx eax,byte ptr [rcx+78h]; ..; cmp al,1`, then the 18h-step walk to 138h), and the same callers (7 on each build, including the call to itself and the call with the bool 1 in the region function that calls BlockTessellator.setRegion). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "4C 89 CF 4D 89 C6 48 89 D3 48 89 CE 0F B6 41 ?? 0F 57 F6 3C ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "C6 44 24 ?? ?? 49 89 ?? E8 ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 15",
        "C6 44 24 ?? ?? 49 89 ?? E8 ?? ?? ?? ?? 90 48 8B 05 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 15",
      ]
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockTessellator.mBlockCache" {
  kind     = "field"
  notes    = "Offset of BlockTessellatorCache BlockTessellator::mBlockCache: 6B0h on 26.51 (header 1712), the anchor of BlockTessellatorLayout. Recipe 0 is the constructor memset of the cache (`lea rcx,[r15+6B0h]; mov qword ptr [r15+..],0; lea rax,[r15+..]; mov [rbp+..],rax; mov r8d,426A0h; xor edx,edx; call`), the lea displacement. Recipe 1 is BlockTessellator.setRegion, which stores the region into the cache mRegion (cache+18h, header): `mov [rcx+8],rdx; mov [rcx+6C8h],rdx`, the second displacement minus 18h. Range window 6A0h..6C0h."

  recipe {
    step "pattern" { bytes = "49 8D 8F ?? ?? ?? ?? 49 C7 87 ?? ?? ?? ?? ?? ?? ?? ?? 49 8D 87 ?? ?? ?? ?? 48 89 45 ?? 41 B8 ?? ?? ?? ?? 31 D2 E8" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "BlockTessellator.setRegion" }
    step "pattern" {
      bytes  = "48 89 51 ?? 48 89 91 ?? ?? ?? ??"
      offset = 4
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = -24 }
  }

  check "range" {
    min = 1696
    max = 1728
  }
}
