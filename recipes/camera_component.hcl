entry "CameraComponent.typeId" {
  kind     = "field"
  notes    = "The entt type id of MinecraftCamera::CameraComponent (the FNV-1a hash of the type name): 4F6047C7h in 26.51. Read from the bucket mask (`and r8d,4F6047C7h`). Recipe 0 is the camera registry function 0x9f28e0 (it walks CameraRegistry::mCameraEntities and looks up the CameraComponent of each entity: `add r14,20h; cmp r14,r12; je; mov rcx,[r14+8]; mov eax,[r14+10h]; mov rdx,[rcx+48h]; ...`; the pattern is the whole lookup, 217 bytes, up to the call that builds the name to compare); recipe 1 is LevelRendererPlayer::setupCamera 0x46cfe60 (it looks up the CameraComponent of the active camera entity, which a call returns into a local EntityContext at rsp+60h, and reads its orientation and position; the pattern is the lookup, 276 bytes, from after that call to the quaternion reads). Each reads the operand of the instruction named. The two recipes are in two functions. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now starts from the LevelRendererPlayer.setupCamera entry and finds the bucket mask of the CameraComponent pool lookup in that function (`and r8d,typeid; lea r9,[rdx+r8*8]; mov rdx,[rcx+68h]`, the only pool lookup there). What changed in 6ad79ee4-17d51891a1119914: in LevelRendererPlayer::setupCamera the compiler put an 8-byte nop in the pool lookup loop (26.51 a 6-byte one), so the 276-byte pattern stopped matching; the lookup itself (pool map at 48h/68h, 20h-byte nodes, mask 3FFFFh, 2048-entry sparse pages, payload at 50h, 128-entry payload pages, stride 9 << 5) is the same on both builds."

  recipe {
    step "pattern" {
      bytes  = "49 83 C6 ?? 4D 39 E6 0F 84 ?? ?? ?? ?? 49 8B 4E ?? 41 8B 46 ?? 48 8B 51 ?? 4C 8B 41 ?? 49 29 D0 49 C1 E8 ?? 41 FF C8 41 81 E0 ?? ?? ?? ?? 4E 8D 0C C2 48 8B 51 ?? 66 2E 0F 1F 84 ?? ?? ?? ?? ?? 4D 8B 01 49 83 F8 ?? 74 ?? 49 C1 E0 ?? 4E 8D 0C 02 42 81 7C 02 ?? ?? ?? ?? ?? 75 ?? 4C 01 C2 48 39 51 ?? 74 ?? 48 8B 4A ?? 48 85 C9 74 ?? 89 C2 81 E2 ?? ?? ?? ?? 41 89 D0 41 C1 E8 ?? 4C 8B 49 ?? 4C 8B 51 ?? 4D 29 CA 49 C1 FA ?? 4D 39 D0 0F 83 ?? ?? ?? ?? 4F 8B 04 C1 4D 85 C0 0F 84 ?? ?? ?? ?? 81 E2 ?? ?? ?? ?? 25 ?? ?? ?? ?? 41 8B 34 90 31 F0 3D ?? ?? ?? ?? 0F 87 ?? ?? ?? ?? 89 F0 25 ?? ?? ?? ?? 48 8B 49 ?? C1 E8 ?? 4C 8B 3C 01 4D 85 FF 0F 84 ?? ?? ?? ??"
      offset = 39
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
  notes    = "The size of MinecraftCamera::CameraComponent is built as factor << shift: the factor is 9 in 26.51 (`lea rax,[rsi+rsi*8]; shl eax,5`: 9 * 20h = 120h; LeviLamina header sum: mId 0 (30h), mOrientation 30h, mPosition 40h, mAspectRatio 4Ch, mFieldOfView 50h, mNearPlane 54h, mFarPlane 58h, mPostViewTransform 5Ch, mSavedProjection 9Ch, mSavedModelView DCh, 11Ch, aligned to 8 = 120h). Read with the scale_factor operand. Recipe 0 is the camera registry function 0x9f28e0, recipe 1 LevelRendererPlayer::setupCamera. Two functions. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now anchors on the component address in setupCamera (`and edx,7Fh; lea rax,[rdx+rdx*8]; shl eax,5; add rbx,rax; movss xmm4,[rbx+30h]; movss xmm1,[rbx+34h]`, the orientation reads after it). What changed in 6ad79ee4-17d51891a1119914: in LevelRendererPlayer::setupCamera the compiler put an 8-byte nop in the pool lookup loop (26.51 a 6-byte one), so the 276-byte pattern stopped matching; the lookup itself (pool map at 48h/68h, 20h-byte nodes, mask 3FFFFh, 2048-entry sparse pages, payload at 50h, 128-entry payload pages, stride 9 << 5) is the same on both builds."

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
}

entry "CameraComponent.sizeShift" {
  kind     = "field"
  notes    = "The shift of the CameraComponent size (factor << shift, see CameraComponent.sizeFactor): 5 in 26.51 (`shl eax,5`). Recipe 0 is the camera registry function 0x9f28e0, recipe 1 LevelRendererPlayer::setupCamera. Two functions. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now anchors on the component address in setupCamera (as CameraComponent.sizeFactor recipe 1). What changed in 6ad79ee4-17d51891a1119914: in LevelRendererPlayer::setupCamera the compiler put an 8-byte nop in the pool lookup loop (26.51 a 6-byte one), so the 276-byte pattern stopped matching; the lookup itself (pool map at 48h/68h, 20h-byte nodes, mask 3FFFFh, 2048-entry sparse pages, payload at 50h, 128-entry payload pages, stride 9 << 5) is the same on both builds."

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
}
