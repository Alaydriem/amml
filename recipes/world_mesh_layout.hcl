entry "ResourceLocation.mFullHash" {
  kind  = "field"
  notes = "Offset of uint64 ResourceLocation::mFullHash, the last member: 30h on 26.51 and 26.60, so the class is 38h bytes (mFileSystem +0, mPath +8, mPathHash +28h, mFullHash +30h). levels builds a ResourceLocation in its own buffer through ResourceLocation.ctor and checks this offset against its mirror, so a class that grows turns the world markers off. ResourceLocation.ctor ends with the two FNV-1a stores: `mov [this+28h],rcx` (mPathHash), then `movzx eax,byte ptr [this]; xor rax,r14; imul rax,rbx; xor rax,rcx; mov [this+30h],rax; mov rax,this; add rsp,..`. The recipe runs inside ResourceLocation.ctor (the pattern matches a second, sibling constructor outside it) and reads the displacement of the last store. Every register, displacement and immediate byte is a wildcard. Range window 28h..60h."

  recipe {
    step "ref" { entry = "ResourceLocation.ctor" }
    step "pattern" {
      bytes  = "48 89 ?? ?? 0F B6 ?? 4C 31 ?? 48 0F AF ?? 48 31 ?? 48 89 ?? ?? 48 89 ?? 48 83 C4"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 40
    max = 96
  }
}

entry "ImageBuffer.mDescription" {
  kind  = "field"
  notes = "Offset of the cg::ImageDescription copy in cg::ImageBuffer: 18h on 26.51 and 26.60, after the blob deleter (+0), the data pointer (+8) and the byte size (+10h); the class is 30h bytes (the 20-byte description, padded). levels builds an ImageBuffer in its own buffer through ImageBuffer.ctorFromDescription and checks this offset against its mirror. The constructor copies the description: `mov eax,[rsi+10h]; mov rcx,[rbp+..]; mov [rcx+28h],eax; movups xmm0,[rsi]; movups [rcx+18h],xmm0`. The recipe runs inside ImageBuffer.ctorFromDescription and reads the displacement of the movups store. Every displacement byte is a wildcard. Range window 10h..40h."

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "8B 46 ?? 48 8B 4D ?? 89 41 ?? 0F 10 06 0F 11 41 ??"
      offset = 13
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "ImageDescription.depth" {
  kind  = "field"
  notes = "Offset of the last member of cg::ImageDescription (the array size or depth): 10h on 26.51 and 26.60, so the description is 20 bytes (width +0, height +4, format +8, colour space +0Ch, image type +0Dh, alpha usage +0Eh, depth +10h). levels fills one for ImageBuffer.ctorFromDescription and checks this offset against its mirror. The recipe runs inside ImageBuffer.ctorFromDescription and reads the source displacement of the first load of the description copy (`mov eax,[rsi+10h]; mov rcx,[rbp+..]; mov [rcx+28h],eax; movups xmm0,[rsi]`). Every displacement byte is a wildcard. Range window 8h..30h."

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" { bytes = "8B 46 ?? 48 8B 4D ?? 89 41 ?? 0F 10 06 0F 11 41 ??" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 48
  }
}

entry "TextureGroup.baseOffset" {
  kind  = "field"
  notes = "Offset of the mce::TextureGroupBase subobject in mce::TextureGroup: 18h on 26.51 and 26.60. TextureGroup.getTexture is a TextureGroupBase member, so each caller adds this offset to the group before the call. Recipe 0 is the call in HudLocatorBarRenderer.render (`add r14,imm8; mov byte ptr [rsp+28h],..; mov qword ptr [rsp+20h],..; mov rcx,r14; lea rdx,[rbp+..]; mov r8,r12; xor r9d,r9d; call`), the same site as recipe 0 of TextureGroup.getTexture. Recipe 1 is the call in the image upload helper (`mov rcx,[rdi+..]; add rcx,imm8; mov byte ptr [rsp+28h],..; ...; call`), the same site as recipe 1 of TextureGroup.getTexture. Both read the immediate of the add. The two sites are in two functions. Every displacement, immediate and branch byte is a wildcard. Range window 8h..40h."

  recipe {
    step "pattern" { bytes = "49 83 C6 ?? C6 44 24 ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 4C 89 F1 48 8D 95 ?? ?? ?? ?? 4D 89 E0 45 31 C9 E8" }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 4F ?? 48 83 C1 ?? C6 44 24 ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 8D 55 ?? 49 89 D8 45 31 C9 E8"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}

entry "ScreenContext.meshContext" {
  kind  = "field"
  notes = "Offset of the mce::MeshContext base in ScreenContext: 10h on 26.51, 26.60 and later, after the 12-byte UIScreenContext base, aligned to 8. Mesh.renderMesh takes the MeshContext&, so its callers load the ScreenContext and add this offset (`mov rdx,[reg+28h]; add rdx,10h`: the ScreenContext is the MinecraftUIRenderContext member at +28h). Recipe 0 is a pattern_any with one alternative for each shape of the draw function that loads the ScreenContext from a stack slot and adds the offset to it (`mov rcx,[rbp+..]; add rcx,imm8; mov [rbp+..],rcx`): alternative 0 is followed by `mov rax,[rax+18h]; mov [rbp+..],rax` (26.63), alternative 1 by `test al,al; mov rdx,[rbp+..]` (1.26.70.26); each matches once on its own build and zero times on the other. It reads the immediate of the add. Every register, displacement and immediate byte is a wildcard. Range window 8h..40h. The entry has one recipe and a range check (the rule for a field). What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the two 26.63 call-site recipes no longer match, because the call arguments are built in another order and the texture argument is another type; the offset is the same. Class: would have survived with a better recipe. Evidence on 1.26.70.26: (1) a pattern drafted from the head of the 26.63 draw function (`mov rsi,[rdx+28h]; mov rax,[r8]; mov [rbp+..],rax; mov r14,[r8+18h]; movss xmm6,[rdx+0B0h]`) matches once on each build, and the function it finds on 1.26.70.26 holds the alternative-1 match, with the ScreenContext taken from the member at +28h as on 26.63; (2) the members of the MeshContext base that resolve on this build lie at the offsets that a base of 10h gives: camera 18h (base + 8), constantBuffers 20h (base + 10h), currentShaderColor 30h (base + 20h), each found from its own code."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 8D ?? ?? ?? ?? 48 83 C1 ?? 48 89 8D ?? ?? ?? ?? 48 8B 40 ?? 48 89 85",
        "48 8B 8D ?? ?? ?? ?? 48 83 C1 ?? 48 89 8D ?? ?? ?? ?? 84 C0 48 8B 95",
      ]
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}

entry "ScreenContext.camera" {
  kind  = "field"
  notes = "Offset in ScreenContext of the `mce::Camera&` member of its MeshContext base, stored as a pointer: 18h on 26.51 and 26.60 (MeshContext at +10h: renderContext +0, camera +8, constantBuffers +10h). A draw function pushes the camera's world matrix stack (+40h of the camera) with MatrixStack.push: `mov rcx,[rax+18h]; add rcx,40h; lea rdx,[rbp+..]; xor r8d,r8d; call`, with rax the ScreenContext. The recipe is the call-site pattern_any of recipe 1 of MatrixStack.push (alternative 1 has the nop that 26.60 adds after the call) and reads the displacement of the first load. Every displacement and immediate byte is a wildcard. Range window 10h..40h."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 48 ?? 48 83 C1 ?? 48 8D 55 ?? 45 31 C0 E8 ?? ?? ?? ?? 48 8B 5D ?? C6 43",
        "48 8B 48 ?? 48 83 C1 ?? 48 8D 55 ?? 45 31 C0 E8 ?? ?? ?? ?? 90 48 8B 5D ?? C6 43",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "ScreenContext.constantBuffers" {
  kind  = "field"
  notes = "Offset in ScreenContext of the `GlobalConstantBuffers&` member of its MeshContext base, stored as a pointer: 20h on 26.51 and 26.60. The shader setup function that calls ActorShaderManager.setEntityConstants (the function of GlobalConstantBuffers.entityConstantBuffer) loads it from the ScreenContext in rcx at its head: `mov r15,[rcx+20h]; mov rax,[r8+..]; movss xmm8,[rax]`; it adds entityConstantBuffer to r15 later. The recipe reads the displacement of that load. It matches once on 26.51 and once on 26.60. Every displacement byte is a wildcard. Range window 10h..40h. The recipe is a pattern_any with one alternative for each shape of the load. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the compiler folded the entityConstantBuffer offset into the load (`mov r14d,imm32; add r14,[rcx+20h]` instead of `mov r15,[rcx+20h]`), so the second alternative reads the same member from the add. Class: would have survived with a better recipe. Evidence on 1.26.70.26: (1) the second alternative, drafted from the 26.63 setup function (the load of the member and the three float reads through the pointer at +240h of the third argument), matches once; (2) the function it matches is the one that calls ActorShaderManager.setEntityConstants, as the 26.63 function does, and it adds the entityConstantBuffer offset to the member."

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 8B 79 ?? 49 8B 80 ?? ?? ?? ?? F3 44 0F 10 00",
        "4C 03 ?? ?? 49 8B 80 ?? ?? ?? ?? F3 44 0F 10 00",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "ScreenContext.currentShaderColor" {
  kind  = "field"
  notes = "Offset in ScreenContext of the `mce::ShaderColor&` member of its MeshContext base, stored as a pointer: 30h on 26.51 and 26.60 (MeshContext at +10h: renderContext +0, camera +8, constantBuffers +10h, constantBufferManager +18h, currentShaderColor +20h). levels writes the colour and the dirty flag through this pointer around each world mesh draw. A flat-colour quad draw (26.60 0x24e5500, 26.51 0x15888b0; the ScreenContext in rdx) stores its colour argument into the shader colour and marks it dirty, then begins a quad on the context's tessellator: `mov rcx,[rdx+30h]; movups xmm0,[rax]; movups [rcx],xmm0; mov byte ptr [rcx+10h],1; mov rdi,[rdx+0B0h]; mov byte ptr [rsp+20h],0; mov rcx,rdi; mov r8b,1; mov r9d,4; call` (the tessellator load is +0B8h on 26.51). The recipe reads the displacement of the first load. It matches once on 26.51 and once on 26.60. Every register, displacement and immediate byte is a wildcard. The pattern ends at the load of the tessellator: the code after it differs between builds. Range window 18h..60h. What changed in 6afa4e47-dadc375797a3d31a (preview 1.26.70.26): the code after the tessellator load builds a std::function before the begin call, so the old tail (`mov byte ptr [rsp+20h],0; mov rcx,rdi; mov r8b,1; mov r9d,4`) is no longer adjacent; the pattern now stops after the tessellator load. Class: would have survived with a better recipe. Evidence on 1.26.70.26: (1) the shorter pattern, drafted from the 26.63 colour store, matches once on both builds; (2) the function that it matches holds the same four vertex calls after the begin call, with the same xmm sources (a run of four `xorps xmm3; mov rcx,rdi; movaps xmm1,..; movaps xmm2,..; call`) that a pattern drafted from the 26.63 vertex run finds once in this build, in this function."

  recipe {
    step "pattern" { bytes = "48 8B ?? ?? 0F 10 ?? 0F 11 ?? C6 ?? ?? ?? 48 8B ?? ?? ?? ?? ??" }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 24
    max = 96
  }
}
