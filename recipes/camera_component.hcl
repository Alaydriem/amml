entry "CameraComponent.typeId" {
  kind     = "field"
  notes    = "The entt type id of MinecraftCamera::CameraComponent (the FNV-1a hash of the type name): 4F6047C7h on every build so far (26.51, 26.63, 1.26.70.26). It is read where the game looks up the CameraComponent of an entity. Two shapes of that lookup exist. Up to 26.63 the pool lookup is inlined and the hash is the bucket mask (`dec r8d; and r8d,4F6047C7h; lea r9,[rdx+r8*8]`). On 1.26.70.26 it is a call to one shared function that takes the hash in edx (`mov rcx,[elem+8]; mov esi,[elem+10h]; mov edx,4F6047C7h; call`). Both recipes read the immediate of the instruction at the cursor. Recipe 0 is the camera registry function (it walks CameraRegistry::mCameraEntities, 20h-byte elements with the registry at +8 and the entity at +10h, and looks up the CameraComponent of each). It is a pattern_any with one alternative per shape, both with the cursor at the instruction that holds the hash: alternative 0 is the 26.63 inline lookup, from the element reads (`mov rdx,[rcx+48h]; mov r8,[rcx+50h]; sub; shr; dec; and r8d,hash`) through the whole probe loop to the page lookup; alternative 1 is the 70.26 lookup, which starts at the nop after the call of the previous element, runs through the element loop end (`add rbx,8; cmp; jne`) and the element reads, reads the hash in `mov edx,imm`, and ends at the call after the pool page lookup, which comes before the FNV hash of the component's mId string in that function. Recipe 1 is LevelRendererPlayer::setupCamera (it looks up the CameraComponent of the active camera entity and reads its orientation and position): the same two shapes found by pattern_any inside the function from the setupCamera entry; alternative 0 and 1 are the two inline forms (26.51 and 26.63 differ in the nop of the probe loop), alternative 2 is the call form (`mov edx,hash; call; test rax,rax; je; mov ecx,ebx; and ecx,mask`). The two recipes are in two functions. Every other displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. What changed in 6afa4e47-dadc375797a3d31a: EnTT type ids and storage lookups were rebuilt; the lookup is now a call with the hash in edx, no longer inlined with a bucket mask (class: would have survived with a better recipe, with a new alternative per shape). Evidence on 6afa4e47: (1) the 26.63 registry function and setupCamera each read the hash from the same operand role (the CameraComponent lookup of one entity), and the 70.26 recipe 0 alternative and recipe 1 alternative, each drafted from the matching 26.63 lookup site, each match once in a different function and give the same value; (2) the 70.26 registry function loads CameraRegistry::mCameraEntities at the same offset as 26.63 (1D8h and 1E0h, begin and end) and walks 20h-byte elements with the entity at +10h and the registry at +8, as the 26.63 function does."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 51 ?? 4C 8B 41 ?? 49 29 D0 49 C1 E8 ?? 41 FF C8 41 81 E0 ?? ?? ?? ?? 4E 8D 0C C2 48 8B 51 ?? 66 2E 0F 1F 84 ?? ?? ?? ?? ?? 4D 8B 01 49 83 F8 ?? 74 ?? 49 C1 E0 ?? 4E 8D 0C 02 42 81 7C 02 ?? ?? ?? ?? ?? 75 ?? 4C 01 C2 48 39 51 ?? 74 ?? 48 8B 4A ?? 48 85 C9 74 ?? 89 C2 81 E2 ?? ?? ?? ?? 41 89 D0 41 C1 E8 ?? 4C 8B 49 ?? 4C 8B 51 ?? 4D 29 CA 49 C1 FA ?? 4D 39 D0 0F 83 ?? ?? ?? ?? 4F 8B 04 C1 4D 85 C0 0F 84 ?? ?? ?? ?? 81 E2 ?? ?? ?? ?? 25 ?? ?? ?? ?? 41 8B 34 90 31 F0 3D ?? ?? ?? ?? 0F 87 ?? ?? ?? ?? 89 F0 25 ?? ?? ?? ?? 48 8B 49 ?? C1 E8 ?? 4C 8B 3C 01 4D 85 FF 0F 84 ?? ?? ?? ??",
        "90 48 83 C3 ?? 4C 39 F3 75 ?? 49 8B 4F ?? 41 8B 77 ?? BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 85 C0 0F 84 ?? ?? ?? ?? 89 F1 81 E1 ?? ?? ?? ?? 89 CA C1 EA ?? 4C 8B 40 ?? 4C 8B 48 ?? 4D 29 C1 49 C1 F9 ?? 4C 39 CA 0F 83 ?? ?? ?? ?? 49 8B 14 D0 48 85 D2 0F 84 ?? ?? ?? ?? 81 E1 ?? ?? ?? ?? 41 89 F0 41 81 E0 ?? ?? ?? ?? 44 33 04 8A 41 81 F8 ?? ?? ?? ?? 0F 87 ?? ?? ?? ?? 89 75 ?? 48 89 C1 48 8D 55 ?? 48 89 C6 E8",
      ]
      offset = 18
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "ref" {
      entry = "LevelRendererPlayer.setupCamera"
    }
    step "pattern_any" {
      patterns = [
        "41 81 E0 ?? ?? ?? ?? 4E 8D 0C C2 48 8B 51 ?? 66 0F 1F 44 00 ?? 4D 8B 01 49 83 F8 ?? 0F 84",
        "41 81 E0 ?? ?? ?? ?? 4E 8D 0C C2 48 8B 51 ?? 0F 1F 84 00 ?? ?? ?? ?? 4D 8B 01 49 83 F8 ?? 0F 84",
        "BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 89 D9 81 E1",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 4294967295
  }
}

