entry "RenderChunkCoordinator._setDirty" {
  kind   = "function"
  hooked = false
  notes  = "PoC (dynamic-lighting). void RenderChunkCoordinator::_setDirty(BlockPos const& min, BlockPos const& max, bool immediate, bool changesVisibility, bool canInterlock): rcx = the coordinator (node+18h of the map at LevelRenderer+30h; vtable 0xf328760 on 1.26.62.01), rdx = min, r8 = max (block coords, inclusive, >>4 to subchunks), r9b = immediate, [rsp+20h] = changesVisibility, [rsp+28h] = canInterlock. Adds each subchunk in the box to the dirty map at coordinator+88h; the per-frame LevelRenderer update rebuilds them (at once within 24 blocks of the camera when immediate). Recipe 0 is the body head (saves, stack cookie, `mov r9d,[rdx]; mov r15d,[rdx+4]; sar r9d; sar r15d`). Recipe 1 is the call in the coordinator's onBlockChanged (`setne [rsp+..]; mov byte [rsp+..],1; lea rdx; lea r8; mov rcx,rdi; mov r9b,1; call`), read as a branch target. 1.26.62.01: 0x7921200. See docs/findings/terrain-dirty-path/report.md."

  recipe {
    step "pattern" { bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 81 EC ?? ?? ?? ?? 44 88 4C 24 ?? 48 89 4C 24 ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 84 24 ?? ?? ?? ?? 44 8B 0A 44 8B 7A ?? 41 C1 F9 ?? 41 C1 FF ?? 8B 42 ??" }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 95 44 24 ?? C6 44 24 ?? 01 48 8D 54 24 ?? 4C 8D 44 24 ?? 48 89 F9 41 B1 01 E8"
      offset = 26
      literal_ok = ["9: canInterlock = true in `mov byte ptr [rsp+28h],1`: the block-change listener always passes it", "25: immediate = true in `mov r9b,1`: the block-change listener always passes it"]
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
