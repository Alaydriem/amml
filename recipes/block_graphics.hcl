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

entry "BlockGraphics.mBlockShape" {
  kind  = "field"
  notes = "Offset of BlockShape BlockGraphics::mBlockShape (a 4-byte enum): 10h on 26.63 (6aed1906-b127418e74d2f2f9) and 70.26 (6afa4e47-dadc375797a3d31a); 26.51 header order: vptr, mBlock 8, mBlockShape 10h. Recipe 0 is BlockTessellator::tessellateInWorld right after its BlockGraphics.getForBlock call: `test rax,rax; je; mov ecx,[rax+10h]; xor ebp,ebp; mov [rsp+..],ecx; mov [rsi+..],r15b`; the value it loads is the switch index of the shape jump table that follows (`cmp rcx,9Eh; ja; lea rdx,[rip+..]; movsxd rcx,[rdx+rcx*4]`, 159 shapes on both builds). It starts after the call and reads the load displacement. Recipe 1 is another getForBlock caller (0x2544780 on 26.63, 0x372ec40 on 70.26) right after the call: `test rax,rax; je; mov eax,[rax+10h]; cmp eax,-1; je; cmp eax,6Ah; je; mov rax,[r14+..]; movss xmm8,[rax+..]` (it skips the Invisible shape -1 and shape 6Ah); it reads the load displacement. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 8..30h. Not derived on builds before 26.63. No build gate: the same member on both builds."

  recipe {
    step "ref" { entry = "BlockTessellator.tessellateInWorld" }
    step "pattern" {
      bytes  = "48 85 C0 74 ?? 8B 48 ?? 31 ED 89 4C 24 ?? 44 88 BE ?? ?? ?? ??"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 85 C0 0F 84 ?? ?? ?? ?? 8B 40 ?? 83 F8 ?? 0F 84 ?? ?? ?? ?? 83 F8 ?? 0F 84 ?? ?? ?? ?? 49 8B 86 ?? ?? ?? ?? F3 44 0F 10 40 ??"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 48
  }
}
