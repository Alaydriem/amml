entry "CameraInstruction.mSpline" {
  kind     = "field"
  notes    = "Offset of `mSpline` in CameraInstruction: B0h in 26.51. It is the size check of the front of the freecam CameraInstructionLayout mirror (mAttachToEntity 0 (18h), mFade 18h (24h), mFieldOfView 3Ch (14h), mSet 50h (60h: SetInstruction 5Ch and its engaged flag at 5Ch), then mSpline B0h; LeviLamina header sum): the game copies the trivially copyable front of the instruction, up to mSpline, with one memcpy of this length. Recipe 0 is the packet copy constructor (0x84ede70: `lea rax,[rip+vtable]; mov [rbx],rax; lea rcx,[rbx+30h]; lea rdx,[rdi+30h]; mov r8d,0B0h; call`); recipe 1 is the camera system that handles the packet (0x5679ad0: `mov r14,rdx; mov rsi,rcx; lea rdx,[rcx+30h]; lea rcx,[rbp-58h]; mov r8d,0B0h; call`). Each reads the immediate of `mov r8d`. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and RIP offset is a wildcard."

  recipe {
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 03 48 8D 4B ?? 48 8D 57 ?? 41 B8 ?? ?? ?? ?? E8"
      offset = 18
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "49 89 D6 48 89 CE 48 8D 51 ?? 48 8D 4D ?? 41 B8 ?? ?? ?? ?? E8"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 64
    max = 512
  }
}
