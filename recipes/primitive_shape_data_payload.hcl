entry "PrimitiveShapeDataPayload.size" {
  kind     = "field"
  notes    = "sizeof(PrimitiveShapeDataPayload), the stride of PrimitiveShapesPacket::mShapes: B8h in 26.51 (the static_assert of the C++ build: 184; LeviLamina header sum: mNetworkId 0 (8), mShapeType 8 (2), mLocation Ch (10h), mRotation 1Ch (10h), mScale 2Ch (8), mColor 34h (14h), mTimeLeftTotalSec 48h (8), mDimensionId 50h (8), mAttachedToId 58h (10h, align 8), mMaxRenderDistance 68h (8), mExtraDataPayload 70h (48h, align 8) = B8h). One recipe: the shape system (0x6686330) steps through the vector (`add rdi,0B8h; cmp rdi,r15; je`); the recipe finds the vector load first (`mov rdi,[rdx+30h]; mov r15,[rdx+38h]; cmp rdi,r15; je`, unique) and then the step in the same function, and reads the immediate. Grade: the header sum and the C++ static_assert. Every displacement, immediate and branch byte is a wildcard. What changed in 6ad79ee4-17d51891a1119914: the shape system keeps the vector end in r14 instead of r15, and another function walks a vector of another packet at +20h/+28h with the old pattern, so the old pattern matched twice; the pattern now wildcards the end register and adds the start of the loop setup (`mov rsi,rcx; lea rbx,[rbp-20h]; xorps xmm6,xmm6; jmp`), and the step pattern wildcards the end register. The value did not change (30h, B8h). Evidence on 26.60, from the 26.51 shape system: (1) a draft of its existing-shape update (`cmp byte ptr [rdi+9],1; jne; mov rax,[rcx]; mov rax,[rax+10h]; mov rdx,rdi; call`) matches once on 26.60, in the function that starts with the anchor; (2) the whole function lines up instruction for instruction with the 26.51 one (vector loads at 30h/38h, `add rdi,0B8h` step, the 20h-byte shape list walk)."

  recipe {
    step "pattern" {
      bytes = "48 8B 7A ?? 4C 8B ?? ?? 4C 39 ?? 0F 84 ?? ?? ?? ?? 48 89 CE 48 8D 5D ?? 0F 57 F6 EB"
    }
    step "pattern" {
      bytes  = "48 81 C7 ?? ?? ?? ?? 4C 39 ?? 0F 84"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 128
    max = 256
  }
}
