entry "SkullBlockRenderer.renderSkull" {
  kind   = "function"
  hooked = true
  notes  = "void SkullBlockRenderer::renderSkull(SkullBlockRenderer* this, BaseActorRenderContext& ctx, dragon::RenderMetadata const& metadata, Vec3 const& pos, int face, float rotationDegrees, HashedString const& skullName, mce::MaterialPtr const* forcedMaterial, mce::ClientTexture const* forcedTexture, int breakingAmount, float animationValue, bool isGlint): rcx = the renderer, rdx = the context, r8 = the metadata, r9 = the position; the eight stack arguments are at [rsp+20h] (face, int), [rsp+28h] (rotation, float, in degrees), [rsp+30h] (the name, a HashedString pointer: the BlockType name, BlockType.mNameInfo.mFullName), [rsp+38h] (forced material, may be null), [rsp+40h] (forced texture, may be null), [rsp+48h] (breaking amount; neither caller below writes this slot right before the call, and the decompiled body does not read it), [rsp+50h] (animation, float) and [rsp+58h] (glint, byte) at the call. It compares the name with the 7 skull block names and returns before any draw when none matches. Face 1 is the floor (the rotation argument is used); faces 2..5 are the walls (0 is the fallback wall). How it was found: SkullBlockRenderer::render (0x6a90c50 on 26.63; it gets the block actor from the render data, tests isType(6), reads the world block at the actor position (BlockActor+8) and its facing_direction state) ends with one direct call to this function; four functions call it (0x6a90c50, 0x598e6f0, 0x5979ac0, 0x531b1f0); the three that are not the block renderer are item and worn-head paths that pass rotation 180.0 and null material and texture. Recipe 0 is the call in SkullBlockRenderer::render: `mov eax,imm32` (the name member offset) `; mov rcx,[rbp+..]; add rax,[rcx+disp8]` (the BlockType of the Block) `; movss [rsp+..],xmm6; mov [rsp+..],r14; mov rcx,[rbp+..]; mov [rsp+..],rcx; mov [rsp+..],rax; movss [rsp+..],xmm7; mov [rsp+..],ebx; mov byte ptr [rsp+..],imm8; lea r8,[rbp+..]; mov rcx,[rbp+..]; mov rdx,rsi; mov r9,[rbp+..]; call`, offset 68, read as a branch target. Recipe 1 is the call in the item path 0x598e6f0: `mov rax,[rax+disp32]` (Item+178h, the block type of the item) `; test rax,rax; je; mov rcx,[r12+disp8]` (the skull renderer, dispatcher renderer 4) `; test rcx,rcx; je; add rax,imm32` (the name member offset) `; mov [rsp+..],r15b; xorps xmm0,xmm0; movups [rsp+..],xmm0; mov [rsp+..],rax; mov dword ptr [rsp+..],imm32` (three times: animation 0, rotation 180.0, face 1) `; lea r9,[rip+..]; mov rdx,rsi; mov r8,[rbp+..]; call`, offset 84, read as a branch target. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Value 0x6a5ea50 on 6aed1906-b127418e74d2f2f9 (26.63). Class: new entry."

  recipe {
    step "pattern" {
      bytes  = "B8 ?? ?? ?? ?? 48 8B 4D ?? 48 03 41 ?? F3 0F 11 74 24 ?? 4C 89 74 24 ?? 48 8B 4D ?? 48 89 4C 24 ?? 48 89 44 24 ?? F3 0F 11 7C 24 ?? 89 5C 24 ?? C6 44 24 ?? ?? 4C 8D 45 ?? 48 8B 4D ?? 48 89 F2 4C 8B 4D ?? E8"
      offset = 68
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4C 24 ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 7C 24 ?? 0F 57 C0 0F 11 44 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8"
      offset = 84
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
