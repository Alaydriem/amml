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
