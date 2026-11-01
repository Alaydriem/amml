entry "Timer.advanceTime" {
  kind      = "function"
  hooked    = true
  msvc_name = "?advanceTime@Timer@@QEAAXM@Z"
  notes     = "void Timer::advanceTime(float preferredFrameStep): 0x47322e0 in 26.51, 0x5a3b180 in 26.60. rcx = the Timer (80h bytes), xmm1 = the step; it returns void. In 26.60 it takes a third argument, `bool forceOnlySingleTick` in r8b (kept in ebx): when it is set and more than one tick is due, the extra ticks go to mOverflowTime and mTicks is 1. The msvc_name is the 26.51 signature; the 26.60 name would be `?advanceTime@Timer@@QEAAXM_N@Z`. A hook must pass r8 on to the original. It advances every Timer, not only the client pair (MinecraftGame.mSimTimer and mRealTimer); a hook compares `this` with those two pointers. Found by its shape: it is the only function under 700h bytes that calls the std::function at [this+78h] and touches +4, +8, +0Ch, +10h, +28h, +30h, +38h and +3Ch. Recipe 0 is the mSteppingTick test: `movss xmm0,[rcx+disp]; xorps xmm1,xmm1; ucomiss xmm0,xmm1; jae; mov rcx,[rsi+disp]; test rcx,rcx; je rel32`, then `function_start_of`. Recipe 1 is the first of the two direct calls in the MinecraftGame timer function (see MinecraftGame.mSimTimer recipe 1): `mov rcx,[reg+sim]; movss xmm6,[rip+..]; movaps xmm1,xmm6; call`, then the same for mRealTimer. 26.60 adds `xor r8d,r8d` before each call, so the recipe is a `pattern_any` with one alternative for each form; the 26.51 alternative starts with three wildcards so that the first call is at offset 21 in both. It reads the branch target. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60 (a real code change): the third argument; the layout (size 80h) did not change."

  recipe {
    step "pattern" {
      bytes = "F3 0F 10 41 ?? 0F 57 C9 0F 2E C1 73 ?? 48 8B 4E ?? 48 85 C9 0F 84"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "?? ?? ?? 48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE E8",
        "48 8B ?? ?? ?? ?? ?? F3 0F 10 35 ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8 ?? ?? ?? ?? 48 8B ?? ?? ?? ?? ?? 0F 28 CE 45 31 C0 E8",
      ]
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
