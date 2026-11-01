entry "LevelChunk.mSubChunks" {
  kind     = "field"
  notes    = "Offset of `mSubChunks` (std::vector<SubChunk>: begin, end, capacity) in LevelChunk: 0x140 in 26.51 (LeviLamina header sum: mRandomTickQueue 8 at 0x138, mSubChunks at 0x140). It is the one LevelChunk member that moves between the two builds, and `amml-game` reads it to select the LevelChunk mirror: 0x140 selects the 26.51 mirror, 0x1C8 the 26.60 mirror (all members after mSubChunks, the heightmaps among them, are 88h later on 26.60); any other value refuses. The members before it (mDimension 58h, mMin 60h, mMax 6Ch, mPosition 78h, mLoadState F0h) are at the same place on both builds and are not entries. Recipe 0 is LevelChunk::getBlock (LevelChunk.getBlock): `mov r9,[rcx+disp]; mov rcx,[rcx+disp+8]; sub; sar 3; imul by the inverse of 13` (the size check, end minus begin over 68h), then `imul rcx,r8,68h; mov rcx,[r9+rcx+30h]`. Recipe 1 is a client function that checks one sub-chunk: `mov rcx,[rax+disp]; imul rax,rdx,68h; movsx edx,byte ptr [rcx+rax+61h]` (SubChunk::mAbsoluteIndex), then calls SubChunk.isPlaceHolderSubChunk on it. Recipe 2 is a LevelChunk set-block function: `mov r12,[r14+disp]; mov rdx,[r14+disp+8]; ...; imul rax,rcx,68h; add r12,rax`, then a SubChunk call with the sub-chunk in rcx. All three read the displacement of the begin load. They are in three functions. Every displacement, immediate and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: recipe 1 is now a pattern_any with one alternative per register shape (see SubChunk.size), ending at the isPlaceHolderSubChunk call. 140h on 26.51, 1C8h on 26.60 (LevelChunk grew before the subchunk vector). Recipe 0 matches in LevelChunk::getBlock, which is now a leaf without .pdata; the match starts at the derived value of LevelChunk.getBlock, so the lint decodes it from there. Evidence on 26.60: recipes 1 and 2 are in two functions found from the old functions and read 1C8h. Class: real code change."

  recipe {
    step "pattern" {
      bytes  = "0F BF 42 ?? 41 89 C0 41 C1 E8 ?? 4D 0F BF C0 4C 8B 89 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 4C 29 C9 48 C1 F9 ?? 49 BA ?? ?? ?? ?? ?? ?? ?? ?? 4C 0F AF D1 4D 39 C2 76 ?? 49 6B C8 ?? 49 8B 4C 09 ?? 48 85 C9 74 ?? 44 0F B6 02"
      offset = 15
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = ["48 8B 88 ?? ?? ?? ?? 48 6B C2 ?? 0F BE 54 01 ?? 66 41 39 D5 75 ?? 48 01 C1 E8", "48 8B 88 ?? ?? ?? ?? 49 6B C0 ?? 44 0F BE 44 01 ?? 66 44 39 C2 75 ?? 48 01 C1 E8"]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 8B A6 ?? ?? ?? ?? 49 8B 96 ?? ?? ?? ?? 4C 29 E2 48 C1 FA ?? 49 0F AF D5 48 39 CA 0F 86 ?? ?? ?? ?? 0F B6 13 C1 E2 ?? 83 E0 ?? 09 D0 44 0F B6 43 ?? 41 C1 E0 ?? 41 01 C0 48 6B C1 ?? 49 01 C4"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 1024
  }
}

entry "LevelChunk.getBlock" {
  kind      = "function"
  msvc_name = "?getBlock@LevelChunk@@QEBAAEBVBlock@@AEBVChunkBlockPos@@@Z"
  notes     = "Block const& LevelChunk::getBlock(ChunkBlockPos const& pos) const, not virtual. ChunkBlockPos is 4 bytes: x (u8) at 0, z (u8) at 1, y (i16) at 2, with y relative to the chunk minimum. The body takes the sub-chunk index y >> 4, checks it against the size of mSubChunks (end minus begin over 68h), loads mBlocksReadPtr[0] of that sub-chunk (`imul rcx,r8,68h; mov rcx,[r9+rcx+30h]`), builds the block index (x << 8) | (z << 4) | (y & 15) and tail-jumps through slot 3 of the block storage; out of range or with no storage it returns the air block from a global. On 26.51 it has a frame (`sub rsp,28h`) and .pdata; on 26.60 (6ad79ee4-17d51891a1119914) it is a leaf without a frame and without .pdata (the assert call on the air path is gone, so the frame went with it). Recipe 0 is a pattern_any on the body from its first byte, one alternative per shape, offset 0: alternative 0 is the framed shape (`sub rsp,imm8`, then the y load, the index and the size check, the imul, the storage load and the x load); alternative 1 is the leaf shape (the same body from the y load, through the index build and `mov rax,[rcx]; mov rax,[rax+disp]; mov r8,[rip+..]; jmp r8`). Alternative 1 cannot match on 26.51 (there `add rsp,28h` sits before the jmp) and alternative 0 cannot match on 26.60 (no `sub rsp`). Recipe 1 is the only direct call, in a BlockSource set-block function: it builds a ChunkBlockPos on the stack from a BlockPos (`x & 15`, `z & 15`, `y - min`: `mov [rsp+..],dl; mov [rsp+..],al; mov [rsp+..],cx; lea rdx,[rsp+..]; mov rcx,<chunk>; call`) and passes the result on (`mov rcx,rax; mov rdx,<pos>; mov r8,rdi; mov r9,rdi; call`); it is read as a branch target. It is a pattern_any, one alternative per shape, both with the call at index 23: on 26.51 the chunk comes from r14 (`mov rcx,r14`, 3 bytes; the alternative starts 2 bytes earlier, inside `and dl,0Fh`), on 26.60 it is reloaded from the stack (`mov rcx,[rsp+..]`, 5 bytes). The source register of the position (r12 / r13) and of the chunk are wildcarded. Every displacement, immediate and branch byte is a wildcard. The check is leaf_start: 16-byte aligned, int3 padding (or a ret / jmp at the end of a .pdata range) before it, not inside the .pdata range of another function, and the target of at least one direct call. It passes on 26.51 (a .pdata start, after int3 padding, with its one caller) and on 26.60 (a leaf after int3 padding, with its one caller). Recipe 0 and recipe 1 anchor on different code (the body against its caller) in two functions. What changed in 6ad79ee4-17d51891a1119914 (26.60): the function became a frameless leaf without .pdata, so the old recipe 0 (body plus function_start_of) and the old check (function_start) could not pass; the caller reloads the chunk from the stack instead of r14. Class: real code change. Evidence on 26.60: (1) the old body, drafted from the 26.51 function, matches once on 26.60, at the leaf start; (2) the only call site of the 26.51 function builds the ChunkBlockPos with the same three stores and passes the result to the same 4-argument call; on 26.60 the same sequence, found from the old call site, calls the leaf, and the leaf is the only target of that call."

  recipe {
    step "pattern_any" {
      patterns = ["48 83 EC ?? 0F BF 42 ?? 41 89 C0 41 C1 E8 ?? 4D 0F BF C0 4C 8B 89 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 4C 29 C9 48 C1 F9 ?? 49 BA ?? ?? ?? ?? ?? ?? ?? ?? 4C 0F AF D1 4D 39 C2 76 ?? 49 6B C8 ?? 49 8B 4C 09 ?? 48 85 C9 74 ?? 44 0F B6 02", "0F BF 42 ?? 41 89 C0 41 C1 E8 ?? 4D 0F BF C0 4C 8B 89 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 4C 29 C9 48 C1 F9 ?? 49 BA ?? ?? ?? ?? ?? ?? ?? ?? 4C 0F AF D1 4D 39 C2 76 ?? 49 6B C8 ?? 49 8B 4C 09 ?? 48 85 C9 74 ?? 44 0F B6 02 41 C1 E0 ?? 83 E0 ?? 44 09 C0 0F B6 52 ?? C1 E2 ?? 01 C2 48 8B 01 48 8B 40 ?? 4C 8B 05 ?? ?? ?? ?? 49 FF E0"]
      offset   = 0
    }
  }

  recipe {
    step "pattern_any" {
      patterns = ["E2 ?? 88 54 24 ?? 88 44 24 ?? 66 89 4C 24 ?? 48 8D 54 24 ?? 4C 89 ?? E8 ?? ?? ?? ?? 48 89 C1 4C 89 ?? 49 89 F8 49 89 F9 E8", "88 54 24 ?? 88 44 24 ?? 66 89 4C 24 ?? 48 8D 54 24 ?? 48 8B 4C 24 ?? E8 ?? ?? ?? ?? 48 89 C1 4C 89 ?? 49 89 F8 49 89 F9 E8"]
      offset   = 23
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}

entry "LevelChunk.getBiome" {
  kind     = "function"
  notes    = "Biome const& LevelChunk::getBiome(ChunkBlockPos const& pos) const: 0x32d9300 in 26.51. Signature: rcx = chunk, rdx = pointer to the 4-byte ChunkBlockPos {u8 x, u8 z, i16 y}, the result in rax. The body tests mUse3DBiomeMaps (`cmp byte ptr [rcx+10D3h],1`); the 3D path checks y >> 4 against m3dBiomeStackSize (+378h) and calls slot 3 of the sub-chunk biome storage at mBiomes.mBiomeSubchunks (+358h) with (x << 8) | (z << 4) | (y & 15); the 2D path reads m2DBiomes (+158h) and looks the id up through Level.getBiomeRegistry. Recipe 0 is the start of the body, then function_start_of. Recipe 1 is its call in BlockSource::getBiome(BlockPos const&), which builds the ChunkBlockPos on the stack (`mov [rsp+24h],r8b; mov [rsp+25h],r9b; mov [rsp+26h],cx; lea rdx,[rsp+24h]; mov rcx,rax; call; jmp`), read as a branch target. It has 20 direct calls. The recipes share no anchor bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Grade: two recipes only."

  recipe {
    step "pattern" {
      bytes = "48 89 D7 48 89 CE 80 B9 ?? ?? ?? ?? ?? 75 ?? 0F B7 86 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 44 0F BF 47 ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "44 88 44 24 ?? 44 88 4C 24 ?? 66 89 4C 24 ?? 48 8D 54 24 ?? 48 89 C1 E8 ?? ?? ?? ?? E9"
      offset = 23
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
