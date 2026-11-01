entry "CameraRegistry.mCameraEntities" {
  kind     = "field"
  notes    = "Offset of `mCameraEntities` (std::vector<OwnerPtr<EntityId>>) in CameraRegistry: 1D8h in 26.51 (LeviLamina header sum: EnableNonOwnerReferences 18h (vtable and control block), mReflectionCtx 18h (80h), mLoader 98h (110h), mComponentInitializers 1A8h, mCameras 1C0h, mCameraEntities 1D8h). Recipe 0 is the camera registry function 0x9f28e0 (it walks CameraRegistry::mCameraEntities and looks up the CameraComponent of each entity: `add r14,20h; cmp r14,r12; je; mov rcx,[r14+8]; mov eax,[r14+10h]; mov rdx,[rcx+48h]; ...`; the pattern is the whole lookup, 217 bytes, up to the call that builds the name to compare); the pattern starts at the step of the walk, and the vector is loaded just before it: the recipe reads `mov r14,[rax+1D8h]` from its own pattern (`mov r14,[rax+..]; mov r12,[rax+..]; jmp; call`). Recipe 1 is the same function taking the address of the vector (`lea rax,[rsi+1D8h]; mov [rbp+..],rax; xor eax,eax; lea rdi,[rip+..]`). The two recipes are in one function (no other function walks this vector by offset). Every other displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" {
      bytes  = "4C 8B B0 ?? ?? ?? ?? 4C 8B A0 ?? ?? ?? ?? EB ?? E8"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8D 86 ?? ?? ?? ?? 48 89 45 ?? 31 C0 48 8D 3D ?? ?? ?? ??"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 768
  }
}

entry "CameraRegistry.cameraEntityStride" {
  kind     = "field"
  notes    = "The size of one element of CameraRegistry::mCameraEntities (OwnerPtr<EntityId>, a std::optional<EntityContext>): 20h in 26.51. Recipe 0 is the step of the walk in the camera registry function 0x9f28e0 (`add r14,20h; cmp r14,r12; je`, the start of the long pattern); recipe 1 is the end pointer step after an append in the same function (`add qword ptr [rax+1E0h],20h; jmp`). One function. Every other displacement, immediate and branch byte is a wildcard. Recipe 1 now anchors on the in-place push of a camera entity in the camera registry function: the engaged flag of the new optional<EntityContext> is set (`mov byte ptr [rdx+18h],1`), then the vector end moves by the stride (`mov rax,[rbp+..]; add qword ptr [rax+..],20h; jmp`). What changed in 6ad79ee4-17d51891a1119914: the alignment nop after the jmp (26.51 `nop dword ptr [rax]`, 26.60 `nop word ptr [rax+rax]`) and the frame offsets; the stride is 20h on both builds."

  recipe {
    step "pattern" {
      bytes  = "49 83 C6 ?? 4D 39 E6 0F 84 ?? ?? ?? ?? 49 8B 4E ?? 41 8B 46 ?? 48 8B 51 ?? 4C 8B 41 ?? 49 29 D0 49 C1 E8 ?? 41 FF C8 41 81 E0 ?? ?? ?? ?? 4E 8D 0C C2 48 8B 51 ?? 66 2E 0F 1F 84 ?? ?? ?? ?? ?? 4D 8B 01 49 83 F8 ?? 74 ?? 49 C1 E0 ?? 4E 8D 0C 02 42 81 7C 02 ?? ?? ?? ?? ?? 75 ?? 4C 01 C2 48 39 51 ?? 74 ?? 48 8B 4A ?? 48 85 C9 74 ?? 89 C2 81 E2 ?? ?? ?? ?? 41 89 D0 41 C1 E8 ?? 4C 8B 49 ?? 4C 8B 51 ?? 4D 29 CA 49 C1 FA ?? 4D 39 D0 0F 83 ?? ?? ?? ?? 4F 8B 04 C1 4D 85 C0 0F 84 ?? ?? ?? ?? 81 E2 ?? ?? ?? ?? 25 ?? ?? ?? ?? 41 8B 34 90 31 F0 3D ?? ?? ?? ?? 0F 87 ?? ?? ?? ?? 89 F0 25 ?? ?? ?? ?? 48 8B 49 ?? C1 E8 ?? 4C 8B 3C 01 4D 85 FF 0F 84 ?? ?? ?? ??"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "C6 42 ?? ?? 48 8B 85 ?? ?? ?? ?? 48 83 80 ?? ?? ?? ?? ?? E9"
      offset = 11
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 24
    max = 64
  }
}
