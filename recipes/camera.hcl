entry "Camera.mForward" {
  kind     = "field"
  notes    = "Anchor of the mce::Camera mirror (amml-game CameraLayout: the three 64-byte MatrixStacks at 0, 40h and 80h, mInverseViewMatrix C0h, mRight 100h, mUp 10Ch, mForward 118h, mPosition 124h); CameraWriter and Projection refuse a build whose value is not 118h. Offset of `mForward` (glm::vec3) in mce::Camera: 0x118 in 26.51 and 26.60. LeviLamina header sum (deps/renderer/Camera.h, no vtable): viewMatrixStack 64, worldMatrixStack 64, projectionMatrixStack 64, mInverseViewMatrix 64 (at 0xC0), mRight 12 (0x100), mUp 12 (0x10C), mForward at 0x118, then mPosition (0x124). Recipe 0 is mce::Camera::updateViewMatrixDependencies (the function that stores the inverse of the top view matrix at camera+0C0h and its columns at 100h, 10Ch, 118h and 124h; LevelRendererCamera::preRenderUpdate calls it on the camera of the ScreenContext, `mov rcx,[rbx+18h]; call`, right after it stores two floats into its own members): the third column is negated with the sign mask (`xorps`) and stored as mForward; the recipe reads the displacement of the first store of the negated column (operand 0). It is a pattern_any with one alternative per build shape: 26.51 negates x, y and z in three registers and stores them with three `movss [rsi+disp]` (alternative 0, from the `xorps xmm6` to the `mov rax,[rsi+60h]; mov rcx,[rsi+58h]` that follows the six stores); 26.60 negates x and y in one register and stores them with `movlps [rsi+disp]` (alternative 1, then the z, position and w stores with a `shufps`). Recipe 1 is LevelRendererCamera::preRenderUpdate (the first call of LevelRendererPlayer::preRenderUpdate, slot 22 of the LevelRendererPlayer vtable in 26.51): `mov rax,[rbx+18h]` (the camera of the ScreenContext), then the forward vector is copied to a stack vec3; the recipe reads the displacement of the first load (operand 1). Again a pattern_any: 26.51 copies x, y, z with three `movss` and passes the vec3 by pointer to a call (`lea r8,[rbp-..]; movss xmm3,[rip+..]; call`); 26.60 copies x and y with one `movsd`, z with `movss`, then reads a clamped index (`mov eax,[rbx+disp]; xor edx,edx; cmp al,imm; cmovb edx,eax`). The two recipes are in two functions and share no pattern bytes. Every register (ModRM) byte, displacement, immediate and RIP offset is a wildcard, except the `xor edx,edx` and `cmovb edx,eax` register bytes of recipe 1 alternative 1. What changed in 6ad79ee4-17d51891a1119914: the compiler vectorised both functions (the matrix copy in updateViewMatrixDependencies uses movsd/movups/movlps, and the forward copy in preRenderUpdate uses movsd), so both old patterns matched nothing; the Camera layout did not change. Class: real code change (code shape, not layout). Evidence for 0x118 on 26.60: (1) the 26.60 LevelRendererCamera::preRenderUpdate is the function whose body lines up with the 26.51 one instruction for instruction at the start (the two float stores, `mov rcx,[rbx+18h]; call`, `mov r12,[rbx+18h]`), and its first call is the 26.60 updateViewMatrixDependencies; in that function the full inverse matrix is stored at 0C0h..0FFh, the right column at 100h, the up column at 10Ch, the sign-masked third column at 118h and the translation at 124h, the same column order as 26.51; (2) the same 26.60 preRenderUpdate copies the camera members 110h, 114h, 118h, 11Ch, 120h, 124h, 128h one by one through `[r12+disp]` into its own members, in the same run as 26.51, and its forward copy reads the vec3 at 118h."

  recipe {
    step "pattern_any" {
      patterns = [
        "0F 57 ?? F3 0F 11 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? ?? ?? ?? F3 44 0F 11 ?? ?? ?? ?? ?? F3 44 0F 11 ?? ?? ?? ?? ?? F3 44 0F 11 ?? ?? ?? ?? ?? F3 44 0F 11 ?? ?? ?? ?? ?? 48 8B ?? ?? 48 8B ?? ??",
        "0F 57 ?? 0F 13 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? ?? ?? ?? 0F C6 ?? ?? F3 0F 11 ?? ?? ?? ?? ?? F3 44 0F 11 ?? ?? ?? ?? ??",
      ]
      offset = 3
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B ?? ?? F3 0F 10 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? F3 0F 10 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? F3 0F 10 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? F3 0F 10 ?? ?? F3 0F 11 ?? 24 ?? 4C 8D ?? ?? F3 0F 10 ?? ?? ?? ?? ?? E8",
        "48 8B ?? ?? F2 0F 10 ?? ?? ?? ?? ?? F2 0F 11 ?? ?? F3 0F 10 ?? ?? ?? ?? ?? F3 0F 11 ?? ?? 8B ?? ?? 31 D2 3C ?? 0F 42 D0",
      ]
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "Camera.updateViewMatrixDependencies" {
  kind      = "function"
  msvc_name = "?updateViewMatrixDependencies@Camera@mce@@QEAAXXZ"
  notes     = "Address of `void mce::Camera::updateViewMatrixDependencies()`: 0x48cb940 in 26.51 (this in rcx). It takes the top of the view stack, stores its inverse at C0h, the columns as mRight 100h, mUp 10Ch, mForward 118h (negated) and mPosition 124h, then builds the view-projection matrix from the projection and world tops and stores it at 140h, and updates the frustum. Recipe 0 is the head of its body (`mov rsi,rcx; mov rax,[rcx+20h]; mov rcx,[rcx+18h]; add rax,rcx; dec rax; mov rcx,[rsi+8]; mov rdx,[rsi+10h]; dec rdx; and rdx,rax; mov rdx,[rcx+rdx*8]; lea rcx,[rbp-18h]; call`, the deque back() of the view stack and the inverse call), then function_start_of. Recipe 1 is the second call of LevelRendererCamera::preRenderUpdate, on its own mWorldSpaceCamera (`movss [r12+3Ch],xmm1; mov rcx,r14; call`, after the translation of the view top), read as a branch target. The first call of preRenderUpdate (`mov rcx,[rbx+18h]; call`, on the camera of the ScreenContext) goes to the same address. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Recipe 1 is now the first call in LevelRendererCamera::preRenderUpdate (`ucomiss xmm1,xmm2; jbe; movss [rsi+..],xmm0; movss [rsi+..],xmm1; mov rcx,[rbx+18h]; call`, the camera of the ScreenContext), the same shape on both builds. What changed in 6ad79ee4-17d51891a1119914: the second call is now preceded by a vectorised store (`movups [r12+30h],xmm3`), so the old recipe 1 matched nothing. Evidence on 26.60: recipe 0 (the body) and recipe 1 (the call, drafted from the 26.51 preRenderUpdate) reach the same function; preRenderUpdate calls it twice, as on 26.51."

  recipe {
    step "pattern" {
      bytes = "48 89 CE 48 8B 41 ?? 48 8B 49 ?? 48 01 C8 48 FF C8 48 8B 4E ?? 48 8B 56 ?? 48 FF CA 48 21 C2 48 8B 14 D1 48 8D 4D ?? E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "0F 2E CA 76 ?? F3 0F 11 86 ?? ?? ?? ?? F3 0F 11 8E ?? ?? ?? ?? 48 8B 4B ?? E8"
      offset = 25
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
