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
  notes = "Offset of the mce::MeshContext base in ScreenContext: 10h on 26.51 and 26.60, after the 12-byte UIScreenContext base, aligned to 8. Mesh.renderMesh takes the MeshContext&, so its callers load the ScreenContext and add this offset (`mov rdx,[reg+28h]; add rdx,10h`). Recipe 0 (26.60) is a call of Mesh.renderMesh with a material and a texture variant from rip-relative globals (`mov rdx,[rsi+..]; add rdx,imm8; lea rax,[rbp+..]; mov [rsp+30h],rax; mov qword ptr [rsp+38h],0; mov dword ptr [rsp+28h],0; mov dword ptr [rsp+20h],0; lea rcx,[rip+..]; lea r8,[rip+..]; lea r9,[rbp+..]; call`). Recipe 1 (26.51 and 26.60) is a call that passes a material at +600h of its object (`mov rdx,[reg+..]; lea r8,[rbx+..]; add rdx,imm8; lea rax,[rbp+..]; ...; lea r9,[rbp+..]; mov rcx,r15; call`); the first instruction's REX and ModRM bytes are wildcards (rdi on 26.51, r14 on 26.60). Both read the immediate of the add. The two sites are in two functions. Every displacement and immediate byte is a wildcard. Range window 8h..40h."

  recipe {
    step "pattern" {
      bytes  = "48 8B 56 ?? 48 83 C2 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 4C 8D 4D ?? E8"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "?? 8B ?? ?? 4C 8D 83 ?? ?? ?? ?? 48 83 C2 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? C7 44 24 ?? ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? 4C 89 F9 E8"
      offset = 11
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
  notes = "Offset in ScreenContext of the `GlobalConstantBuffers&` member of its MeshContext base, stored as a pointer: 20h on 26.51 and 26.60. The shader setup function that calls ActorShaderManager.setEntityConstants (the function of GlobalConstantBuffers.entityConstantBuffer) loads it from the ScreenContext in rcx at its head: `mov r15,[rcx+20h]; mov rax,[r8+..]; movss xmm8,[rax]`; it adds entityConstantBuffer to r15 later. The recipe reads the displacement of that load. It matches once on 26.51 and once on 26.60. Every displacement byte is a wildcard. Range window 10h..40h."

  recipe {
    step "pattern" { bytes = "4C 8B 79 ?? 49 8B 80 ?? ?? ?? ?? F3 44 0F 10 00" }
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
  notes = "Offset in ScreenContext of the `mce::ShaderColor&` member of its MeshContext base, stored as a pointer: 30h on 26.51 and 26.60 (MeshContext at +10h: renderContext +0, camera +8, constantBuffers +10h, constantBufferManager +18h, currentShaderColor +20h). levels writes the colour and the dirty flag through this pointer around each world mesh draw. A flat-colour quad draw (26.60 0x24e5500, 26.51 0x15888b0; the ScreenContext in rdx) stores its colour argument into the shader colour and marks it dirty, then begins a quad on the context's tessellator: `mov rcx,[rdx+30h]; movups xmm0,[rax]; movups [rcx],xmm0; mov byte ptr [rcx+10h],1; mov rdi,[rdx+0B0h]; mov byte ptr [rsp+20h],0; mov rcx,rdi; mov r8b,1; mov r9d,4; call` (the tessellator load is +0B8h on 26.51). The recipe reads the displacement of the first load. It matches once on 26.51 and once on 26.60. Every register, displacement and immediate byte is a wildcard. Range window 18h..60h."

  recipe {
    step "pattern" { bytes = "48 8B ?? ?? 0F 10 ?? 0F 11 ?? C6 ?? ?? ?? 48 8B ?? ?? ?? ?? ?? C6 44 24 ?? ?? 48 89 ?? 41 B0 ?? 41 B9 ?? ?? ?? ?? E8" }
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
