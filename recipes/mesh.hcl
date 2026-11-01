entry "Mesh.moveCtor" {
  kind      = "function"
  msvc_name = "??0Mesh@mce@@QEAA@$$QEAV01@@Z"
  notes     = "mce::Mesh::Mesh(Mesh&&): rcx = this, rdx = the source. BaseActorRenderer._extractRenderTextObject moves the Mesh that Tessellator.end returned into a make_shared<Mesh> block (288h bytes; the object at +10h) with it, then destroys the local with Mesh.dtor. On 26.51 the body stores the IndexBufferContainer vtable, copies the byte at +40h, zero-fills the MeshData at +58h (memset of +60h, 148h bytes) and calls MeshData::clear on it, then move-assigns. On 26.60 there is no vtable store, the byte is at +38h and the MeshData at +50h (memset of +58h, 148h bytes). Recipe 0 is the part of the body that both builds share: `movzx eax,byte ptr [rdx+..]; mov [rcx+..],al; movups [rcx+..],xmm6; lea r64,[rcx+..]; add rcx,..; mov r8d,..` (the memset size), then function_start_of. The REX and ModRM bytes of the lea and every displacement and immediate are wildcards. Recipe 1 is the call site in BaseActorRenderer._extractRenderTextObject (`lea rdx,[rbp+..]; mov rcx,r14; mov [rbp+..],r64; call`), with the REX and ModRM bytes of the store wildcarded. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): mce::Mesh lost the IndexBufferContainer vtable pointer and 16 more bytes (632 to 608 bytes; see Mesh.size), so the 26.51 body head (which held the vtable store) did not match; the call site keeps the shared_ptr block in r15 instead of rdi. Real code change (layout); the recipes would have survived with the shared part of the body."

  recipe {
    step "pattern" { bytes = "0F B6 42 ?? 88 41 ?? 0F 11 71 ?? ?? 8D ?? ?? 48 83 C1 ?? 41 B8 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "BaseActorRenderer._extractRenderTextObject" }
    step "pattern" {
      bytes  = "48 8D 55 ?? 4C 89 F1 ?? 89 ?? ?? ?? ?? ?? E8"
      offset = 14
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Mesh.dtor" {
  kind      = "function"
  msvc_name = "??1Mesh@mce@@QEAA@XZ"
  notes     = "mce::Mesh::~Mesh(): rcx = this. The body first calls a reset of the GPU state, frees the vector at +260h and destroys the MeshData at +58h with MeshData.dtor. Recipe 0 is the body (`mov rsi,rcx; call; mov rcx,[rsi+..]; test rcx,rcx; je; mov rdx,[rsi+..]; sub rdx,rcx; cmp rdx,..; jb; mov rax,[rcx-8]`), then function_start_of. Recipe 1 is the call site in BaseActorRenderer._extractRenderTextObject after the move: `mov qword ptr [rbp+..],rax; mov byte ptr [rbp+..],0; lea rcx,[rbp+..]; call`. Check function_start."

  recipe {
    step "pattern" { bytes = "48 89 CE E8 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 96 ?? ?? ?? ?? 48 29 CA 48 81 FA ?? ?? ?? ?? 72 ?? 48 8B 41 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "ref" { entry = "BaseActorRenderer._extractRenderTextObject" }
    step "pattern" {
      bytes  = "48 89 85 ?? ?? ?? ?? C6 85 ?? ?? ?? ?? ?? 48 8D 4D ?? E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Mesh.isValid" {
  kind      = "function"
  msvc_name = "?isValid@Mesh@mce@@QEBA_NXZ"
  notes     = "bool mce::Mesh::isValid() const: rcx = this, al = the result. It returns true when the positions vector (the first vector of the MeshData: Mesh+60h on 26.51, Mesh+58h on 26.60) is not empty; else it checks the buffer handles (`movzx eax,word ptr [rcx+..]; or ax,[rcx+..]; jne`). Callers test al right after Tessellator.end. Recipe 0 is the early return and the first handle test, which both builds share: `je; add rsp,..; pop rdi; pop rsi; ret; movzx eax,word ptr [rcx+..]; or ax,[rcx+..]; jne`, then function_start_of. Recipe 1 is a call site right after Tessellator.end (`mov r8d,2; lea r9,[rbp+..]; call end; ..; call isValid; test al,al; je`), one pattern_any with one alternative per shape, the isValid call at offset 27 in both: alternative 0 (26.51) `mov rcx,r14; mov rdx,r13; mov r8d,..; lea r9,[rbp+..]; call; mov rcx,r13; call; test al,al; je`; alternative 1 (26.60) `lea rdx,[rbp+..]; mov r8d,..; lea r9,[rbp+..]; call; nop; lea rcx,[rbp+..]; call; nop; test al,al; je`. Both run past the end call. Check function_start. What changed in 6ad79ee4-17d51891a1119914 (26.60): the vertex count test is a 32-bit imul and the Mesh members moved (-8h before the MeshData, -18h at the handles), so the body head did not match; the call site keeps the Mesh on the stack and has a nop after each call. The new function was accepted on two facts from the 26.51 function: the same body (count test, `mov al,1`, return, then the word handle test at offsets 18h lower, the Mesh.dtor delta), and the same caller shape (the 26.51 caller of recipe 1 and its 26.60 counterpart both call Tessellator.end with mode 2, then this function, then test al and store xmm6 into a local). Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "74 ?? 48 83 C4 ?? 5F 5E C3 0F B7 81 ?? ?? ?? ?? 66 0B 81 ?? ?? ?? ?? 75 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 89 F1 4C 89 EA 41 B8 ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? 4C 89 E9 E8 ?? ?? ?? ?? 84 C0 0F 84",
        "48 8D 55 ?? 41 B8 ?? ?? ?? ?? 4C 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8D 4D ?? E8 ?? ?? ?? ?? 90 84 C0 0F 84",
      ]
      offset = 27
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Mesh.renderMesh" {
  kind      = "function"
  msvc_name = "?renderMesh@Mesh@mce@@QEBAXAEAVMeshContext@2@AEBVMaterialPtr@2@AEBV?$variant@Umonostate@std@@VTexturePtr@mce@@UClientTexture@4@UServerTexture@4@@std@@IIAEBUOffscreenCaptureDescription@@PEBVIndexBufferContainer@2@@Z"
  notes     = "void mce::Mesh::renderMesh(MeshContext&, MaterialPtr const&, std::variant<monostate, TexturePtr, ClientTexture, ServerTexture> const&, uint, uint, OffscreenCaptureDescription const&, IndexBufferContainer const*) const, the first overload: rcx = this, rdx = MeshContext (ScreenContext+10h), r8 = material, r9 = the texture variant (index byte at +20h), [rsp+20h] start, [rsp+28h] count, [rsp+30h] offscreen, [rsp+38h] index buffer. A caller passes `mov rdx,[rsi+28h]; add rdx,10h` and a TexturePtr variant with index 1. The body copies the variant (`movsx rcx,byte ptr [r9+20h]; inc rcx`) and calls the implementation. Recipe 0 is the body head (`mov rsi,r8; mov rdi,rdx; mov rbx,rcx; mov byte ptr [rbp+..],0; mov qword ptr [rbp+..],0; mov byte ptr [rbp+..],..; lea rax,[rbp+..]`), then function_start_of. Recipe 1 is a call site in a draw helper after Tessellator.end: `mov dword ptr [rsp+20h],0; mov rcx,r15; mov rdx,rdi; mov r8,r14; mov r9,rbx; call`. Check function_start."

  recipe {
    step "pattern" { bytes = "4C 89 C6 48 89 D7 48 89 CB C6 85 ?? ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? C6 45 ?? ?? 48 8D 45 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "C7 44 24 ?? ?? ?? ?? ?? 4C 89 F9 48 89 FA 4D 89 F0 49 89 D9 E8"
      offset = 20
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "Mesh.size" {
  kind     = "field"
  notes    = "Size of mce::Mesh: 278h (632) on 26.51 and 260h (608) on 26.60. It selects the mirror (MeshLayoutKind in stratum): MeshLayout2651 (632) on 26.51, MeshLayout2660 (608) on 26.60. BaseActorRenderer._extractRenderTextObject moves a Mesh into a make_shared<Mesh> block that it allocates with 288h bytes, 632 for the Mesh and 16 for the control block (`mov edx,288h; call [guard]`, then the control block counts at +8 and the Mesh at +10h). The recipe reads that immediate inside BaseActorRenderer._extractRenderTextObject (the call reads the allocator slot through rax) and subtracts 16. The pattern ends at the allocator call (`mov rax,[rax+..]; mov edx,imm32; call [rip+..]`). Range window 260h..290h. What changed in 6ad79ee4-17d51891a1119914 (26.60): the block is 270h bytes (Mesh 260h); Mesh.dtor on both builds (inspect fields) shows the members after the base moved by -8h, and from +1C8h on by -18h, as the move constructor no longer stores a vtable. The compiler adds a nop after the allocator call, so the old pattern (which ran past the call) did not match. Real code change (layout): a second mirror."

  recipe {
    step "ref" { entry = "BaseActorRenderer._extractRenderTextObject" }
    step "pattern" {
      bytes  = "48 8B 40 ?? BA ?? ?? ?? ?? FF 15"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
    step "add" { value = -16 }
  }

  check "range" {
    min = 608
    max = 656
  }
}