entry "CameraComponent.sizeFactor" {
  kind     = "field"
  notes    = "The size of MinecraftCamera::CameraComponent is built as factor << shift: the factor is 9 in 26.51 (`lea rax,[rsi+rsi*8]; shl eax,5`: 9 * 20h = 120h; LeviLamina header sum: mId 0 (30h), mOrientation 30h, mPosition 40h, mAspectRatio 4Ch, mFieldOfView 50h, mNearPlane 54h, mFarPlane 58h, mPostViewTransform 5Ch, mSavedProjection 9Ch, mSavedModelView DCh, 11Ch, aligned to 8 = 120h). Read with the scale_factor operand. Recipe 0 is a camera registry function (it compares the HashedString hash of each component), recipe 1 LevelRendererPlayer::setupCamera. Two functions. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now anchors on the component address in setupCamera (`and edx,7Fh; lea rax,[rdx+rdx*8]; shl eax,5; add rbx,rax; movss xmm4,[rbx+30h]; movss xmm1,[rbx+34h]`, the orientation reads after it). What changed in 6ad79ee4-17d51891a1119914: in LevelRendererPlayer::setupCamera the compiler put an 8-byte nop in the pool lookup loop (26.51 a 6-byte one), so the 276-byte pattern stopped matching; the lookup itself (pool map at 48h/68h, 20h-byte nodes, mask 3FFFFh, 2048-entry sparse pages, payload at 50h, 128-entry payload pages, stride 9 << 5) is the same on both builds. Layout gate: 26.63 only; on 70.x the component is 128h bytes and the stride is one `imul reg,128h` with no factor and shift (every member after mId moved by 8: mId, a HashedString, grew from 30h to 38h); the replacement is CameraComponent.size."

  recipe {
    step "pattern" {
      bytes  = "83 E6 ?? 48 8D 04 F6 C1 E0 ?? 49 01 C7 49 8B 07 48 3B 45 ??"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "scale_factor"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "83 E2 ?? 48 8D 04 D2 C1 E0 ?? 48 01 C3 F3 0F 10 63 ?? F3 0F 10 4B ??"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "scale_factor"
    }
  }

  check "range" {
    min = 2
    max = 32
  }
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "CameraComponent.sizeShift" {
  kind     = "field"
  notes    = "The shift of the CameraComponent size (factor << shift, see CameraComponent.sizeFactor): 5 in 26.51 (`shl eax,5`). Recipe 0 is a camera registry function (it compares the HashedString hash of each component), recipe 1 LevelRendererPlayer::setupCamera. Two functions. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now anchors on the component address in setupCamera (as CameraComponent.sizeFactor recipe 1). What changed in 6ad79ee4-17d51891a1119914: in LevelRendererPlayer::setupCamera the compiler put an 8-byte nop in the pool lookup loop (26.51 a 6-byte one), so the 276-byte pattern stopped matching; the lookup itself (pool map at 48h/68h, 20h-byte nodes, mask 3FFFFh, 2048-entry sparse pages, payload at 50h, 128-entry payload pages, stride 9 << 5) is the same on both builds. Layout gate: 26.63 only; on 70.x the component is 128h bytes and the stride is one `imul reg,128h` with no factor and shift (every member after mId moved by 8: mId, a HashedString, grew from 30h to 38h); the replacement is CameraComponent.size."

  recipe {
    step "pattern" {
      bytes  = "83 E6 ?? 48 8D 04 F6 C1 E0 ?? 49 01 C7 49 8B 07 48 3B 45 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "83 E2 ?? 48 8D 04 D2 C1 E0 ?? 48 01 C3 F3 0F 10 63 ?? F3 0F 10 4B ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 0
    max = 8
  }
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 456
  }
}

entry "CameraComponent.size" {
  kind     = "field"
  notes    = "The size of MinecraftCamera::CameraComponent on 70.x, the stride of its pool pages: 128h on 70.26, read from the immediate of `imul reg,imm32`. It replaces CameraComponent.sizeFactor and CameraComponent.sizeShift (26.63: `lea rax,[rdx+rdx*8]; shl eax,5`, 9 << 5 = 120h), which have no instruction to read on 70.x. Layout on 70.26 (from the reads of LevelRendererPlayer::setupCamera after the stride): mId (HashedString) 0 to 38h, mOrientation 38h (glm::quat, four floats), mPosition 48h (three floats), mAspectRatio 54h, mFieldOfView 58h, mNearPlane 5Ch, mFarPlane 60h, mPostViewTransform 64h (`lea r12,[rbx+64h]`), then mSavedProjection and mSavedModelView (40h each) to 124h, aligned to 8 = 128h. Every member after mId is 8 bytes later than on 26.63 (orientation 30h, position 40h, projection floats 4Ch to 58h, mPostViewTransform 5Ch): mId grew from 30h to 38h. Recipe 0 is LevelRendererPlayer::setupCamera: ref LevelRendererPlayer.setupCamera, then the component address before the orientation reads, `and eax,imm8; imul rax,rax,imm32; add rbx,rax; movss xmm4,[rbx+..]; movss xmm1,[rbx+..]`, and the immediate of the imul. Recipe 1 is the camera registry function that hashes a camera name (an inline FNV loop) and stores the new HashedString into the component: `and r10d,imm8; imul rdx,r10,imm32; add rcx,rdx; mov [rbp+..],rax; movdqa xmm0,[rbp+..]`, the immediate of the imul. Two functions. Every displacement, immediate and branch byte is a wildcard. Check: range 100h..200h. Evidence from the 26.63 function LevelRendererPlayer::setupCamera (a resolved entry on both builds): (1) at the place where 26.63 builds the component address from the pool index (`and edx,7Fh; lea rax,[rdx+rdx*8]; shl eax,5; add rbx,rax`, 120h), 70.26 has `and eax,7Fh; imul rax,rax,128h; add rbx,rax`, followed by the same orientation computation; (2) every member read after that address in the 26.63 body (orientation 30h/34h/38h, position 40h/48h, projection 4Ch to 58h, `lea r15,[rbx+5Ch]`) is read 8 bytes later in the 70.26 body (38h/3Ch/40h, 48h/50h, 54h to 60h, `lea r12,[rbx+64h]`), which fits a size of 120h + 8. Gate: 70.x only (entry_is LevelChunk.mSubChunks = 552). What changed in 6afa4e47-dadc375797a3d31a (70.26): the component grew by 8 bytes (the HashedString layout), and 128h is not 9 << 5, so the compiler uses imul. Class: real code change (layout)."

  recipe {
    step "ref" { entry = "LevelRendererPlayer.setupCamera" }
    step "pattern" {
      bytes  = "83 E0 ?? 48 69 C0 ?? ?? ?? ?? 48 01 C3 F3 0F 10 63 ?? F3 0F 10 4B ??"
      offset = 3
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "41 83 E2 ?? 49 69 D2 ?? ?? ?? ?? 48 01 D1 48 89 45 ?? 66 0F 6F 85"
      offset = 4
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
  check "entry_is" {
    entry = "LevelChunk.mSubChunks"
    value = 552
  }
}
