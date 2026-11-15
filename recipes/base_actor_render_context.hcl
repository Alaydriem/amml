entry "BaseActorRenderContext.ctor" {
  kind     = "function"
  notes    = "BaseActorRenderContext::BaseActorRenderContext(ScreenContext& screenContext, IClientInstance& client, IMinecraftGame& game): rcx the object (a buffer of BaseActorRenderContext.size bytes, 8-byte aligned), rdx the ScreenContext, r8 the client instance, r9 the game; the object is returned in rax. It stores the vtable, the client and game pointers and the item renderer at +58h, and ends with the byte stores at +0B8h..+0BCh. 24 callers build one on the stack; the game itself does so for every inventory item it draws, passing the game from the client's virtual at +270h (ClientInstance.getMinecraftGame). Recipe 0 is the call in the helper that draws one flying item: `mov rdx,[rsi+..]; lea r15,[rbp+..]; mov rcx,r15; mov r8,r14; mov r9,rax; call`, read as the call target. Recipe 1 is the call in the inventory item renderer: `lea r15,[rbp+..]; mov rcx,r15; mov rdx,r14; mov r8,rbx; mov r9,rax; call`, read the same way. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 57E1DA0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" {
      bytes  = "48 8B 56 ?? 4C 8D 7D ?? 4C 89 F9 4D 89 F0 49 89 C1 E8 ?? ?? ?? ??"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 8D 7D ?? 4C 89 F9 4C 89 F2 49 89 D8 49 89 C1 E8 ?? ?? ?? ??"
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
    value = 456
  }
}

