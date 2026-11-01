entry "ActorShaderManager.setEntityConstants" {
  kind      = "function"
  msvc_name = "?setEntityConstants@ActorShaderManager@@SAXAEAVActorConstants@mce@@AEAVRenderContext@3@AEBVColor@3@AEBVVec2@@AEBU?$vec@$03M$0A@@glm@@22223AEBVVec4@@MMMM@Z"
  notes     = "static void ActorShaderManager::setEntityConstants(ActorConstants&, RenderContext&, Color const&, Vec2 const&, glm::vec4 const&, Color const& x4, Vec2 const&, Vec4 const&, float x4): rcx = constants, rdx = render context, r8 = the first Color, r9 = the first Vec2, the rest on the stack from [rsp+20h] at the call. The ActorConstants init stores the 11 shader constant pointers at +10h..+60h; the body writes the constants from +18h on (TILE_LIGHT_COLOR from r8, TILE_LIGHT_INTENSITY from r9, then the stack arguments) and marks each dirty (+31h). A shader setup function passes `mov r15,[screen+20h]` (MeshContext::constantBuffers) plus C0h (GlobalConstantBuffers.entityConstantBuffer). Recipe 0 is the head (`mov rax,[rsp+..]; mov rdx,[rsp+..]; movss xmm0,[rsp+..]; movss xmm1,[rsp+..]; mov r10,[rsp+..]`), then function_start_of. Recipe 1 is that call (`mov [rsp+20h],rax; lea r8,[rbp+..]; lea r9,[rbp+..]; mov rcx,r15; call`). Check function_start. Recipe 1 now finds the shader setup function by the `add r15,C0h` of GlobalConstantBuffers.entityConstantBuffer (the same pattern_any), then the call in it: `lea r9,[rbp+..]; mov rcx,r15; call` (the second Vec2 and the constants), read as a branch target. What changed in 6ad79ee4-17d51891a1119914 (26.60): the first Color is at a frame offset above 80h (`lea r8,[rbp+disp32]`), so the 26.51 call-site pattern with a disp8 did not match. The function has 2 direct callers on each build, and the shader setup function calls it with r15 = constant buffers + C0h on both. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 8B 44 24 ?? 48 8B 54 24 ?? F3 0F 10 84 24 ?? ?? ?? ?? F3 0F 10 8C 24 ?? ?? ?? ?? 4C 8B 94 24 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C7 ?? ?? ?? ?? 80 79 ?? ?? 74 ?? F3 44 0F 10 29",
        "49 81 C7 ?? ?? ?? ?? 49 89 C8 8B 49 ?? 84 C9 74",
      ]
    }
    step "pattern" {
      bytes  = "4C 8D 8D ?? ?? ?? ?? 4C 89 F9 E8"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
