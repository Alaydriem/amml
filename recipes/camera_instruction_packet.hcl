entry "CameraInstructionPacket.mCameraInstruction" {
  kind     = "field"
  notes    = "Offset of `mCameraInstruction` (CameraInstruction, 150h bytes) in CameraInstructionPacket: 30h in 26.51 (LeviLamina header: PayloadPacket<CameraInstructionPacketPayload>, the Packet base is 30h bytes; the packet is 188h bytes, as its deleting destructor frees). The trivially copyable front of the instruction (B0h bytes, up to mSpline) is copied with memcpy from packet+30h. Recipe 0 is the packet copy constructor (0x84ede70: `lea rax,[rip+vtable]; mov [rbx],rax; lea rcx,[rbx+30h]; lea rdx,[rdi+30h]; mov r8d,0B0h; call`), reading the displacement of `lea rcx,[rbx+30h]`. Recipe 1 is the camera system that handles the packet (0x5679ad0: `mov r14,rdx; mov rsi,rcx; lea rdx,[rcx+30h]; lea rcx,[rbp-58h]; mov r8d,0B0h; call`), reading the displacement of `lea rdx,[rcx+30h]`. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and RIP offset is a wildcard."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 03 48 8D 4B ?? 48 8D 57 ?? 41 B8 ?? ?? ?? ?? E8"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 D6 48 89 CE 48 8D 51 ?? 48 8D 4D ?? 41 B8 ?? ?? ?? ?? E8"
      offset = 6
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

entry "CameraInstructionPacket.size" {
  kind     = "field"
  notes    = "sizeof(CameraInstructionPacket): 188h in 26.51 and in 6ad79ee4-17d51891a1119914. It is the size check of the back of the freecam CameraInstructionLayout mirror: CameraInstructionPacket.mCameraInstruction (30h) + the CameraInstruction (150h: mSpline B0h (78h), mTarget 128h (20h), mClear 148h, mDetachFromEntity 14Ah, mRemoveTarget 14Ch) + mSerializationMode (4 bytes at 180h), aligned to 8 = 188h (LeviLamina header sum). One recipe: the packet copy constructor (the anchor of CameraInstructionPacket.mCameraInstruction recipe 0: `lea rax,[rip+vtable]; mov [rbx],rax; lea rcx,[rbx+30h]; lea rdx,[rdi+30h]; mov r8d,0B0h; call`) gives the packet vtable (the RIP target of the lea); its first slot is the deleting destructor (the first virtual of Packet), read with read_pointer; in that function the size goes to operator delete (`mov edx,188h; mov rcx,rsi; call`), and the recipe reads the immediate. The destructor itself is not a unique pattern: 26.51 tests the delete flag with `test edi,edi`, 26.60 with `test dil,1`, and the rest is the shape of every packet destructor with an optional member. Grade: the header sum; on 26.60 the destructor still tests the spline flag at 150h and frees 188h bytes, and the make_shared case of createPacket allocates 198h (the packet and a 10h control block) on both builds. Every displacement, immediate and RIP offset is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the destructor code shape (the delete flag test) and the make_shared code (the zeroing order), so the v1 recipes stopped matching; the size did not change."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 03 48 8D 4B ?? 48 8D 57 ?? 41 B8 ?? ?? ?? ?? E8"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "read_pointer" {}
    step "pattern" {
      bytes = "BA ?? ?? ?? ?? 48 89 F1 E8"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 256
    max = 1024
  }
}
