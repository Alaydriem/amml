entry "RenderMaterial.mEnableBlend" {
  kind     = "field"
  notes    = "Offset of the `bool` enableBlend in mce::RenderMaterial: 185h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) turns the state flags word of the material definition (def+0A8h) into RenderMaterial members: bit 0 gives depthTestEnabled (+188h), bit 5 depthWriteMask (+193h), bit 8 stencilTestEnabled (+189h), bit 3 and bit 4 the color write mask (+184h), and bit 1 enableBlend: `shr al,1; and al,1; mov [r8+185h],al`, right after the color mask store `mov [r8+184h],cl`. The draw-state builder 0x66b1f30 reads it back: `cmp byte ptr [rdx+185h],1; jne` decides if the blend factors at +180h and +181h are used, and a value other than 1 gives the opaque blend mode. Recipe 0 is the parser site: `test al,imm; movzx ecx,cl; cmovne ecx,edx; mov [r8+..],cl; shr al,1; and al,imm; mov [r8+..],al`; it reads the displacement of operand 0 of the last store at offset 19. Recipe 1 is the reader site, one pattern_any with one alternative per code shape: `cmp byte ptr [reg+..],imm; jne; movzx r,byte ptr [reg+..]; movzx r,byte ptr [reg+..]; mov r,r; xor r8,imm; mov r,r; xor r8,imm` (the blend factor tests); it reads the displacement of operand 0 of the cmp at offset 0. Alternative 0 is the shape of the standalone draw-state builder (26.63: legacy-prefix cmp, 32-bit movzx with the second one REX.R); alternative 1 is the shape after the compiler inlined the builder into its caller (Preview 1.26.70.26: REX-prefixed cmp and movzx, r8d and r9d as the factor registers). Each alternative matches once on its own build and zero times on the other. Every register, displacement, immediate and branch byte is a wildcard. The two recipes are in two functions (the parser and the draw-state builder) and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry. What changed in 6afa4e47-dadc375797a3d31a: the draw-state builder was inlined into a caller, so the prologue-relative bytes of the old recipe 1 are gone; the blend tests and the read of the flag are unchanged (would have survived with a better recipe). Evidence: (1) the blend factor tests drafted from the old builder body match once in the inlined copy on the new build, and that copy reads the same flag before the same factor loads; (2) recipe 0, from the old material JSON parser (a different function), reads the store of the enableBlend byte on the new build and gives the same member offset as recipe 1."

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
    step "pattern_any" {
      patterns = [
        "80 ?? ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 0F B6 ?? ?? ?? ?? ?? 44 0F B6 ?? ?? ?? ?? ?? 89 ?? 80 ?? ?? 45 89 ?? 41 80 ?? ??",
        "41 80 ?? ?? ?? ?? ?? ?? 0F 85 ?? ?? ?? ?? 45 0F B6 ?? ?? ?? ?? ?? 45 0F B6 ?? ?? ?? ?? ?? 45 89 ?? 41 80 ?? ?? 45 89 ?? 41 80 ?? ??",
      ]
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
  notes    = "Offset of the `bool` depthWriteMask in mce::RenderMaterial: 193h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) sets it from bit 5 of the state flags word of the material definition (def+0A8h), right after depthTestEnabled from bit 0: `not cl; and cl,1; mov [rdx+188h],cl; test al,20h; sete [rdx+193h]; shr eax,8; and al,1; mov [rdx+189h],al`. The draw-state builder 0x66b1f30 reads it back: `movzx ecx,byte ptr [rdx+188h]; mov [rax+0Ch],cl; cmp byte ptr [rdx+193h],0; setne r10b; and r10b,cl` (depth write is on only when the depth test is on too). Recipe 0 is the reader site: `movzx ecx,byte ptr [rdx+..]; mov [rax+..],cl; cmp byte ptr [rdx+..],imm; setne r10b; and r10b,cl`; it reads the displacement of operand 0 of the cmp at offset 10. Recipe 1 is the parser site: `mov [rdx+..],cl; test al,imm; sete [rdx+..]; shr eax,imm; and al,imm; mov [rdx+..],al`; it reads the displacement of operand 0 of the sete at offset 8. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry. What changed in 6afa4e47-dadc375797a3d31a: the draw-state builder was inlined into a caller, so the prologue-relative bytes of the old recipe 1 are gone; the blend tests and the read of the flag are unchanged (would have survived with a better recipe). Evidence: (1) the blend factor tests drafted from the old builder body match once in the inlined copy on the new build, and that copy reads the same flag before the same factor loads; (2) recipe 0, from the old material JSON parser (a different function), reads the store of the enableBlend byte on the new build and gives the same member offset as recipe 1."

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
  notes    = "Offset of the one-byte mce::BlendTarget blendSource in mce::RenderMaterial: 180h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) reads it from the material definition key \"blendSrc\": `lea r8,[r12+180h]; lea rax,[rip+..] (\"blendSrc\"); mov [rbp+50h],rax; mov qword ptr [rbp+58h],8; lea rdx,[rbp+50h]; mov rsi,[rbp+80h]; ...; call 0x4df6090`, the enum reader that writes through r8. The next call does the same for \"blendDst\" into +181h. The draw-state builder 0x66b1f30 reads it back only when enableBlend (+185h) is 1: `movzx ecx,byte ptr [rdx+180h]; movzx r9d,byte ptr [rdx+181h]; mov ebx,ecx; xor bl,3` (the first test of the blend table is source One, destination Zero). Recipe 0 is the reader site: `movzx ecx,byte ptr [rdx+..]; movzx r9d,byte ptr [rdx+..]; mov ebx,ecx; xor bl,imm`; it reads the displacement of operand 1 of the first movzx at offset 0. Recipe 1 is the parser site: `lea r8,[r12+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov qword ptr [rbp+..],imm; lea rdx,[rbp+..]; mov rsi,[rbp+..]`; it reads the displacement of operand 1 of the first lea at offset 0. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry. What changed in 6afa4e47-dadc375797a3d31a: the draw-state builder was inlined into a caller, so the prologue-relative bytes of the old recipe 1 are gone; the blend tests and the read of the flag are unchanged (would have survived with a better recipe). Evidence: (1) the blend factor tests drafted from the old builder body match once in the inlined copy on the new build, and that copy reads the same flag before the same factor loads; (2) recipe 0, from the old material JSON parser (a different function), reads the store of the enableBlend byte on the new build and gives the same member offset as recipe 1. What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): the blend conversion function that reads the members now has its first test as `cmp byte ptr [reg+185h],1` and reads the two bytes into r8d and r9d (REX forms; the xor table is the same), and in the material parser the frame slots of the key string are disp32 and the out pointer is saved to the frame; each recipe is a pattern_any with the old shape as alternative 0 and the new shape as alternative 1, the same offset. Evidence on 26.70.26: (1) the blend conversion function drafted from the old one: the first two reads and `xor 3`, `xor 2` of the old function are in one new function, which reads +180h and +181h under the enableBlend test; (2) the material parser, found by the string `blendDst` that only it loads on both builds, writes the blendSrc value at +180h and the blendDst value at +181h, in the order of the old parser. Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "0F B6 8A ?? ?? ?? ?? 44 0F B6 8A ?? ?? ?? ?? 89 CB 80 F3 ??",
        "45 0F B6 ?? ?? ?? ?? ?? 45 0F B6 ?? ?? ?? ?? ?? 45 89 ?? 41 80 ?? ??",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 55 ?? 48 8B B5",
        "4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? 48 89 F1 4C 89 45 ??",
      ]
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
  notes    = "Offset of the one-byte mce::BlendTarget blendDestination in mce::RenderMaterial: 181h on 6ad79ee4-17d51891a1119914 (26.60). The material JSON parser (0x4df2780) reads it from the material definition key \"blendDst\", right after blendSrc: `mov bl,al; lea r8,[r12+181h]; lea rax,[rip+..] (\"blendDst\"); mov [rbp],rax; mov qword ptr [rbp+8],8; mov rdx,rbp; mov rcx,rsi; mov rdi,r8; call 0x4df6090`, the enum reader that writes through r8. The draw-state builder 0x66b1f30 reads it back only when enableBlend (+185h) is 1: `movzx ecx,byte ptr [rdx+180h]; movzx r9d,byte ptr [rdx+181h]; mov ebx,ecx; xor bl,3; mov r10d,r9d; xor r10b,2` (the first test of the blend table is source One, destination Zero). Recipe 0 is the reader site: the same pattern as recipe 0 of mBlendSource; it reads the displacement of operand 1 of the second movzx at offset 7. Recipe 1 is the parser site: `mov bl,al; lea r8,[r12+..]; lea rax,[rip+..]; mov [rbp+..],rax; mov qword ptr [rbp+..],imm; mov rdx,rbp; mov rcx,rsi; mov rdi,r8`; it reads the displacement of operand 1 of the lea at offset 2. The two recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Range window 100h..200h. 26.51 is not derived for this entry. What changed in 6afa4e47-dadc375797a3d31a: the draw-state builder was inlined into a caller, so the prologue-relative bytes of the old recipe 1 are gone; the blend tests and the read of the flag are unchanged (would have survived with a better recipe). Evidence: (1) the blend factor tests drafted from the old builder body match once in the inlined copy on the new build, and that copy reads the same flag before the same factor loads; (2) recipe 0, from the old material JSON parser (a different function), reads the store of the enableBlend byte on the new build and gives the same member offset as recipe 1. What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): the blend conversion function that reads the members now has its first test as `cmp byte ptr [reg+185h],1` and reads the two bytes into r8d and r9d (REX forms; the xor table is the same), and in the material parser `mov rdx,rbp` became `lea rdx,[rbp+..]` with a disp32 frame; each recipe is a pattern_any with the old shape as alternative 0 (alternative 0 of recipe 0 has one leading wildcard so that one offset serves both) and the new shape as alternative 1. Evidence on 26.70.26: (1) the blend conversion function drafted from the old one: the two reads and `xor 3`, `xor 2` of the old function are in one new function, which reads +180h then +181h; (2) the material parser, found by the string `blendDst` that only it loads on both builds, writes the blendDst value to +181h right after blendSrc at +180h, as the old parser does. Class: would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? 0F B6 8A ?? ?? ?? ?? 44 0F B6 8A ?? ?? ?? ?? 89 CB 80 F3 ??",
        "45 0F B6 ?? ?? ?? ?? ?? 45 0F B6 ?? ?? ?? ?? ?? 45 89 ?? 41 80 ?? ??",
      ]
      offset = 8
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "88 C3 4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 EA 48 89 F1 4C 89 C7",
        "88 C3 4D 8D 84 24 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 55 ?? 48 89 F1 4C 89 C7",
      ]
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
