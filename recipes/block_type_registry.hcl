entry "BlockTypeRegistry.instance" {
  kind      = "global"
  msvc_name = "?mBlockTypeRegistry@BlockTypeRegistry@@2V?$Owner@VBlockTypeRegistry@@@Bedrock@@A"
  notes     = "The global Bedrock::Owner<BlockTypeRegistry> BlockTypeRegistry::mBlockTypeRegistry on 26.51. Owner derives from EnableNonOwnerReferences (a vtable and a control-block shared_ptr, 18h bytes), so the registry (Owner::mValue) is at instance+18h: its destructor stores the Owner vtable at the global, destroys the registry at +18h, then releases the control block at +8/+10h. Code passes the registry as `this` with `lea rcx,[rip+registry]`: BlockTessellatorCache.getBlock (the lazy air block) and BlockTessellator.setRegion (the air fill) call getDefaultBlockState that way (`lea rcx,[rip+..]; lea rdx,[rip+air name]; mov r8b,1; call`). Both recipes read that lea target and subtract 18h; they are in two functions. Check section .data. Both recipes are now a pattern_any, one alternative per shape: alternative 0 (26.51) the getDefaultBlockState call (`lea rcx,[rip+registry]; lea rdx,[rip+air name]; mov r8b,1; call`); alternative 1 (26.60) the _lookupByNameImpl call that getDefaultBlockState inlines to (`lea rcx,[rip+registry]; lea r8,[rip+air name]; lea rdx,[rsp+..]; xor r9d,r9d; call`). Both read the lea target. What changed in 6ad79ee4-17d51891a1119914 (26.60): getDefaultBlockState is inlined (direct callers of _lookupByNameImpl went from 1545 to 5258), so the two 26.51 call sites changed shape. The Owner layout is the same: the code that stores the Owner vtable writes the global 18h before the registry on both builds (5 stores each). Would have survived with a better recipe."

  recipe {
    step "ref" { entry = "BlockTessellatorCache.getBlock" }
    step "pattern_any" {
      patterns = [
        "48 8D 0D ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 41 B0 ?? E8",
        "48 8D 0D ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 48 8D 54 24 ?? 45 31 C9 E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "add" { value = -24 }
  }

  recipe {
    step "ref" { entry = "BlockTessellator.setRegion" }
    step "pattern_any" {
      patterns = [
        "48 8D 0D ?? ?? ?? ?? 48 8D 15 ?? ?? ?? ?? 41 B0 ?? E8",
        "48 8D 0D ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 48 8D 54 24 ?? 45 31 C9 E8",
      ]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "add" { value = -24 }
  }

  check "section" { name = ".data" }
}

entry "BlockTypeRegistry.lookupByName" {
  kind      = "function"
  msvc_name = "?_lookupByNameImpl@BlockTypeRegistry@@QEBA?AULookupByNameImplReturnType@1@AEBVHashedString@@HW4LookupByNameImplResolve@1@_N@Z"
  notes     = "BlockTypeRegistry::LookupByNameImplReturnType BlockTypeRegistry::_lookupByNameImpl(HashedString const& name, int data, LookupByNameImplResolve resolve, bool logNotFound) const: rcx = the registry (BlockTypeRegistry.instance + 18h), rdx = the 16-byte result {BlockType*, Block*} (hidden return pointer), r8 = name, r9d = data, [rsp+20h] = resolve, [rsp+28h] = logNotFound at the call. It returns an empty result when the name text is empty (`cmp qword ptr [r8+18h],0`), then takes the registry lock (shared_ptr at this+178h/180h). getDefaultBlockState calls it with (name, 0, 0, false) and reads result.mBlockType+240h (header mDefaultState 576). Recipe 0 is the body head (`cmp qword ptr [r8+..],..; je; mov rdi,r8; mov [rbp+..],r9d; mov [rbp+..],rdx; xorps; movaps`), then function_start_of. Recipe 0 is a pattern_any on the body head, one alternative per shape (alternative 1, 26.60: `mov rsi,[r8+18h]; test rsi,rsi; je; mov rdi,r8; ..; mov [rbp+..],rcx; mov r12,[rcx]`), then function_start_of. Recipe 1 is a direct call with a static name and the registry: `mov dword ptr [rsp+20h],resolve; lea rcx,[rip+registry]; lea r8,[rip+name]; lea rdx,[rsp+..]; xor r9d,r9d; call; mov r14,[rsp+..]; test r14,r14; je` (the result BlockType test after it), read as a branch target; it runs past the call (no nop on either build). Check function_start. On 26.60 the inlined callers set only the resolve argument at [rsp+20h]; the logNotFound byte at [rsp+28h] is no longer written by them, and the header still lists it, so the mod still passes it. What changed in 6ad79ee4-17d51891a1119914 (26.60): getDefaultBlockState is inlined everywhere, so the structural route through it is gone, and the body tests the name length through rsi and no longer takes the registry lock first. The 26.60 function was accepted on two facts from the 26.51 function: the call from the BlockTessellatorCache.getBlock air path with the registry in rcx (inlined on 26.60), and the call site of recipe 1, drafted from a 26.51 caller, which matches once on each build and calls this function on both. Real code change (inlining and body); the signature is unchanged."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 83 78 ?? ?? 74 ?? 4C 89 C7 44 89 8D ?? ?? ?? ?? 48 89 95 ?? ?? ?? ?? 0F 57 C0 0F 29 85 ?? ?? ?? ??",
        "49 8B 70 ?? 48 85 F6 74 ?? 4C 89 C7 44 89 8D ?? ?? ?? ?? 48 89 95 ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 4C 8B 21",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "C7 44 24 ?? ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? 4C 8D 05 ?? ?? ?? ?? 48 8D 54 24 ?? 45 31 C9 E8 ?? ?? ?? ?? 4C 8B 74 24 ?? 4D 85 F6 0F 84"
      offset = 30
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
