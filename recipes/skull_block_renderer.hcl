entry "SkullBlockRenderer.renderSkull" {
  kind   = "function"
  hooked = true
  notes  = "void SkullBlockRenderer::renderSkull(SkullBlockRenderer* this, BaseActorRenderContext& ctx, dragon::RenderMetadata const& metadata, Vec3 const& pos, int face, float rotationDegrees, HashedString const& skullName, mce::MaterialPtr const* forcedMaterial, mce::ClientTexture const* forcedTexture, int breakingAmount, float animationValue, bool isGlint): rcx = the renderer, rdx = the context, r8 = the metadata, r9 = the position; the eight stack arguments are at [rsp+20h] (face, int), [rsp+28h] (rotation, float, in degrees), [rsp+30h] (the name, a HashedString pointer: the BlockType name, BlockType.mNameInfo.mFullName), [rsp+38h] (forced material, may be null), [rsp+40h] (forced texture, may be null), [rsp+48h] (breaking amount; neither caller below writes this slot right before the call, and the decompiled body does not read it), [rsp+50h] (animation, float) and [rsp+58h] (glint, byte) at the call. It compares the name with the 7 skull block names and returns before any draw when none matches. Face 1 is the floor (the rotation argument is used); faces 2..5 are the walls (0 is the fallback wall). How it was found: SkullBlockRenderer::render (0x6a90c50 on 26.63; it gets the block actor from the render data, tests isType(6), reads the world block at the actor position (BlockActor+8) and its facing_direction state) ends with one direct call to this function; four functions call it (0x6a90c50, 0x598e6f0, 0x5979ac0, 0x531b1f0); the three that are not the block renderer are item and worn-head paths that pass rotation 180.0 and null material and texture. Recipe 0 is the call in SkullBlockRenderer::render: `mov eax,imm32` (the name member offset) `; mov rcx,[rbp+..]; add rax,[rcx+disp8]` (the BlockType of the Block) `; movss [rsp+..],xmm6; mov [rsp+..],r14; mov rcx,[rbp+..]; mov [rsp+..],rcx; mov [rsp+..],rax; movss [rsp+..],xmm7; mov [rsp+..],ebx; mov byte ptr [rsp+..],imm8; lea r8,[rbp+..]; mov rcx,[rbp+..]; mov rdx,rsi; mov r9,[rbp+..]; call`, offset 68, read as a branch target. Recipe 1 is the call in the item path 0x598e6f0: `mov rax,[rax+disp32]` (Item+178h, the block type of the item) `; test rax,rax; je; mov rcx,[r12+disp8]` (the skull renderer, dispatcher renderer 4) `; test rcx,rcx; je; add rax,imm32` (the name member offset) `; mov [rsp+..],r15b; xorps xmm0,xmm0; movups [rsp+..],xmm0; mov [rsp+..],rax; mov dword ptr [rsp+..],imm32` (three times: animation 0, rotation 180.0, face 1) `; lea r9,[rip+..]; mov rdx,rsi; mov r8,[rbp+..]; call`, offset 84, read as a branch target. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Value 0x6a5ea50 on 6aed1906-b127418e74d2f2f9 (26.63). Class: new entry. Layout gate: 26.63 only; on 70.x the function is gone: its model choice and material choice are two helpers that take the name, and the face move, turn and draw are copied into each of its four former callers, so no function takes this argument list; the replacement is SkullBlockRenderer.render (placed skulls), SkullBlockRenderer.renderSkullUpright (a named skull at a position, the former item path), SkullBlockRenderer.getSkullModel and SkullBlockRenderer.getSkullMaterials (the two name helpers)."

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

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "SkullBlockRenderer.vtable" {
  kind  = "global"
  notes = "The vtable of SkullBlockRenderer (the BlockActorRenderer for skull block actors), on 26.63 and 70.x. It has no RTTI. The constructor is the only function that loads the text `ui_mob_head.skinning` (the skull model geometry name) on both builds; after the base constructor call it stores the vtable: `call; nop; lea rax,[rip+..]; mov [rdi],rax`. Recipe 0 finds the constructor by the text, then that store in it, and reads the lea target. The store order is the clue (the first vtable store after the base constructor call). Every displacement is a wildcard. The check is section .rdata; SkullBlockRenderer.render, found by its own body, checks in_vtable against this table, which catches a wrong table. Needed by SkullBlockRenderer.render. Evidence on 70.26: the text is loaded by one function on each build, with the same prologue and the same skull model hash store (91C049F736FAD420h) after the store; slot 0 of the table on both builds is the destructor that stores the same table and frees the member at +A18h first, and slot 2 is the render function (SkullBlockRenderer.render). No build gate: the same table on both builds."

  recipe {
    step "string_xref" { text = "ui_mob_head.skinning" }
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 90 48 8D 05 ?? ?? ?? ?? 48 89 07"
      offset = 6
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "SkullBlockRenderer.render" {
  kind   = "function"
  hooked = true
  notes  = "void SkullBlockRenderer::render(SkullBlockRenderer* this, BaseActorRenderContext& ctx, BlockActorRenderData& data): the BlockActorRenderer::render override (slot 2 of SkullBlockRenderer.vtable on 26.63 and 70.x). rcx = the renderer, rdx = the context, r8 = the render data ([data] the BlockSource, [data+8] the block actor, [data+18h] the position Vec3, [data+30h] the metadata). It tests the actor with isType(6) (Skull) and returns when it is not a skull; reads the world block at the actor position (BlockActor+8) from the BlockSource and its facing_direction state (the face: 1 floor, 2..5 walls); the turn is the SkullBlockActor rotation member (+0F8h on 26.63, +100h on 70.x) and the name is the BlockType full name of that world block. On 26.63 it then calls SkullBlockRenderer.renderSkull with face, turn and name; on 70.x it moves and turns the matrix stack top itself and calls SkullBlockRenderer.getSkullMaterials, SkullBlockRenderer.getSkullModel and the draw. Recipe 0 is the head of the body: `movss xmm6,[rdx+..]; mov rax,[rcx]; mov rax,[rax+disp8]; call [guard]` (the block actor) `; mov reg,rax; mov rax,[rax]; mov rax,[rax+disp32]; mov rcx,reg; mov dl,6; call [guard]` (isType), then function_start_of. The isType argument 6 is kept (the other block actor renderers test other types); every displacement and the two result-register bytes are wildcards. Checks: function_start and in_vtable SkullBlockRenderer.vtable slots 0..4 (slot 3 is not code on both builds, so the window stays inside this table). Evidence on 70.26: (1) the function is slot 2 of the vtable stored by the constructor that loads `ui_mob_head.skinning` on both builds (the 26.63 function is slot 2 of the 26.63 table); (2) the pattern of recipe 0, drafted from the 26.63 body, matches once on 70.26 in this function, and its 70.26 body keeps the rest of the 26.63 body order: the helper call with `(*(data+BCh) ^ 1) & *(data+BBh)`, the FNV-1a hash of the block position bytes with 9E3779B9h, the material variant switch and the shared_ptr cleanup. No build gate: the same function and signature on both builds. Hook signature: 3 register arguments on both builds. What changed in 6afa4e47-dadc375797a3d31a: the renderSkull call is inlined. Class: real code change (body)."

  recipe {
    step "pattern" {
      bytes      = "F3 0F 10 B2 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 49 89 ?? 48 8B 00 48 8B 80 ?? ?? ?? ?? 4C 89 ?? B2 06 FF 15"
      literal_ok = [
        "38: mov dl,6: isType(Skull); the other block actor renderers have the same head with other type ids",
      ]
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "SkullBlockRenderer.vtable"
    min_slot = 0
    max_slot = 4
  }
}

entry "SkullBlockRenderer.getSkullModel" {
  kind   = "function"
  hooked = true
  notes  = "SkullModel* SkullBlockRenderer::getSkullModel(SkullBlockRenderer const* this, HashedString const& name, float animation, bool glint) on 70.x (the name is ours: the function has no symbol). rcx = the renderer, rdx = the skull name (a BlockType full name, BlockType.mNameInfo.mFullName), xmm2 = the animation value, r9b = glint. It compares the name with the skull names (7 static HashedStrings) and returns null when none matches; else it takes the model member (this+A10h, A18h, 9D8h, 9E0h, 9E8h, 9F0h or 9F8h), writes the animation to model+140h for the two animated heads, copies the glint-indexed texture into the model and returns the model in rax. Recipe 0 is the head of the body after the pushes: `mov ebx,r9d; movaps xmm6,xmm2; mov r14,rdx; mov rdi,rcx; lea rdx,[rip+..]; mov rcx,r14; call` (the first name compare), then function_start_of. Recipe 1 is the call in SkullBlockRenderer::render: `cmp qword ptr [rbx],..; je; mov [rbp+..],rbx; mov rcx,rsi; mov rdx,r14; movaps xmm2,xmm6; xor r9d,r9d; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The 26.63 renderSkull (SkullBlockRenderer.renderSkull) chose the model by the skull name with the members this+A10h, A18h, 9D8h, 9E0h, 9E8h, 9F0h, 9F8h (with the +140h animation store and the glint-indexed texture copy) and the material set with this+238h, 378h, 5F8h, 738h, 4B8h, 878h and 9B8h (glint); on 70.x these are two separate functions. Evidence on 70.26: (1) the body has exactly the model part of the 26.63 renderSkull body (the same seven model members in the same name order, the +140h animation store and the +148h/+150h glint-indexed copy); (2) SkullBlockRenderer::render (found from the 26.63 render through the constructor text and slot 2) calls it with the BlockType name of the world block at the place where the 26.63 render called renderSkull, and the 70.26 counterpart of the 26.63 item path (SkullBlockRenderer.renderSkullUpright) calls it too. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552); on 26.63 this code is inline in renderSkull. Class: real code change."

  recipe {
    step "pattern" {
      bytes = "44 89 CB 0F 28 F2 49 89 D6 48 89 CF 48 8D 15 ?? ?? ?? ?? 4C 89 F1 E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 83 3B ?? 74 ?? 48 89 5D ?? 48 89 F1 4C 89 F2 0F 28 D6 45 31 C9 E8"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "SkullBlockRenderer.getSkullMaterials" {
  kind   = "function"
  hooked = true
  notes  = "SkullMaterials* SkullBlockRenderer::getSkullMaterials(SkullBlockRenderer const* this, SkullMaterials* out, HashedString const& name, bool glint) on 70.x (the name is ours). rcx = the renderer, rdx = a 56-byte out buffer, r8 = the skull name, r9b = glint. It compares the name with the skull names and picks the material group member (this+F8h, 238h, 378h, 5F8h, 738h, 4B8h or 878h); it writes the group's two material pointers, a flag, a third material, a second flag, the glint material (this+9B8h, only when glint) and 0 into out (null materials become a shared empty material) and returns out in rax. Recipe 0 is the head of the body after the pushes: `mov ebx,r9d; mov r14,r8; mov rsi,rdx; mov rdi,rcx; lea rdx,[rip+..]; mov rcx,r8; call` (the first name compare), then function_start_of. Recipe 1 is the call in SkullBlockRenderer::render: `lea r15,[rbp+..]; mov rcx,rsi; mov rdx,r15; mov r8,r14; xor r9d,r9d; call`, read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The 26.63 renderSkull (SkullBlockRenderer.renderSkull) chose the model by the skull name with the members this+A10h, A18h, 9D8h, 9E0h, 9E8h, 9F0h, 9F8h (with the +140h animation store and the glint-indexed texture copy) and the material set with this+238h, 378h, 5F8h, 738h, 4B8h, 878h and 9B8h (glint); on 70.x these are two separate functions. Evidence on 70.26: (1) the body has exactly the material part of the 26.63 renderSkull body (the same members F8h..878h and the glint member 9B8h); (2) SkullBlockRenderer::render calls it with the same name it passes to getSkullModel, and it has four callers, one for each of the four 26.63 callers of renderSkull. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). Class: real code change."

  recipe {
    step "pattern" {
      bytes = "44 89 CB 4D 89 C6 48 89 D6 48 89 CF 48 8D 15 ?? ?? ?? ?? 4C 89 C1 E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8D 7D ?? 48 89 F1 4C 89 FA 4D 89 F0 45 31 C9 E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}

entry "SkullBlockRenderer.renderSkullUpright" {
  kind   = "function"
  hooked = true
  notes  = "void SkullBlockRenderer::renderSkullUpright(SkullBlockRenderer* this, BaseActorRenderContext& ctx, dragon::RenderMetadata const& metadata, Vec3 const& pos, HashedString const& name, float animation, bool glint) on 70.x (the name is ours). rcx = the renderer, rdx = the context, r8 = the metadata, r9 = the position; [rsp+20h] the name (a HashedString pointer), [rsp+28h] the animation (float), [rsp+30h] glint (byte) at the call. It pushes a copy of the matrix stack top (ctx+28h, then +18h), moves it to pos plus the floor offset, turns it by the fixed item turn (the same constant as the turn of face 3 in render), calls SkullBlockRenderer.getSkullModel and, when a model is found, SkullBlockRenderer.getSkullMaterials and the draw, then pops the matrix. It draws nothing for a name that is not a skull. It is the 70.x form of the 26.63 call `renderSkull(ctx, metadata, pos, face 1, 180.0, name, null, null, .., animation, glint)`; any other pose is a transform that the caller puts on the matrix stack top before the call. Recipe 0 is the call in the item path: `mov rax,[rax+disp32]` (the Item block type) `; test rax,rax; je; mov rcx,[r13+disp8]` (the skull renderer in the dispatcher) `; test rcx,rcx; je; add rax,imm32` (the name member offset) `; mov [rsp+..],r12b; mov [rsp+..],rax; mov dword ptr [rsp+..],imm32` (animation 0) `; lea r9,[rip+..]; mov rdx,rsi; mov r8,[rbp+..]; call`, read as the call target. Recipe 1 is the body after the model call: `test rax,rax; je; mov r15,rax; mov rcx,rdi; mov rdx,r14; mov r8,r13; mov r9d,r12d; call` (getSkullMaterials), then function_start_of. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Evidence on 70.26: (1) the 26.63 item path that called renderSkull with face 1, turn 180.0 and null material and texture (`mov rax,[rax+178h]; test; je; mov rcx,[r12+20h]; test; je; add rax,0E0h; ...; lea r9,[rip+..]; mov rdx,rsi; mov r8,[rbp+..]; call renderSkull`) is found on 70.26 with the same order (Item+188h, dispatcher+20h, name +0E8h, animation 0, the rip-relative position, the glint byte), and it calls this function in place of renderSkull, with no face, turn, material or texture argument; (2) its body has the floor part of the 26.63 renderSkull body (the matrix push, the floor offset, the fixed turn) and calls the model and material helpers that hold the rest of the 26.63 body. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). Class: real code change."

  recipe {
    step "pattern" {
      bytes  = "48 8B 80 ?? ?? ?? ?? 48 85 C0 74 ?? 49 8B 4D ?? 48 85 C9 74 ?? 48 05 ?? ?? ?? ?? 44 88 64 24 ?? 48 89 44 24 ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 0D ?? ?? ?? ?? 48 89 F2 4C 8B 45 ?? E8"
      offset = 59
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes = "48 85 C0 74 ?? 49 89 C7 48 89 F9 4C 89 F2 4D 89 E8 45 89 E1 E8"
    }
    step "function_start_of" {}
  }

  check "function_start" {}

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}
