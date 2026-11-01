entry "ActorRenderDispatcher.render" {
  kind      = "function"
  hooked    = true
  msvc_name = "?render@ActorRenderDispatcher@@QEAAXAEAVBaseActorRenderContext@@AEAVActor@@AEBVVec3@@2AEBVVec2@@_N@Z"
  notes     = "void ActorRenderDispatcher::render(BaseActorRenderContext&, Actor&, Vec3 const& cameraTargetPos, Vec3 const& pos, Vec2 const& rot, bool ignoreLighting) (the seven-argument form with `this`): 0x1d51540 in 26.51, 0x594be00 in 26.60. rcx = the dispatcher, rdx = the context, r8 = the actor, r9 = cameraTargetPos, and on the stack pos (entry+28h), rot (+30h) and the bool (+38h). It returns void. It fills an ActorRenderData (mActor, position, rotation, mIgnoreLighting) and draws the actor; a hook that returns without the original skips the draw. 15 direct callers in each build; the three-argument render(ctx, actor, bool) calls it. Recipe 0 is `string_xref` of \"No renderer found - have you set the entity's description:identifier correctly?\": the only function that loads it in both builds. Recipe 1 is the stack argument loads: `mov r15,[rbp+..]; mov r12,[rbp+..]; movzx r14d,byte ptr [rbp+..]; movzx eax,byte ptr [rsi+..]; mov [rbp+..],al; mov [rsi+..],r14b; mov rcx,[rsi+..]`, then `function_start_of`. The two recipes share no bytes. Every immediate, displacement and branch byte of the pattern is a wildcard. Both are unique in both exes and agree. Check: function_start. What changed in 26.60: the address only; the code shape is the same."

  recipe {
    step "string_xref" { text = "No renderer found - have you set the entity's description:identifier correctly?" }
  }

  recipe {
    step "pattern" {
      bytes = "4C 8B BD ?? ?? ?? ?? 4C 8B A5 ?? ?? ?? ?? 44 0F B6 B5 ?? ?? ?? ?? 0F B6 86 ?? ?? ?? ?? 88 45 ?? 44 88 B6 ?? ?? ?? ?? 48 8B 4E"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}
