entry "SubChunk.size" {
  kind     = "field"
  notes    = "sizeof(SubChunk), the stride of LevelChunk::mSubChunks: 0x68 in 26.51. The LeviLamina header gives no size; the field sum of SubChunk.h (mDirtyTicksCounter 8 at 0 up to mIsInitialized at 0x64) ends at 0x65, rounded up to the alignment 8 gives 0x68. The value is the immediate of the index-scale `imul reg,reg,68h` in code that indexes mSubChunks; the same functions divide end minus begin by it (`sar 3`, then a multiply by 4EC4EC4EC4EC4EC5h, the inverse of 13). Recipe 0 is LevelChunk::getBlock (`imul rcx,r8,68h; mov rcx,[r9+rcx+30h]`). Recipe 1 is the client sub-chunk check of LevelChunk.mSubChunks recipe 1 (`imul rax,rdx,68h; movsx edx,byte ptr [rcx+rax+61h]`). Recipe 2 is the LevelChunk set-block function of LevelChunk.mSubChunks recipe 2 (`imul rax,rcx,68h; add r12,rax`). All three read the immediate operand. They are in three functions. Every displacement, immediate and branch byte of the patterns is a wildcard (the read immediate too). What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the subchunk loop of the LevelChunk function was compiled with other registers (`imul rax,rdx,68h; movsx edx,byte ptr [rcx+rax+..]` became `imul rax,r8,68h; movsx r8d,byte ptr [rcx+rax+..]`). Recipe 1 is now a pattern_any with one alternative per shape, both ending at the isPlaceHolderSubChunk call. Recipe 0 still matches; on 26.60 its match is in LevelChunk::getBlock, which is now a leaf without .pdata, and starts at the derived value of LevelChunk.getBlock, so the lint decodes it from there. 68h on both: the SubChunk size did not change. Class: real code change (register allocation). Recipes 0 and 2 are now a pattern_any with one alternative for each shape; the new alternative starts later (4 and 10 tokens after the old start) so that the immediate of the imul sits at the same offset for both alternatives (55 and 57), and the old alternative still starts at the function start (the lint decodes a match that starts there). Recipe 1 has a third alternative for the 70.26 form of the same client sub-chunk check (`jne rel32` instead of `jne short`). What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the size is still 68h: LevelChunk::getBlock, the client sub-chunk check and the set-block function index mSubChunks (now at 228h) with `imul reg,reg,68h` and divide the vector size with the inverse of 13 (4EC4EC4EC4EC4EC5h, 68h / 8). The registers and the near jumps changed. Evidence from the old function: (1) the three functions are the same functions that LevelChunk.mSubChunks recipes 0, 1 and 2 find, matched by their old bodies with the new register and jump forms; each new alternative matches once on 70.26 (getBlock 0x4528300, the sub-chunk check in the caller of SubChunk.isPlaceHolderSubChunk, the set-block function); (2) in getBlock the vector size is divided by 13 with the multiply constant 4EC4EC4EC4EC4EC5h (0x68 / 8 = 13) and the same function indexes with imul 68h, so the stride is confirmed by an independent constant in the same body, and the three functions agree. The 70.26 patterns of recipes 0 and 2 are the ones of LevelChunk.mSubChunks. Class: would have survived with a better recipe (register allocation and jump encoding)."

  recipe {
    step "pattern_any" {
      patterns = [
        "0F BF 42 ?? 41 89 C0 41 C1 E8 ?? 4D 0F BF C0 4C 8B 89 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 4C 29 C9 48 C1 F9 ?? 49 BA ?? ?? ?? ?? ?? ?? ?? ?? 4C 0F AF D1 4D 39 C2 76 ?? 49 6B C8 ?? 49 8B 4C 09 ?? 48 85 C9 74 ?? 44 0F B6 02",
        "41 89 C0 41 C1 E8 ?? 4D 0F BF C0 48 8B B1 ?? ?? ?? ?? 48 8B 89 ?? ?? ?? ?? 48 29 F1 48 C1 F9 ?? 49 B9 ?? ?? ?? ?? ?? ?? ?? ?? 4C 0F AF C9 4D 39 C1 0F 86 ?? ?? ?? ?? 4D 6B D0 ?? 4A 8B 4C 16 ?? 48 85 C9 0F 84 ?? ?? ?? ?? 44 0F B6 0A",
      ]
      offset = 55
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 88 ?? ?? ?? ?? 48 6B C2 ?? 0F BE 54 01 ?? 66 41 39 D5 75 ?? 48 01 C1 E8",
        "48 8B 88 ?? ?? ?? ?? 49 6B C0 ?? 44 0F BE 44 01 ?? 66 44 39 C2 75 ?? 48 01 C1 E8",
        "48 8B 88 ?? ?? ?? ?? 49 6B C0 ?? 44 0F BE 44 01 ?? 66 44 39 C2 0F 85 ?? ?? ?? ?? 48 01 C1 E8",
      ]
      offset = 7
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 8B A6 ?? ?? ?? ?? 49 8B 96 ?? ?? ?? ?? 4C 29 E2 48 C1 FA ?? 49 0F AF D5 48 39 CA 0F 86 ?? ?? ?? ?? 0F B6 13 C1 E2 ?? 83 E0 ?? 09 D0 44 0F B6 43 ?? 41 C1 E0 ?? 41 01 C0 48 6B C1 ?? 49 01 C4",
        "?? ?? ?? ?? 4D 29 E0 49 C1 F8 ?? 49 BA ?? ?? ?? ?? ?? ?? ?? ?? 4D 0F AF D0 4D 39 CA 0F 86 ?? ?? ?? ?? 0F B6 C9 C1 E1 ?? 83 E2 ?? 44 0F B6 C0 41 C1 E0 ?? 41 09 D0 41 01 C8 49 6B C1 ?? 49 01 C4",
      ]
      offset = 57
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  check "range" {
    min = 64
    max = 256
  }
}

