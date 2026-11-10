entry "LightPropagationCoordinator.propagateForMesh" {
  kind   = "function"
  hooked = true
  notes  = "dynamic-lighting (Vibrant Visuals block light). bool LightPropagationCoordinator::propagateForMesh(LightPropagationCoordinator* self, BlockSource* src, SubChunkPos const* pos, uint16 variant, uint16 flags, std::function<...>* emissionProvider, FnRef* onResult, FnRef* onEmpty) (names guessed, ABI read from the code): rcx = self, rdx = src, r8 = pos (12 bytes, copied), r9w = variant, [rsp+28h] = flags (word), [rsp+30h] = provider (std::function, impl pointer at +38h), [rsp+38h] = onResult, [rsp+40h] = onEmpty (each a {fn, ctx} pair). It locks the weak store at self+20h/28h (returns 0 and calls nothing when the lock fails), calls the light spread core with a copy of the provider, then calls onResult.fn(ctx, result) or onEmpty.fn(ctx) and puts the result in the store keyed by (pos, variant, src->getDimensionId()); returns 1. RenderChunkBuilder::build (the mesher) calls it twice after tessellation: variant 0, flags 100h (the vertex block light rewrite through onResult), and variant 1 only when a render setting is on. Found from the mesher (dynamic-lighting docs/findings/vv-static-light-hook/report.md); checked here with inspect disasm and callers on every build. Recipe 0 is the body after the prologue (`mov rdx,[rcx+28h]; test; je; mov esi,r9d; mov rdi,r8; mov r13,[rbp+..]; movzx r15d,word [rbp+..]; mov eax,[rdx+8]`): the store weak_ptr lock and the word-sized 4th and 5th arguments; every displacement and the branch byte are wildcarded; then function_start_of. Recipe 1 is the variant 0 call in the mesher: three pointer stores to [rsp+..] (provider, onResult, onEmpty), `mov word [rsp+20h],imm16` (flags), `lea r8; mov rcx,rdi; mov rdx,[rbp+..]; xor r9d,r9d; call`, read as a branch target; every displacement and the flags immediate are wildcarded. Recipe 2 is the variant 1 call in the same mesher (`mov [rsp+..],rcx; mov word [rsp+20h],imm16; lea r8; mov rcx,rdi; mov rdx,[rbp+..]; mov r9w,imm16; call`), a second call site; it shares no bytes with recipe 1 but sits in the same caller. Check: function_start. There is no build gate: the 26.51 shape is excluded by the recipes themselves (see below), and the Preview 6ae4eeee-4dd4fa7acacb4c41 changed LevelChunk.mSubChunks (228h) without changing this function, so a LevelChunk gate would refuse a correct value there. Values: 26.60 0x5adb560, 26.62 0x5adbd20, 26.63 0x5adba80 (the body is the same instruction for instruction on the three builds). 26.51: refused on purpose. 26.51 has the same function at 0x46f1cb0 (called from its mesher at 0x1c98b8a and 0x1c98c05) but with another signature: 7 arguments, r9b = variant (a byte), [rsp+28h] = flags, [rsp+30h] = onResult, [rsp+38h] = onEmpty, no provider argument. A detour built for the 8-argument form would read onResult as the provider, so one name cannot mean both. No recipe matches 26.51: recipe 0 needs `mov esi,r9d` and the word load of the 5th argument, recipe 1 needs three stack pointer stores (26.51 makes two), recipe 2 needs `mov r9w,imm16` (26.51 passes a byte). On 26.51 the entry is reported as missing (no recipe matches), never as a wrong value. A 26.51 hook needs its own entry name. What changed in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): the function is the 26.63 body instruction for instruction (same size, same 8-argument use: word 4th and 5th arguments, provider, onResult, onEmpty), so recipe 0 matches unchanged. The mesher keeps the BlockSource-holder in rsi (`mov rcx,rsi`), so recipe 2 wildcards the ModRM byte of that move; and its variant 0 call passes onEmpty from a register (`mov [rsp+28h],r13`) instead of a third `lea rax; mov [rsp+..],rax` pair, so recipe 1 is now a pattern_any: alternative 0 the old shape, alternative 1 the Preview shape, which starts one instruction earlier (`mov [rbp+..],r12d`, the last word of the 12-byte position copy) so that the call lies at the same offset 63. The call arguments are the same as on 26.63 (r9d 0 / r9w 1 for the variant, word 100h at [rsp+20h], three pointers at [rsp+28h..38h], r8 the position copy), so the hook signature did not change. Evidence on the Preview: (1) the 26.63 body (recipe 0, drafted from 26.60) matches once and the function is the same instruction for instruction; (2) it is called twice by the mesher with the 26.63 argument setup (variant 0 with flags 100h, then variant 1). Class: would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 8B 51 ?? 48 85 D2 74 ?? 44 89 CE 4C 89 C7 4C 8B AD ?? ?? ?? ?? 44 0F B7 BD ?? ?? ?? ?? 8B 42 ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 66 C7 44 24 ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 48 89 F9 48 8B 95 ?? ?? ?? ?? 45 31 C9 E8",
        "44 89 A5 ?? ?? ?? ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 48 8D 85 ?? ?? ?? ?? 48 89 44 24 ?? 4C 89 ?? 24 ?? 66 C7 44 24 ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 48 89 ?? 48 8B 95 ?? ?? ?? ?? 45 31 C9 E8",
      ]
      offset = 63
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 4C 24 ?? 66 C7 44 24 ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 48 89 ?? 48 8B 95 ?? ?? ?? ?? 66 41 B9 ?? ?? E8"
      offset = 34
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

}
