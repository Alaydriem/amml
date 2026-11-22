entry "ParticleSystemEngine.tick" {
  kind      = "function"
  hooked    = true
  msvc_name = "?tick@ParticleSystemEngine@@QEAAXM_N@Z"
  notes     = "void ParticleSystemEngine::tick(float a, bool paused): 0x216d250 in 26.51, 0x6b59aa0 in 26.60. rcx = the engine, xmm1 = the frame alpha, r8b = the paused flag; it returns void. The body reads steady_clock::now(), uses mLastTickTimeValid (bool, +0B8h) and mLastTickTime (+0C0h) for the step, fills the light texture (+0E8h, data at +0F0h) and at the end sets +0B8h to 1 and stores the time at +0C0h; mTotalParticleCount is at +10F8h. These fields are the same in both builds. One direct caller in each build (`movss xmm1,[timer+8]; mov rcx,rdi; mov r8d,eax; call`). The exe has no RTTI for the class. Recipe 0 is the head: `mov ebx,r8d; movaps xmm6,xmm1; mov rsi,rcx`, then the first two global clears (`mov qword ptr [rip+..],imm`), then `function_start_of`. Recipe 1 shares no bytes with it: the time delta, `cmp byte ptr [rsi+disp],imm; je rel32; mov rax,rdx; sub rax,[rsi+disp]`, then `function_start_of`. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start (it cannot fail after `function_start_of`; the two agreeing recipes carry the entry). What changed in 26.60: the address only; the code is the same."

  recipe {
    step "pattern" {
      bytes = "44 89 C3 0F 28 F1 48 89 CE 48 C7 05 ?? ?? ?? ?? ?? ?? ?? ?? 48 C7 05 ?? ?? ?? ?? ?? ?? ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "80 BE ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 89 D0 48 2B 86"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "ParticleSystemEngine.mValid" {
  kind  = "field"
  notes = "Offset of the bool that says mLastTickTime holds a time (mLastTickTimeValid) in ParticleSystemEngine: B8h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). ParticleSystemEngine.tick tests it before the time delta and sets it to 1 at its end (`mov byte ptr [rsi+0B8h],1; mov [rsi+0C0h],rdx`, 26.63 0x6b5ab4f, 70.26 0x3eed7d8). Recipe 0 is the delta in tick (the anchor of tick recipe 1): `cmp byte ptr [rsi+..],imm; je rel32; mov rax,rdx; sub rax,[rsi+..]` (26.63 0x6b5a533, 70.26 0x3eed174), scanned inside tick; offset 0, the displacement of operand 0 of the cmp. Recipe 1 is the constructor (26.63 0x6b57f60, 70.26 0x3eeab30; it stores the members that tick uses: B8h, C0h, E8h, the 1000h-byte block at F0h, 10F0h, 10F4h, 10F8h, 1100h): `mov byte ptr [r12+..],imm; mov qword ptr [r12+..],imm; mov [r12+..],rdi; mov [r12+..],rsi; xorps xmm0,xmm0; movups [r12+..],xmm0` (26.63 0x6b58187, 70.26 0x3eead57), unique in the image; offset 0, the displacement of operand 0 of the first store. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 80h..100h. Class: new entry; the same value on both builds and in the 26.51 / 26.60 notes of tick."

  recipe {
    step "ref" { entry = "ParticleSystemEngine.tick" }
    step "pattern" { bytes = "80 BE ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 89 D0 48 2B 86 ?? ?? ?? ??" }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" { bytes = "41 C6 84 24 ?? ?? ?? ?? ?? 49 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 49 89 BC 24 ?? ?? ?? ?? 49 89 B4 24 ?? ?? ?? ?? 0F 57 C0 41 0F 11 84 24" }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}

entry "ParticleSystemEngine.mLastTickTime" {
  kind  = "field"
  notes = "Offset of mLastTickTime (a steady_clock time point, a signed 64-bit count of nanoseconds) in ParticleSystemEngine: C0h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). ParticleSystemEngine.tick computes dt = now - mLastTickTime and stores now there (`sub rax,[rsi+0C0h]; mov [rbp+..],rax; mov [rsi+0C0h],rdx`), and stores now again at its end next to mValid. Recipe 0 is the delta in tick: the pattern of ParticleSystemEngine.mValid recipe 0, offset 16, the displacement of operand 1 of the sub (26.63 0x6b5a543, 70.26 0x3eed184). Recipe 1 is the constructor: the pattern of ParticleSystemEngine.mValid recipe 1, offset 9, the displacement of operand 0 of `mov qword ptr [r12+..],0` (26.63 0x6b58190, 70.26 0x3eead60). The two recipes are in two functions (tick and the constructor). Every displacement, immediate and branch byte is a wildcard. Range check 80h..100h. Class: new entry; the same value on both builds."

  recipe {
    step "ref" { entry = "ParticleSystemEngine.tick" }
    step "pattern" {
      bytes  = "80 BE ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 89 D0 48 2B 86 ?? ?? ?? ??"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "41 C6 84 24 ?? ?? ?? ?? ?? 49 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 49 89 BC 24 ?? ?? ?? ?? 49 89 B4 24 ?? ?? ?? ?? 0F 57 C0 41 0F 11 84 24"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}

entry "ParticleSystemEngine.mParticleCount" {
  kind  = "field"
  notes = "Offset of the total particle count (mTotalParticleCount, 64-bit) in ParticleSystemEngine: 10F8h on 6aed1906-b127418e74d2f2f9 (26.63) and on 6afa4e47-dadc375797a3d31a (70.26). ParticleSystemEngine.tick sums the particles of every emitter in r13 and stores the sum after the loop. Recipe 0 is that store: `mov rax,[rsi+..]; cmp rax,[rsi+..]; je; mov [rsi+..],rax; mov [rsi+..],r13` (26.63 0x6b5aa6a, 70.26 0x3eed73a), scanned inside tick; offset 14, the displacement of operand 0 of the last store (26.63 0x6b5aa78, 70.26 0x3eed748). Recipe 1 is the constructor: `mov [r12+..],r15d; mov byte ptr [r12+..],imm; mov qword ptr [r12+..],imm; lea rcx,[r12+..]` (26.63 0x6b58204, 70.26 0x3eeadd4), unique in the image; offset 17, the displacement of operand 0 of the qword clear (26.63 0x6b58215, 70.26 0x3eeade5). The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range check 1000h..1200h. Class: new entry; the same value on both builds."

  recipe {
    step "ref" { entry = "ParticleSystemEngine.tick" }
    step "pattern" {
      bytes  = "48 8B 46 ?? 48 3B 46 ?? 74 ?? 48 89 46 ?? 4C 89 AE ?? ?? ?? ??"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "45 89 BC 24 ?? ?? ?? ?? 41 C6 84 24 ?? ?? ?? ?? ?? 49 C7 84 24 ?? ?? ?? ?? ?? ?? ?? ?? 49 8D 8C 24"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 4096
    max = 4608
  }
}