entry "SubChunk.isPlaceHolderSubChunk" {
  kind      = "function"
  msvc_name = "?isPlaceHolderSubChunk@SubChunk@@QEBA_NXZ"
  notes     = "bool SubChunk::isPlaceHolderSubChunk() const, not virtual: true when the sub-chunk is still the client-request placeholder. The body gets the default block of the type minecraft:client_request_placeholder_block (`lea rcx,[rip+..]; lea rdx,[rip+..]` (the HashedString of that name); `mov sil,1; mov r8b,1; call`), asks mBlocksReadPtr[0] (`[rbx+30h]`) whether it is uniform with that block (slot 1 of the storage), and, when it is, asks the same of mBlocksReadPtr[1] (`[rbx+38h]`) by a tail jump, or returns true when that layer is null. Found through the HashedString of the placeholder name: of its 25 users, this is the small function next to SubChunk::_setBlock. Recipe 0 is the body after the register saves, then function_start_of. Recipe 1 is its only direct call, in a client function that indexes mSubChunks, checks mAbsoluteIndex (`imul rax,rdx,68h; movsx edx,byte ptr [rcx+rax+61h]; cmp r13w,dx; jne; add rcx,rax; call`) and inverts the result (`xor al,1`); it is read as a branch target. Every displacement, immediate and branch byte is a wildcard. The check is function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the body changed: the block lookup at the start is now a loop over a registry helper, and the old recipe 0 started before it; the call site was compiled with other registers. Recipe 0 is now the end of the body: `mov rdx,rdi; call [guard]; test al,al; je; mov rcx,[reg+..]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax+..]; mov r8,[rip+..]` (the tail call), with the base register byte as a wildcard, then function_start_of; it is unique on both builds. Recipe 1 is a pattern_any for the call site in the subchunk loop (`movsx edx,byte ptr [rcx+rax+..]; cmp r13w,dx; jne; add rcx,rax; call` on 26.51, `movsx r8d,...; cmp dx,r8w; jne; add rcx,rax; call` on 26.60; the 26.60 alternative starts after its REX byte so the call is at offset 14 in both). Evidence on 26.60: the function that holds the only call site is found from the old caller by a draft (one match), and the call at the same place in its subchunk loop goes to the candidate; the candidate ends like the old body (the palette virtual +8 on [this+30h] with the looked-up block, then the tail call through [this+38h]). Class: real code change."

  recipe {
    step "pattern" {
      bytes = "48 89 FA FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 40 ?? 4C 8B 05"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["0F BE 54 01 ?? 66 41 39 D5 75 ?? 48 01 C1 E8", "0F BE 44 01 ?? 66 44 39 C2 75 ?? 48 01 C1 E8"]
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
