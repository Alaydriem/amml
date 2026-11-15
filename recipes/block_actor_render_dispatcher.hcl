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
