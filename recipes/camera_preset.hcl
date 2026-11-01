entry "CameraPreset.size" {
  kind     = "field"
  notes    = "sizeof(SharedTypes::v1_26_50::CameraPreset), the stride of CameraPresets::mPresets: 130h in 26.51. One recipe: the camera code that handles a CameraInstructionPacket (0x5679ad0) indexes the preset list with the preset index (`imul rax,rax,130h; add rbx,rax; mov rcx,rbx; call`) and reads the immediate. The count before it divides by 10h and multiplies by the inverse of 13h, the same size. Grade: the LeviLamina header sum (v1_21_90::CameraPreset with the v1_26_50 members). Every displacement and immediate is a wildcard."

  recipe {
    step "pattern" {
      bytes = "48 69 C0 ?? ?? ?? ?? 48 01 C3 48 89 D9 E8"
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
