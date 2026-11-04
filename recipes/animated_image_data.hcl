entry "AnimatedImageData.size" {
  kind     = "field"
  notes    = "sizeof(AnimatedImageData), the stride of SerializedSkinImpl.mSkinAnimatedImages: 40h on 26.60 (mType +0, the mce::Image +8, 30h bytes, then the frame count and the expression). The skin size function (0x298f4e0 on 26.60; the anchor of SerializedSkinImpl.mSkinImage) adds the image byte size of each element and steps to the next (`add rbx,[rax+..]; add rax,imm8; cmp rax,rdx; jne`). One recipe: that pattern, read as the immediate of `add rax,imm8`. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 09 48 8B 99 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 03 99 ?? ?? ?? ?? 48 8B 91 ?? ?? ?? ?? 48 39 D0 74 ?? ?? ?? 48 03 58 ?? 48 83 C0 ?? 48 39 D0 75"
      offset = 45
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 48
    max = 128
  }
}

entry "AnimatedImageData.mImage" {
  kind     = "field"
  notes    = "The offset of AnimatedImageData::mImage (an mce::Image: +0 format, +4 width, +8 height, +0Ch depth, +10h usage, +18h the mce::Blob with its data pointer at +20h and its byte size at +28h) in one element of SerializedSkinImpl.mSkinAnimatedImages: 8h on 26.60. Recipe 0 is the skin size function (0x298f4e0 on 26.60; the anchor of SerializedSkinImpl.mSkinImage), which adds the image byte size of each element (`add rbx,[rax+..]; add rax,imm8; cmp rax,rdx; jne`), read as the displacement of the add, then `add -40` (Image blob 18h + Blob size 10h), the same step as SerializedSkinImpl.mSkinImage. Recipe 1 is the connection request JSON writer (0x4b9b60 on 26.60, found by string_xref of `PremiumSkin`, which only it loads), which writes the type of each element (`mov edx,[rsi]; mov rcx,rbx; call`, then the `Type` key), then the image bytes (`mov rdx,[rsi+..] (the data pointer); mov r8,[rsi+..] (the byte size); mov rcx,rbx; mov r9b,imm; call`), read as the displacement of the size load, then `add -40`; the pattern starts at the type read, because the skin and cape images have the same image-bytes shape. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 89 CF 48 8B 09 48 8B 99 ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 03 99 ?? ?? ?? ?? 48 8B 91 ?? ?? ?? ?? 48 39 D0 74 ?? ?? ?? 48 03 58 ?? 48 83 C0 ?? 48 39 D0 75"
      offset = 41
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -40 }
  }

  recipe {
    step "string_xref" { text = "PremiumSkin" }
    step "pattern" {
      bytes  = "8B 16 48 89 D9 E8 ?? ?? ?? ?? ?? 4C 89 F1 48 8D 15 ?? ?? ?? ?? E8 ?? ?? ?? ?? ?? 48 89 C1 48 89 DA E8 ?? ?? ?? ?? 48 89 D9 E8 ?? ?? ?? ?? 48 8B 56 ?? 4C 8B 46 ??"
      offset = 50
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -40 }
  }

  check "range" {
    min = 0
    max = 64
  }
}

entry "AnimatedImageData.mType" {
  kind     = "field"
  notes    = "The offset of AnimatedImageData::mType (persona::AnimatedTextureType, a 4-byte enum; 1 is the face) in one element of SerializedSkinImpl.mSkinAnimatedImages: 0 on 26.60. The connection request JSON writer (0x4b9b60 on 26.60) writes it first for each element, under the key `Type` (`mov rcx,r14; mov dl,imm; call; mov edx,[rsi]; mov rcx,rbx; call`). The persona piece loops of the same function have that shape too, so the pattern starts at the load of the vector end and runs through the loop head (`mov r12,[rax+..]; lea r14,[rbp+..]; lea rdi,[rbp+..]; lea r13,[rip+..]; lea r15,[rip+..]; cmp rsi,r12; je; nop`). One recipe: string_xref of `PremiumSkin` (three functions load `AnimatedImageData`; this writer is the only one that loads `PremiumSkin`), then that pattern in the function, read as the displacement of `mov edx,[rsi]` (an operand without a displacement byte reads as 0). Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "string_xref" { text = "PremiumSkin" }
    step "pattern" {
      bytes  = "4C 8B A0 ?? ?? ?? ?? 4C 8D 75 ?? 48 8D 7D ?? 4C 8D 2D ?? ?? ?? ?? 4C 8D 3D ?? ?? ?? ?? 4C 39 E6 0F 84 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 4C 89 F1 B2 ?? E8 ?? ?? ?? ?? 8B 16"
      offset = 56
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 32
  }
}
