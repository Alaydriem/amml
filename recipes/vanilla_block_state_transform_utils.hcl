entry "VanillaBlockStateTransformUtils.transformBlock" {
  kind      = "function"
  msvc_name = "?transformBlock@VanillaBlockStateTransformUtils@@SAPEBVBlock@@AEBV2@W4Rotation@@W4Mirror@@@Z"
  notes     = "static Block const* VanillaBlockStateTransformUtils::transformBlock(Block const&, Rotation, Mirror): rcx = block, dl = rotation, r8b = mirror (the three-argument overload; the two others take one direction). It looks the block's states up in the block type state map (BlockType+1D8h) and rotates or mirrors each known state. Structure placement passes the two bytes of its settings: `movzx edx,byte ptr [rax]; movzx r8d,byte ptr [rax+1]; mov rcx,[rdi+10h]; call`. Recipe 0 is the body head (`mov ebx,edx; mov edi,r8d; mov rsi,rcx; mov r8,[rcx+68h]; mov r9,[rip+..]; mov rax,[r8+..]`), then function_start_of. Recipe 1 is that structure call site. Check function_start. The REX byte of `mov rcx,[r64+10h]` in recipe 1 is a wildcard. What changed in 6ad79ee4-17d51891a1119914 (26.60): the structure placement function keeps its settings owner in r15 (rdi on 26.51), so recipe 1 did not match; the call site is otherwise the same and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "89 D3 44 89 C7 48 89 CE 4C 8B 41 ?? 4C 8B 0D ?? ?? ?? ?? 49 8B 80 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "0F B6 10 44 0F B6 40 ?? ?? 8B 4F ?? E8"
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "VanillaBlockStateTransformUtils.hasTransformableWallStates" {
  kind      = "function"
  msvc_name = "?hasTransformableWallStates@VanillaBlockStateTransformUtils@@SA_NAEBVBlock@@@Z"
  notes     = "static bool VanillaBlockStateTransformUtils::hasTransformableWallStates(Block const&): rcx = block, al = result. A leaf without .pdata: it looks the wall_connection_type_north state id (the state global that the vanilla state initializer fills right after the wall_connection_type_north text) up in the block type state map. The two structure placement helpers call it on the transformed block and, when it is true, tail-jump to VanillaBlockStateTransformUtils.transformWallConnectionTypes; else they test VanillaBlockStateTransformUtils.hasTransformableConnectionStates. Recipe 0 is the call in the first helper (`mov r13,[rsi+..]; mov rcx,rdi; call`); recipe 1 is the call in the second helper (`cmp rdx,r8; jne; xor r9d,r9d; jmp; mov rcx,rsi; call`). Check leaf_start. Recipe 0 now starts at the jb before the load and wildcards the ModRM byte of the load (`jb; mov r64,[rsi+..]; mov rcx,rdi; call`, offset 16). What changed in 6ad79ee4-17d51891a1119914 (26.60): the first helper loads into r15 (r13 on 26.51), so recipe 0 did not match; recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "0F 82 ?? ?? ?? ?? 4C 8B ?? ?? ?? ?? ?? 48 89 F9 E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "4C 39 C2 75 ?? 45 31 C9 E9 ?? ?? ?? ?? 48 89 F1 E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}

entry "VanillaBlockStateTransformUtils.transformWallConnectionTypes" {
  kind      = "function"
  msvc_name = "?transformWallConnectionTypes@VanillaBlockStateTransformUtils@@SAPEBVBlock@@AEBV2@W4Rotation@@W4Mirror@@@Z"
  notes     = "static Block const* VanillaBlockStateTransformUtils::transformWallConnectionTypes(Block const&, Rotation, Mirror): rcx = block, dl = rotation, r8b = mirror. Besides the vanilla state initializer it is the only function with all four wall_connection_type state globals; transformBlock also calls it. Recipe 0 is its first bytes (`push r15..rbx; sub rsp,..; mov ebx,edx; mov ebp,r8d; mov rsi,rcx; mov rax,[rip+..]`; transformConnectionStates has the same start with `mov edi,r8d`), so function_start is a real check. Recipe 1 is the tail jump of the first structure placement helper after the wall check, followed by the connection check: `pop r15; jmp; mov rcx,rdi; call`, the jump target. Check function_start."

  recipe {
    step "pattern" { bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC ?? 89 D3 44 89 C5 48 89 CE 48 8B 05 ?? ?? ?? ??" }
  }

  recipe {
    step "pattern" {
      bytes  = "41 5F E9 ?? ?? ?? ?? 48 89 F9 E8"
      offset = 2
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "VanillaBlockStateTransformUtils.hasTransformableConnectionStates" {
  kind      = "function"
  msvc_name = "?hasTransformableConnectionStates@VanillaBlockStateTransformUtils@@SA_NAEBVBlock@@@Z"
  notes     = "static bool VanillaBlockStateTransformUtils::hasTransformableConnectionStates(Block const&): rcx = block, al = result. A leaf without .pdata with the same shape as hasTransformableWallStates (a state map lookup of another state global). The structure placement helpers call it after the wall test and, when it is true, tail-jump to VanillaBlockStateTransformUtils.transformConnectionStates. Recipe 0 is the call in the first helper right after the wall tail jump (`pop r15; jmp; mov rcx,rdi; call`); recipe 1 is the call in the second helper right after its wall tail jump (`mov rcx,rsi; mov edx,edi; mov r8d,ebx; add rsp,..; pop rbx..r15; jmp; mov rcx,rsi; call`). Check leaf_start. Recipe 1 is now a pattern_any, one alternative per shape of the second helper, the call at offset 27 in both: alternative 0 (26.51) `mov r8d,ebx; add rsp; pop x8; jmp; mov rcx,rsi; call`; alternative 1 (26.60) `mov rcx,rsi; add rsp; pop x8; jmp; mov rcx,rsi; call` and the code after it (`test al,al; je; movzx r8d,byte ptr [rdi+9]`). What changed in 6ad79ee4-17d51891a1119914 (26.60): the second helper checks the stack cookie before the tail jump and reads the rotation and mirror from its settings again, so the 26.51 shape did not match; recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" {
      bytes  = "41 5F E9 ?? ?? ?? ?? 48 89 F9 E8"
      offset = 10
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "41 89 D8 48 83 C4 ?? 5B 5D 5F 5E 41 5C 41 5D 41 5E 41 5F E9 ?? ?? ?? ?? 48 89 F1 E8",
        "48 89 F1 48 83 C4 ?? 5B 5D 5F 5E 41 5C 41 5D 41 5E 41 5F E9 ?? ?? ?? ?? 48 89 F1 E8 ?? ?? ?? ?? 84 C0 74 ?? 44 0F B6 47",
      ]
      offset = 27
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "leaf_start" {}
}

entry "VanillaBlockStateTransformUtils.transformConnectionStates" {
  kind      = "function"
  msvc_name = "?transformConnectionStates@VanillaBlockStateTransformUtils@@SAPEBVBlock@@AEBV2@W4Rotation@@W4Mirror@@@Z"
  notes     = "static Block const* VanillaBlockStateTransformUtils::transformConnectionStates(Block const&, Rotation, Mirror): rcx = block, dl = rotation, r8b = mirror. transformBlock calls it, and the structure placement helpers tail-jump to it after hasTransformableConnectionStates. Recipe 0 is its first bytes (`push r15..rbx; sub rsp,..; mov ebx,edx; mov edi,r8d; mov rsi,rcx; mov rax,[rip+..]`; transformWallConnectionTypes uses ebp), so function_start is a real check. Recipe 1 is the tail jump of the first helper: `pop r15; jmp; or r14b,r15b; or r14b,bpl`, the jump target. Check function_start. Recipe 1 is now a pattern_any on the code after the tail jump, one alternative per shape: alternative 0 (26.51) `or r14b,r15b; or r14b,bpl`; alternative 1 (26.60) `test bpl,bpl; je`. What changed in 6ad79ee4-17d51891a1119914 (26.60): the code after the tail jump in the first helper changed; the jump (after the hasTransformableConnectionStates call and the stack cookie check) is the same, and recipe 0 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "41 57 41 56 41 55 41 54 56 57 55 53 48 83 EC ?? 89 D3 44 89 C7 48 89 CE 48 8B 05 ?? ?? ?? ??" }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "41 5F E9 ?? ?? ?? ?? 45 08 FE 41 08 EE",
        "41 5F E9 ?? ?? ?? ?? 40 84 ED 0F 84",
      ]
      offset = 2
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
