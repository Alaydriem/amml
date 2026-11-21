entry "GlobalConstantBuffers.entityConstantBuffer" {
  kind     = "field"
  notes    = "Offset of mce::ActorConstants GlobalConstantBuffers::entityConstantBuffer: C0h on 26.51 (header: renderChunk 0, world 20h, perFrame 50h, animation 98h, entity C0h, shader 128h; the Singleton base is empty), the anchor for the constants that ActorShaderManager.setEntityConstants takes. The GlobalConstantBuffers init calls the ActorConstants init on this+C0h between animation (+98h) and shader (+128h). The recipe is the shader setup function that calls setEntityConstants: it loads the constant buffers from the screen context and adds the offset to r15 (`add r15,C0h`), then tests the flag byte at +0Ch of the optional color it reads next; it reads the immediate. It is one pattern_any from the add, one alternative per shape: alternative 0 (26.51) `add r15,imm32; cmp byte ptr [rcx+..],..; je; movss xmm13,[rcx]`; alternative 1 (26.60) `add r15,imm32; mov r8,rcx; mov ecx,[rcx+..]; test cl,cl; je`. Range window B0h..D0h. Value C0h on 26.51 and 26.60. What changed in 6ad79ee4-17d51891a1119914 (26.60): the color math around the add is vectorized, so the divss before the add and the flag test after it changed; the add and its offset did not. Would have survived with a better recipe. What changed in 6afa4e47-dadc375797a3d31a: the offset is no longer added with `add r15,imm32` after the constant buffers pointer is loaded; the shader setup function now loads the offset first and adds the base (`mov r14d,imm32; add r14,[rcx+..]`, at the top of the same function, right before the actor position loads), so alternative 2 reads the immediate of the `mov`; the recipe cursor is the start of that `mov`, and operand 1 is the immediate. Class: would have survived with a better recipe. Evidence: (1) the old function is one of the two callers of ActorShaderManager.setEntityConstants (a resolved entry on both builds), and the function that holds alternative 2 is one of the two callers of it on the new build; (2) the old function reads the actor position through the state vector component (`mov rax,[r8+disp]` with disp equal to Actor.stateVectorComponent, 240h on 6aed1906-b127418e74d2f2f9) and the function with alternative 2 reads it through the same member (248h = Actor.stateVectorComponent on 6afa4e47-dadc375797a3d31a); the pattern matches once there and not on the old build."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C7 ?? ?? ?? ?? 80 79 ?? ?? 74 ?? F3 44 0F 10 29",
        "49 81 C7 ?? ?? ?? ?? 49 89 C8 8B 49 ?? 84 C9 74",
        "41 BE ?? ?? ?? ?? 4C 03 71 ?? 49 8B 80 ?? ?? ?? ?? F3 44 0F 10 00 F3 0F 10 70 ?? F3 0F 10 78 ?? 0F 10 05 ?? ?? ?? ?? 0F 29 45 ?? 4C 89 C1 E8",
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
