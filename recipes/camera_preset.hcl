entry "CameraPreset.size" {
  kind     = "field"
  notes    = "sizeof(SharedTypes::v1_26_50::CameraPreset), the stride of CameraPresets::mPresets: 130h in 26.51. One recipe: the camera code that handles a CameraInstructionPacket (0x5679ad0) indexes the preset list with the preset index (`imul rax,rax,130h; add rbx,rax; mov rcx,rbx; call`) and reads the immediate. The count before it divides by 10h and multiplies by the inverse of 13h, the same size. Grade: the LeviLamina header sum (v1_21_90::CameraPreset with the v1_26_50 members). Every displacement and immediate is a wildcard. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the size is still 130h. The preset name compare is inlined after the index (`lea rbx,[rax+rdx]; mov rcx,[rax+rdx+10h]; cmp qword ptr [rax+rdx+18h],10h; mov r8,rbx`), so the imul no longer ends in `add rbx,rax; mov rcx,rbx; call`: recipe 0 is a pattern_any with that shape as alternative 1 (the immediate is read from the imul in both). The function (found by its fingerprint) is the same camera handler: the element count is computed with the multiplicative inverse of 13h (stride 130h divided by 10h) as on the old build. Evidence on 6afa4e47: the imul stride in the index code (the old recipe), and the element count computed with the inverse of 13h in the same function. Class: would have survived with a better recipe (the compiler inlined the name compare)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 69 C0 ?? ?? ?? ?? 48 01 C3 48 89 D9 E8",
        "48 69 ?? ?? ?? ?? ?? 48 8D ?? ?? 48 8B ?? ?? ?? 48 83 7C ?? ?? ?? 49 89 ??",
      ]
    }
    step "operand" {
      index = 2
      kind  = "immediate"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}
