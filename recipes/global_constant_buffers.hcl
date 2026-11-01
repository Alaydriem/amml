entry "GlobalConstantBuffers.entityConstantBuffer" {
  kind     = "field"
  notes    = "Offset of mce::ActorConstants GlobalConstantBuffers::entityConstantBuffer: C0h on 26.51 (header: renderChunk 0, world 20h, perFrame 50h, animation 98h, entity C0h, shader 128h; the Singleton base is empty), the anchor for the constants that ActorShaderManager.setEntityConstants takes. The GlobalConstantBuffers init calls the ActorConstants init on this+C0h between animation (+98h) and shader (+128h). The recipe is the shader setup function that calls setEntityConstants: it loads the constant buffers from the screen context and adds the offset to r15 (`add r15,C0h`), then tests the flag byte at +0Ch of the optional color it reads next; it reads the immediate. It is one pattern_any from the add, one alternative per shape: alternative 0 (26.51) `add r15,imm32; cmp byte ptr [rcx+..],..; je; movss xmm13,[rcx]`; alternative 1 (26.60) `add r15,imm32; mov r8,rcx; mov ecx,[rcx+..]; test cl,cl; je`. Range window B0h..D0h. Value C0h on 26.51 and 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): the color math around the add is vectorized, so the divss before the add and the flag test after it changed; the add and its offset did not. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C7 ?? ?? ?? ?? 80 79 ?? ?? 74 ?? F3 44 0F 10 29",
        "49 81 C7 ?? ?? ?? ?? 49 89 C8 8B 49 ?? 84 C9 74",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 176
    max = 208
  }
}
