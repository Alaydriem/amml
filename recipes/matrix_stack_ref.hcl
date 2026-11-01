entry "MatrixStackRef.dtor" {
  kind      = "function"
  msvc_name = "??1MatrixStackRef@MatrixStack@@QEAA@XZ"
  notes     = "MatrixStack::MatrixStackRef::~MatrixStackRef(): rcx = the ref. A leaf without .pdata (201 callers): when the stack pointer is set it sets the dirty flag, clears the sort origin when the stack falls below it, pops one matrix (size - 1, offset reset when empty) and zeroes the ref. Recipe 0 is the body at its first byte (`mov rax,[rcx]; test rax,rax; je; mov byte ptr [rax+..],1; cmp byte ptr [rax+..],0; je; mov rdx,[rax+..]; lea r8,[rdx-1]`). Recipe 1 starts at the MatrixStack.push call on the camera world stack and takes the ref destructor call at the end of the draw (`mov r8,r14; mov r9,rsi; call; lea rcx,[rbp+..]; call`, the second call). Check leaf_start. The call site of push is now a pattern_any from `mov rcx,[rax+18h]; add rcx,40h; lea rdx,[rbp+..]; xor r8d,r8d; call` on to the code after the call (`mov rbx,[rbp+..]; mov byte ptr [rbx+38h],1`, the dirty flag set through the ref), one alternative per shape: alternative 1 has the nop that 26.60 adds after the call. It matches once on each build, in a draw function that pushes the ScreenContext matrix stack (+40h of the object at [ScreenContext+28h]+18h) and pops it with MatrixStackRef.dtor on a local at the end. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site (another draw function, with `mov rax,[rdi+28h]` first) has the ScreenContext in r12 (a SIB byte) and a nop after the call, and push has 4 direct callers (9 on 26.51), so the site moved to the function with the same shape on both builds. Would have survived with a better recipe. Recipe 1 is now another push caller, found by its push site (`lea rdx,[rbp+..]; xor r8d,r8d; call push; mov r14,[rbp+..]; mov byte ptr [r14+38h],1; mov r12,[rbp+..]`, one pattern_any alternative per shape: 26.60 has a nop after the call), then in that function the ref destructor call on the same local (`call; lea rcx,[rbp+..]; call; lea rax,[rip+..]`), read as the call target. The first route (the end of the push recipe 1 function) cannot be linted on 26.60: the linear decode of that function does not land on the match."

  recipe {
    step "pattern" { bytes = "48 8B 01 48 85 C0 74 ?? C6 40 ?? ?? 80 78 ?? ?? 74 ?? 48 8B 50 ?? 4C 8D 42 ??" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 95 ?? ?? ?? ?? 45 31 C0 E8 ?? ?? ?? ?? 4C 8B B5 ?? ?? ?? ?? 41 C6 46 ?? ?? 4C 8B A5",
        "48 8D 95 ?? ?? ?? ?? 45 31 C0 E8 ?? ?? ?? ?? 90 4C 8B B5 ?? ?? ?? ?? 41 C6 46 ?? ?? 4C 8B A5",
      ]
    }
    step "pattern" {
      bytes  = "E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 05"
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}
