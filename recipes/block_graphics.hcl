entry "BlockGraphics.getForBlock" {
  kind      = "function"
  msvc_name = "?getForBlock@BlockGraphics@@SAPEBV1@AEBVBlock@@@Z"
  notes     = "static BlockGraphics const* BlockGraphics::getForBlock(Block const&): rcx = block, the result in rax (may be null; 151 callers). It asks a per-block cache at block+118h first, then hashes the block state bytes (114h..117h, FNV) into the graphics map. The block tessellator cache fills its mBlockGraphics array (cache+232A0h, header 144032) from it. Recipe 0 is the map path (`cmp byte ptr [rsi+..],..; jne; movzx eax,byte ptr [rsi+..]; mov rcx,FNV basis; xor rcx,rax; mov rax,FNV prime`), then function_start_of. Recipe 1 is the cache fill call (`add rdi,rcx; add rdi,imm; mov rcx,rsi; call`). Check function_start."

  recipe {
    step "pattern" { bytes = "80 BE ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 0F B6 86 ?? ?? ?? ?? 48 B9 ?? ?? ?? ?? ?? ?? ?? ?? 48 31 C1 48 B8 ?? ?? ?? ?? ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 01 CF 48 81 C7 ?? ?? ?? ?? 48 89 F1 E8"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "BlockGraphics.mRenderLayer" {
  kind     = "field"
  notes    = "Offset of BlockRenderLayer BlockGraphics::mRenderLayer: 1Ah on 26.51 (header: vptr, mBlock 8, mBlockShape 10h, mAmbientOcclusionExponent 14h, mFancy 18h, mAllowSame 19h, mRenderLayer 1Ah, mIsotropicFaceData 1Bh), the anchor of the graphics read. No game code that reads the byte itself was found (the game asks the virtual getRenderLayer), so both recipes read its neighbours on the BlockGraphics.getForBlock result: recipe 0 is `movaps xmm0,xmm8; test byte ptr [rax+1Bh],1; jne` (the mIsotropicFaceData bit of the first face; the five other faces test 2..20h, so the bit is kept) minus 1, recipe 1 is `movzx r9d,bp; cmp byte ptr [rax+19h],0; je` (mAllowSame) plus 1, after two hash multiplies. Range window 10h..28h."

  recipe {
    step "pattern" {
      bytes      = "41 0F 28 C0 F6 40 ?? 01 75"
      literal_ok = ["7: the first face bit 1: the five other faces test 2, 4, 8, 10h and 20h"]
      offset     = 4
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = -1 }
  }

  recipe {
    step "pattern" {
      bytes  = "44 69 27 ?? ?? ?? ?? 69 4F ?? ?? ?? ?? ?? 44 0F B7 CD 80 78 ?? ?? 0F 84"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = 1 }
  }

  check "range" {
    min = 16
    max = 40
  }
}
