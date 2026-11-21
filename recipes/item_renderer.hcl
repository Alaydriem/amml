entry "ItemRenderer.renderGuiItemNew" {
  kind     = "function"
  notes    = "void ItemRenderer::renderGuiItemNew(BaseActorRenderContext& ctx, ItemStack const& stack, int frame, float x, float y, bool foil, float transparency, float light, float scale, int zOrder): the game's own call to draw one item picture in the GUI. rcx the ItemRenderer, rdx the BaseActorRenderContext, r8 the stack, r9d the frame, then on the stack from [rsp+28h] (the callee's view; the caller writes [rsp+20h..50h]): x, y (f32), foil (byte), transparency, light, scale (f32), zOrder (int); void. The item renderer is the pointer at BaseActorRenderContext+58h, which the caller loads (`mov rcx,[rbp+..]`). The inner function with the frame first and 13 parameters (6E73DC0h) is a different function that this one calls; do not use it. Its only callers are two calls in one helper (the flying-item draw: it gets the local player, asks the item for its animation frame, builds a BaseActorRenderContext, draws, and when the glint flag is set draws again with foil = 1). Recipe 0 is the prologue and the stack setup of the function (`push rbp; push r15; push r14; push r13; push r12; push rsi; push rdi; push rbx; sub rsp,..; lea rbp,[rsp+..]; movaps [rbp+..],xmm10; movaps [rbp+..],xmm9; movaps [rbp+..],xmm8; movaps [rbp+..],xmm7; movaps [rbp+..],xmm6; mov qword ptr [rbp+..],-2; mov r14d,r9d; mov`). Recipe 1 is the first call in the helper: the stores of transparency, light and the foil byte, then `mov rdx,r15; mov r8,rdi; mov r9d,ebx; call`, read as the call target. The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is function_start. No build gate: builds before 26.63 are not supported. Value 6E73200h on 6ad79ee4-17d51891a1119914 (26.60). What changed in 6afa4e47-dadc375797a3d31a (preview 26.70.26): nothing in the function; only the old build gate refused it. Evidence on 26.70.26: (1) recipe 0, a pattern drafted from the old prologue and stack setup, matches once, in a function with the same two callers in one helper; (2) recipe 1, the first call of that helper in the old function (the stores of transparency, light and the foil byte before the call), reaches the same function, and the old function has the same two callers in one helper. Class: gate only."

  recipe {
    step "pattern" { bytes = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC ?? ?? ?? ?? 48 8D AC 24 ?? ?? ?? ?? 44 0F 29 95 ?? ?? ?? ?? 44 0F 29 8D ?? ?? ?? ?? 44 0F 29 85 ?? ?? ?? ?? 0F 29 BD ?? ?? ?? ?? 0F 29 B5 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 45 89 CE 4C" }
  }

  recipe {
    step "pattern" {
      bytes  = "C7 44 24 ?? ?? ?? ?? ?? C6 44 24 ?? ?? 4C 89 FA 49 89 F8 41 89 D9 E8 ?? ?? ?? ??"
      offset = 22
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

}
