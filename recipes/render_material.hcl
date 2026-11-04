entry "RenderMaterial.mEnableBlend" {
  kind     = "field"
  notes    = "Offset of the `bool` enableBlend in mce::RenderMaterial: 185h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) turns the state flags word of the material definition (def+0A8h) into RenderMaterial members: bit 0 gives depthTestEnabled (+188h), bit 5 depthWriteMask (+193h), bit 8 stencilTestEnabled (+189h), bit 3 and bit 4 the color write mask (+184h), and bit 1 enableBlend: `shr al,1; and al,1; mov [r8+185h],al`, right after the color mask store `mov [r8+184h],cl`. The draw-state builder 0x66b1f30 reads it back: `cmp byte ptr [rdx+185h],1; jne` decides if the blend factors at +180h and +181h are used, and a value other than 1 gives the opaque blend mode. Recipe 0 is the parser site: `test al,imm; movzx ecx,cl; cmovne ecx,edx; mov [r8+..],cl; shr al,1; and al,imm; mov [r8+..],al`; it reads the displacement of operand 0 of the last store at offset 19. Recipe 1 is the reader site: `mov rcx,[rdx]; mov rdx,[rcx+..]; xor r11d,r11d; cmp byte ptr [rdx+..],imm; jne`; it reads the displacement of operand 0 of the cmp at offset 10. The two recipes are in two functions (the parser and the draw-state builder) and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry."

  recipe {
    step "pattern" {
      bytes  = "A8 ?? 0F B6 C9 0F 45 CA 41 88 88 ?? ?? ?? ?? D0 E8 24 ?? 41 88 80 ?? ?? ?? ??"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 0A 48 8B 51 ?? 45 31 DB 80 BA ?? ?? ?? ?? ?? 0F 85"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "RenderMaterial.mDepthWriteMask" {
  kind     = "field"
  notes    = "Offset of the `bool` depthWriteMask in mce::RenderMaterial: 193h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) sets it from bit 5 of the state flags word of the material definition (def+0A8h), right after depthTestEnabled from bit 0: `not cl; and cl,1; mov [rdx+188h],cl; test al,20h; sete [rdx+193h]; shr eax,8; and al,1; mov [rdx+189h],al`. The draw-state builder 0x66b1f30 reads it back: `movzx ecx,byte ptr [rdx+188h]; mov [rax+0Ch],cl; cmp byte ptr [rdx+193h],0; setne r10b; and r10b,cl` (depth write is on only when the depth test is on too). Recipe 0 is the reader site: `movzx ecx,byte ptr [rdx+..]; mov [rax+..],cl; cmp byte ptr [rdx+..],imm; setne r10b; and r10b,cl`; it reads the displacement of operand 0 of the cmp at offset 10. Recipe 1 is the parser site: `mov [rdx+..],cl; test al,imm; sete [rdx+..]; shr eax,imm; and al,imm; mov [rdx+..],al`; it reads the displacement of operand 0 of the sete at offset 8. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry."

  recipe {
    step "pattern" {
      bytes  = "0F B6 8A ?? ?? ?? ?? 88 48 ?? 80 BA ?? ?? ?? ?? ?? 41 0F 95 C2 41 20 CA"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "88 8A ?? ?? ?? ?? A8 ?? 0F 94 82 ?? ?? ?? ?? C1 E8 ?? 24 ?? 88 82"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 256
    max = 512
  }
}

entry "RenderMaterial.mBlendSource" {
  kind     = "field"
  notes    = "Offset of the one-byte mce::BlendTarget blendSource in mce::RenderMaterial: 180h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) reads it from the material definition key \"blendSrc\": `lea r8,[r12+180h]; lea rax,[rip+..] (\"blendSrc\"); mov [rbp+50h],rax; mov qword ptr [rbp+58h],8; lea rdx,[rbp+50h]; mov rsi,[rbp+80h]; ...; call 0x4df6090`, the enum reader that writes through r8. The next call does the same for \"blendDst\" into +181h. The draw-state builder 0x66b1f30 reads it back only when enableBlend (+185h) is 1: `movzx ecx,byte ptr [rdx+180h]; movzx r9d,byte ptr [rdx+181h]; mov ebx,ecx; xor bl,3` (the first test of the blend table is source One, destination Zero). Recipe 0 is the reader site: `movzx ecx,byte ptr [rdx+..]; movzx r9d,byte ptr [rdx+..]; mov ebx,ecx; xor bl,imm`; it reads the displacement of operand 1 of the first movzx at offset 0. Recipe 1 is the parser site: `lea r8,[r12+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov qword ptr [rbp+..],imm; lea rdx,[rbp+..]; mov rsi,[rbp+..]`; it reads the displacement of operand 1 of the first lea at offset 0. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry."

  recipe {
    step "pattern" {
      bytes  = "0F B6 8A ?? ?? ?? ?? 44 0F B6 8A ?? ?? ?? ?? 89 CB 80 F3 ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 55 ?? 48 8B B5"
      offset = 0
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

entry "RenderMaterial.mBlendDestination" {
  kind     = "field"
  notes    = "Offset of the one-byte mce::BlendTarget blendDestination in mce::RenderMaterial: 181h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) reads it from the material definition key \"blendDst\", right after blendSrc: `mov bl,al; lea r8,[r12+181h]; lea rax,[rip+..] (\"blendDst\"); mov [rbp],rax; mov qword ptr [rbp+8],8; mov rdx,rbp; mov rcx,rsi; mov rdi,r8; call 0x4df6090`, the enum reader that writes through r8. The draw-state builder 0x66b1f30 reads it back only when enableBlend (+185h) is 1: `movzx ecx,byte ptr [rdx+180h]; movzx r9d,byte ptr [rdx+181h]; mov ebx,ecx; xor bl,3; mov r10d,r9d; xor r10b,2` (the first test of the blend table is source One, destination Zero). Recipe 0 is the reader site: the same pattern as recipe 0 of mBlendSource; it reads the displacement of operand 1 of the second movzx at offset 7. Recipe 1 is the parser site: `mov bl,al; lea r8,[r12+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov qword ptr [rbp+..],imm; mov rdx,rbp; mov rcx,rsi; mov rdi,r8`; it reads the displacement of operand 1 of the lea at offset 2. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry."

  recipe {
    step "pattern" {
      bytes  = "0F B6 8A ?? ?? ?? ?? 44 0F B6 8A ?? ?? ?? ?? 89 CB 80 F3 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "88 C3 4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 EA 48 89 F1 4C 89 C7"
      offset = 2
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
