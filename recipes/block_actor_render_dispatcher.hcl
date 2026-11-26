entry "BlockActorRenderDispatcher.render" {
  kind  = "function"
  notes = "void BlockActorRenderDispatcher::render(BaseActorRenderContext&, BlockSource& renderSource, IVanillaRenderBlockActorComponent&, Block const&, Vec3 const& renderPos, BlockPos const& worldPos, bool renderAlphaLayer, mce::MaterialPtr const& forcedMat, mce::ClientTexture const* forceTex, int breakingAmount, std::optional<dragon::RenderMetadata>), the form with renderPos: rcx = the dispatcher, rdx = the context, r8 = the BlockSource, r9 = the component, then on the stack at the call [rsp+20h] Block*, [rsp+28h] &renderPos, [rsp+30h] &worldPos, [rsp+38h] the bool, [rsp+40h] &forcedMat, [rsp+48h] forceTex, [rsp+50h] breakingAmount, [rsp+58h] a pointer to the optional (passed by address). It returns void. The body calls component slot 4 (`[rax+20h]`, getRendererId) and indexes the renderer table at this+0 (`mov rdi,[rdi+rax*8]`); when the optional is empty it builds the metadata from the Block. The first form (0x6a5e650 on 26.63) takes the camera target from the context (`mov r13,[rdx+0A8h]`, then +20h), calls component slot 6 (`[rax+30h]`) as a visibility test, calls component slot 2 (`[rax+10h]`, the position) twice, computes renderPos = Vec3(position) - camera target, so renderPos is camera relative, and calls this form. Callers of this form on 26.63: the first form, MovingBlockActorRenderer (0x6a83a10), and two item or GUI helpers (0x4c8cbb0, 0x6aef1d0). Recipe 0 is the head after the prologue (`mov rbx,r9; mov r15,r8; mov rsi,rdx; mov rdi,rcx; mov rax,[r9]; mov rax,[rax+disp8]; mov rcx,r9; call [guard]`), then function_start_of. Recipe 1 is the call in the first form (`lea rax,[rbp+..]; mov [rsp+28h],rax; mov [rsp+20h],r10; mov rcx,r14; mov rdx,rbx; mov r8,rdi; mov r9,rsi; call`), read as a branch target. The two recipes are in two functions. Every displacement and immediate is a wildcard. Value 0x6a5dcf0 on 26.63 (6aed1906-b127418e74d2f2f9)."

  recipe {
    step "pattern" {
      bytes = "4C 89 CB 4D 89 C7 48 89 D6 48 89 CF 49 8B 01 48 8B 40 ?? 4C 89 C9 FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 45 ?? 48 89 44 24 ?? 4C 89 54 24 ?? 4C 89 F1 48 89 DA 49 89 F8 49 89 F1 E8"
      offset = 26
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "RenderMetadata.engaged" {
  kind  = "field"
  notes = "Offset of the engaged flag (a bool) in std::optional<dragon::RenderMetadata>, the last argument of BlockActorRenderDispatcher.render, passed by address: 38h on 26.63 (6aed1906-b127418e74d2f2f9), 30h on 70.26 (6afa4e47-dadc375797a3d31a). The RenderMetadata value starts at offset 0 of the optional and ends at the flag: 26.63 copies 10h bytes plus a byte at 10h before the variant (index byte 30h); 70.26 copies 8 bytes plus a byte at 8 (index byte 28h), so the value is 8 bytes shorter and the optional is 40h bytes on 26.63 and 38h on 70.26. Recipe 0 is BlockActorRenderDispatcher.render itself: the test of the flag through the optional pointer (loaded from the stack argument), then the copy of the first value byte. It is a pattern_any, one alternative per shape: alternative 0 (26.63) `cmp byte ptr [r14+38h],1; jne; movzx eax,byte ptr [r14+10h]; mov [rbp+..],al`; alternative 1 (70.26) `cmp byte ptr [rbx+30h],1; jne; movzx eax,byte ptr [rbx+8]; mov [rbp+..],al`. It reads the displacement of the compare. Recipe 1 is the form of render that takes the camera target (it calls BlockActorRenderDispatcher.render; 0x6a5e650 on 26.63, 0x8a38480 on 70.26): it finds that function by the call site of the render recipe 1 (`lea rax,[rbp+..]; mov [rsp+28h],rax; mov [rsp+20h],r10; mov rcx,r14; mov rdx,rbx; mov r8,rdi; mov r9,rsi; call`), then the destruction of its own optional argument after the call: `mov rdx,[rbp+..]; cmp byte ptr [rdx+38h],1; jne; movzx eax,byte ptr [rdx+30h]; cmp rax,3; ja` (the flag, then the variant index of the four variant types); it reads the displacement of the compare. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard, and the compare value 1 too. Range window 20h..50h. Not derived on builds before 26.63. No build gate: the offset moves with the size of RenderMetadata; each build derives its own value."

  recipe {
    step "ref" { entry = "BlockActorRenderDispatcher.render" }
    step "pattern_any" {
      patterns = [
        "41 80 7E ?? ?? 75 ?? 41 0F B6 46 ?? 88 85",
        "80 7B ?? ?? 75 ?? 0F B6 43 ?? 88 85",
      ]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" { bytes = "48 8D 45 ?? 48 89 44 24 ?? 4C 89 54 24 ?? 4C 89 F1 48 89 DA 49 89 F8 49 89 F1 E8" }
    step "pattern" {
      bytes  = "48 8B 95 ?? ?? ?? ?? 80 7A ?? ?? 0F 85 ?? ?? ?? ?? 0F B6 42 ?? 48 83 F8 ?? 77"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 80
  }
}

entry "BlockActorRenderDispatcher.mSkullRenderer" {
  kind  = "field"
  notes = "Offset of the skull renderer (SkullBlockRenderer*) in BlockActorRenderDispatcher: 20h on 26.63 (6aed1906-b127418e74d2f2f9) and 70.26 (6afa4e47-dadc375797a3d31a). The dispatcher holds its renderers in a table at this+0, indexed by the renderer id of the block actor component (BlockActorRenderDispatcher.render: `call [component slot 4]; movzx eax,al; mov rdi,[rdi+rax*8]`); the skull id is 4. The item and worn-head paths take the dispatcher from the render context (BaseActorRenderContext+30h) and load this member right before the skull draw, after the null test of the item block type. Recipe 0 is the item path call of the skull draw (the bytes of SkullBlockRenderer.renderSkull recipe 1 on 26.63 and SkullBlockRenderer.renderSkullUpright recipe 0 on 70.26): a pattern_any, alternative 0 (26.63) `mov rax,[rax+..]; test rax,rax; je; mov rcx,[r12+20h]; test rcx,rcx; je; add rax,imm32; ...; call renderSkull`, alternative 1 (70.26) the same with `mov rcx,[r13+20h]` and `call renderSkullUpright`. It reads the displacement of the renderer load (offset 12 in both). Recipe 1 is the worn-head path (0x531b1f0 on 26.63, 0x6bad2e0 on 70.26): a pattern_any, alternative 0 (26.63) `mov rcx,[rdi+30h]; mov rcx,[rcx+20h]; test rcx,rcx; je; add rdx,imm32; mov [rsp+..],al; xorps xmm0,xmm0; movups [rsp+..],xmm0; mov [rsp+..],rdx; mov dword ptr [rsp+..],..`, alternative 1 (70.26) `mov rcx,[rcx+30h]; mov rcx,[rcx+20h]; test rcx,rcx; je; mov rdx,[rbp+..]; mov rdx,[rdx+..]; movss xmm0,[rdx]; add rax,..`; it reads the displacement of the second load. Each alternative matches once on its own build and 0 times on the other. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 8..60h. Not derived on builds before 26.63. No build gate: the same member on both builds."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4C 24 ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 7C 24 ?? 0F 57 C0 0F 11 44 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8",
        "48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4D ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 64 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8",
      ]
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 4F ?? 48 8B 49 ?? 48 85 C9 74 ?? 48 81 C2 ?? ?? ?? ?? 88 44 24 ?? 0F 57 C0 0F 11 44 24 ?? 48 89 54 24 ?? C7 44 24",
        "48 8B 49 ?? 48 8B 49 ?? 48 85 C9 74 ?? 48 8B 95 ?? ?? ?? ?? 48 8B 92 ?? ?? ?? ?? F3 0F 10 02 48 05",
      ]
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 96
  }
}