entry "BaseActorRenderContext.dtor" {
  kind     = "function"
  notes    = "BaseActorRenderContext::~BaseActorRenderContext() (the complete object destructor, not a deleting one): rcx the object, void. It stores the base vtable, then releases the variant at +0A8h (a switch on its index byte) and the members. 47 callers destroy a stack object after the draw. Both recipes start at a caller that is found by its own prologue and look inside it, so the pattern needs to be unique only in that function. Recipe 0 is in the flying item helper (the function with the two renderGuiItemNew calls): `call <draw>; nop; lea rdx,[rbp+..]; lea r8,[rbp+..]; mov rcx,rsi; call; nop; lea rcx,[rbp+..]; call`, read as the last call target. Recipe 1 is in the inventory item renderer: `lock dec dword ptr [rsi+0Ch]; jne; mov rax,[rsi]; mov rax,[rax+8]; mov rcx,rsi; call [guard]; lea rcx,[rbp+..]; call`, read the same way. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 57E2220h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 44 0F 29 85 ?? ?? ?? ?? 0F 29 BD ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 49 8B" }
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 90 48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 90 48 8D 4D ?? E8 ?? ?? ?? ??"
      offset = 30
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48" }
    step "pattern" {
      bytes  = "F0 FF 4E ?? 75 ?? 48 8B 06 48 8B 40 ?? 48 89 F1 FF 15 ?? ?? ?? ?? 48 8D 4D ?? E8 ?? ?? ?? ??"
      offset = 26
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

entry "BaseActorRenderContext.size" {
  kind     = "field"
  notes    = "sizeof(BaseActorRenderContext): 0C0h on 26.60 (the LeviLamina header gives the same; the game builds the object in a 0C0h local). The constructor's last stores are `mov dword ptr [r15+0B8h],10101h` and `mov word ptr [r15+0BCh],0`, so the last member ends at 0BEh and the object is rounded up to the 8-byte alignment of its pointers, 0C0h. The recipe reads the displacement of that last store inside BaseActorRenderContext.ctor (0BCh) and adds 4 (the 2-byte store plus the padding to the alignment); a later build that adds a member after it changes the displacement, and one that adds a longer last store breaks the add, so the range check bounds it. A field with one recipe and a range check, as the porting guide has it for a size. Every displacement and immediate byte is a wildcard. The gate (check entry_is) LevelChunk.mSubChunks = 456 (1C8h, the 26.60 layout) ties this entry to the 26.60 build: it refuses as not for this build on 26.51, and must be widened when a later build is added. Value 0C0h on 6ad79ee4-17d51891a1119914 (26.60)."

  recipe {
    step "ref" { entry = "BaseActorRenderContext.ctor" }
    step "pattern" { bytes = "66 41 C7 87 ?? ?? ?? ?? ?? ?? ?? ??" }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = 4 }
  }

  check "range" {
    min = 128
    max = 256
  }

  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "BaseActorRenderContext.mBlockEntityRenderDispatcher" {
  kind  = "field"
  notes = "Offset of `BlockActorRenderDispatcher& mBlockEntityRenderDispatcher` in BaseActorRenderContext: 30h on 26.63 (6aed1906-b127418e74d2f2f9). The header order puts it after the ScreenContext, ClientInstance, IMinecraftGame, render dispatcher and texture references. Every caller of BlockActorRenderDispatcher.render loads it from the context into rcx right before the call. Recipe 0 is in the shared LevelRendererCamera::renderBlockEntities body (0x5369340): `mov rcx,[rsi+30h]; mov byte ptr [rsp+..],imm8; mov eax,[rip+..]` (rsi = the context), the displacement of operand 1. Recipe 1 is in MovingBlockActorRenderer's draw (0x6a83a10), before its call of BlockActorRenderDispatcher.render: `mov rcx,[r15+30h]; mov byte ptr [rbp+..],imm8; mov edx,[rip+..]`. A third load (`mov rcx,[rdx+30h]`) is in the block selection overlay draw (0x539b520). The two recipes are in two functions. Every displacement and immediate is a wildcard. The range check is 10h..60h."

  recipe {
    step "pattern" {
      bytes = "48 8B 4E ?? C6 84 24 ?? ?? ?? ?? ?? 8B 05"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes = "49 8B 4F ?? C6 45 ?? ?? 8B 15"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 96
  }
}

entry "BaseActorRenderContext.mImpl" {
  kind  = "field"
  notes = "Offset of `std::unique_ptr<BaseActorRenderContext::Impl> mImpl` in BaseActorRenderContext: A8h on 26.63 (6aed1906-b127418e74d2f2f9). The Impl is a 238h-byte heap object; the LeviLamina header gives its members as OffscreenCaptureDescription (20h bytes), then mCameraTargetPosition (Vec3, 20h), mCameraPosition (Vec3, 2Ch), mWorldClipRegion and mHistoricalFrameTimes. The header order puts mImpl after mRenderUniqueIdOverride. How it was found: the BaseActorRenderContext constructor (BaseActorRenderContext.ctor, 0x57e22c0) allocates 238h bytes, keeps `lea rcx,[r15+A8h]` (the address of the unique_ptr) and later fills Impl+20h..+34h from the LevelRendererCamera at LevelRenderer+480h (LevelRendererCamera.mCameraTargetPos 658h and mCameraPos 64Ch). BlockActorRenderDispatcher's first render form (0x6a5e650) loads `mov r13,[rdx+A8h]` from the context and passes `lea rdx,[r13+20h]` to the component's visibility test (slot 6), then computes renderPos = component position - [r13+20h]. Recipe 0 is the head of that first form, after the prologue: `mov rsi,r9; mov rdi,r8; mov rbx,rdx; mov r14,rcx; mov r13,[rdx+disp32]; lea rdx,[r13+disp8]; mov rax,[r9]; mov rax,[rax+disp8]; mov rcx,r9; call [guard]`, offset 12, the displacement of operand 1 of the load. Recipe 1 is in the constructor, after the allocation of the Impl: `test rax,rax; je rel32; lea rcx,[r15+disp32]; mov [rbp+disp8],rcx; mov rbx,rax; xorps xmm0,xmm0; movaps [rax+disp8],xmm0`, offset 9, the displacement of operand 1 of the lea. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range check is 80h..D0h."

  recipe {
    step "pattern" {
      bytes  = "4C 89 CE 4C 89 C7 48 89 D3 49 89 CE 4C 8B AA ?? ?? ?? ?? 49 8D 55 ?? 49 8B 01 48 8B 40 ?? 4C 89 C9 FF 15"
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 85 C0 0F 84 ?? ?? ?? ?? 49 8D 8F ?? ?? ?? ?? 48 89 4D ?? 48 89 C3 0F 57 C0 0F 29 40"
      offset = 9
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 208
  }
}

entry "BaseActorRenderContext.Impl.mCameraTargetPosition" {
  kind  = "field"
  notes = "Offset of `Vec3 mCameraTargetPosition` in BaseActorRenderContext::Impl (the object that BaseActorRenderContext.mImpl points to): 20h on 26.63 (6aed1906-b127418e74d2f2f9). The LeviLamina header puts it after the 20h-byte OffscreenCaptureDescription and before mCameraPosition (2Ch). It is a copy of LevelRendererCamera.mCameraTargetPos that the BaseActorRenderContext constructor takes from the LevelRendererCamera at LevelRenderer+480h, the same object as the LevelRendererPlayer whose renderBlockEntities runs in the frame. BlockActorRenderDispatcher's first render form (0x6a5e650) uses it as the origin of renderPos: renderPos = Vec3(component position) - mCameraTargetPosition. Recipe 0 is in that first form: `subss xmm0,[r13+disp8]` (the z), `mov r10,[rbp+disp32]; movsd xmm1,[rax]; cvtdq2ps xmm1,xmm1; movsd xmm2,[r13+disp8]; subps xmm1,xmm2` (x and y), offset 20, the displacement of operand 1 of the movsd. Recipe 1 is in the BaseActorRenderContext constructor (BaseActorRenderContext.ctor, 0x57e22c0): the second of the two camera copies, the one before the `[rax+3C0h]` test: `mov rdx,[r9]; mov r8d,[rcx+disp32]; mov [rdx+disp8],r8d; mov rcx,[rcx+disp32]; mov [rdx+disp8],rcx; mov rcx,[rax+disp32]; test rcx,rcx; je`, offset 21, the displacement of operand 0 of the qword store (the x and y of the target; the source is LevelRendererCamera.mCameraTargetPos, 658h). The first copy (mCameraPosition, 2Ch, from mCameraPos 64Ch) is followed by `mov rcx,[rax+disp32]; mov rdx,[r9]` instead, so the pattern does not match it. The two recipes are in two functions and share no bytes. Every displacement, immediate and branch byte is a wildcard. The range check is 10h..28h; it leaves out mCameraPosition (2Ch)."

  recipe {
    step "pattern" {
      bytes  = "F3 41 0F 5C 45 ?? 4C 8B 95 ?? ?? ?? ?? F2 0F 10 08 0F 5B C9 F2 41 0F 10 55 ?? 0F 5C CA"
      offset = 20
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 8B 11 44 8B 81 ?? ?? ?? ?? 44 89 42 ?? 48 8B 89 ?? ?? ?? ?? 48 89 4A ?? 48 8B 88 ?? ?? ?? ?? 48 85 C9 74"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 40
  }
}
