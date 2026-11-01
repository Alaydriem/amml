entry "MatrixStack.push" {
  kind      = "function"
  msvc_name = "?push@MatrixStack@@QEAA?AUMatrixStackRef@1@_N@Z"
  notes     = "MatrixStack::MatrixStackRef MatrixStack::push(bool isSortOrigin): rcx = the stack, rdx = the 16-byte MatrixStackRef buffer (hidden return pointer), r8b = isSortOrigin. It sets the dirty flag (+38h), stores the sort origin (+28h, flag +30h) when r8b is set, pushes a copy of the top matrix (deque push_back of back()) and writes {stack, &top} to [rdx]. pushIdentity (the function after it) has the same head but pushes an identity matrix. Recipe 0 is the head (`mov rsi,rdx; mov rdi,rcx; mov byte ptr [rcx+..],1; mov rax,[rcx+..]; test r8b,r8b; je; mov [rdi+..],rax; mov byte ptr [rdi+..],1; mov rcx,[rdi+..]; add rax,rcx; dec rax`), then function_start_of; the last three instructions tell it from pushIdentity. Recipe 1 is a call on the camera world matrix stack: `mov rax,[rdi+..]; mov rcx,[rax+..]; add rcx,40h; lea rdx,[rbp+..]; xor r8d,r8d; call`. Check function_start. The call site of push is now a pattern_any from `mov rcx,[rax+18h]; add rcx,40h; lea rdx,[rbp+..]; xor r8d,r8d; call` on to the code after the call (`mov rbx,[rbp+..]; mov byte ptr [rbx+38h],1`, the dirty flag set through the ref), one alternative per shape: alternative 1 has the nop that 26.60 adds after the call. It matches once on each build, in a draw function that pushes the ScreenContext matrix stack (+40h of the object at [ScreenContext+28h]+18h) and pops it with MatrixStackRef.dtor on a local at the end. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site (another draw function, with `mov rax,[rdi+28h]` first) has the ScreenContext in r12 (a SIB byte) and a nop after the call, and push has 4 direct callers (9 on 26.51), so the site moved to the function with the same shape on both builds. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 D6 48 89 CF C6 41 ?? ?? 48 8B 41 ?? 45 84 C0 74 ?? 48 89 47 ?? C6 47 ?? ?? 48 8B 4F ?? 48 01 C8 48 FF C8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 48 ?? 48 83 C1 ?? 48 8D 55 ?? 45 31 C0 E8 ?? ?? ?? ?? 48 8B 5D ?? C6 43",
        "48 8B 48 ?? 48 83 C1 ?? 48 8D 55 ?? 45 31 C0 E8 ?? ?? ?? ?? 90 48 8B 5D ?? C6 43",
      ]
      offset = 15
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
