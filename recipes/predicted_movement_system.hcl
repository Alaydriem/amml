entry "PredictedMovementSystem.tickPredictedMovement" {
  kind      = "function"
  hooked    = true
  msvc_name = "?_tickPredictedMovement@PredictedMovementSystem@@YAXAEAVPredictedMovementComponent@@AEAUStateVectorComponent@@AEAVActorOwnerComponent@@V?$Optional@$$CBUMobFlagComponent@@@@V?$Optional@$$CBUVehicleInputIntentComponent@@@@V?$Optional@UMovementInterpolatorComponent@@@@V?$Optional@$$CBUActorRotationComponent@@@@V?$Optional@$$CBUOnGroundFlagComponent@@@@V?$Optional@$$CBUActorHeadRotationComponent@@@@AEBUSystemUniforms@1@@Z"
  notes     = "static void PredictedMovementSystem::_tickPredictedMovement(PredictedMovementComponent&, StateVectorComponent const&, ActorOwnerComponent&, Optional<...>, ... , SystemUniforms const&) (ten arguments): 0x7caecc0 in 26.51, 0x7b02fc0 in 26.60. rcx = the PredictedMovementComponent, rdx = the StateVectorComponent, r8 = the ActorOwnerComponent, r9 = the first Optional; the next five arguments are pointer-sized values on the stack, and argument 10 (at entry+50h) is the SystemUniforms (mNow, a steady_clock time in ns, at +8). It returns void. A hook passes rcx..r9 and the six stack slots on unchanged. It is a real function: it has no direct caller, and the system tick wrapper (0x7cb9a20 in 26.51, 0x7b32520 in 26.60) takes its address with `lea rax,[rip+..]` right after it stores now() into the local SystemUniforms; so a hook on the address works. Found through the debug format string of PredictionDbgData (`[type: {}; fromItem: {}; ...]`): its only user has one caller, and that caller's only caller is this function. Recipe 0 is the argument moves after the prologue: `mov r13,r9; mov r15,r8; mov rbx,rdx; mov rdi,rcx; mov rsi,[rsp+..]; mov r12,[rsp+..]; mov rax,[rsp+..]; mov rcx,[rax]; mov edx,[rax+disp]`, then `function_start_of`. Recipe 1 is the `lea` in the tick wrapper, read as a RIP target at offset 7. The bytes around it differ: 26.51 `add rdx,rcx; mov [rbp+..],rdx; lea rax,[rip+..]; mov [rbp+disp32],rax; mov qword ptr [rbp+disp32],0; mov rax,[rbp+..]`, 26.60 `add rdx,rcx; mov [rbp+..],rdx; lea rax,[rip+..]; mov [rbp+disp8],rax; mov qword ptr [rbp+disp8],0; mov rcx,rbx; call`. The short 26.60 form (up to the first store) has 5 matches (other system wrappers); with the store of 0 and `mov rcx,rbx; call` it is unique. So the recipe is a `pattern_any` with these two alternatives; each is unique on its build and has no match on the other. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60: the address and the wrapper's frame offsets; the function's own code did not change."

  recipe {
    step "pattern" {
      bytes = "4D 89 CD 4D 89 C7 48 89 D3 48 89 CF 48 8B B4 24 ?? ?? ?? ?? 4C 8B A4 24 ?? ?? ?? ?? 48 8B 84 24 ?? ?? ?? ?? 48 8B 08 8B 50 ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 01 CA 48 89 55 ?? 48 8D 05 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 85",
        "48 01 CA 48 89 55 ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 D9 E8",
      ]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
