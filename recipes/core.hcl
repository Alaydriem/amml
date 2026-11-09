entry "core.operator_new" {
  kind      = "function"
  msvc_name = "??2@YAPEAX_K@Z"
  notes     = "The game's global operator new(size_t). It loads the global allocator object from a RIP-relative global, calls slot 1 of its vtable (allocate) through the CFG dispatch thunk, and on a null result for a non-zero size calls the out-of-memory handler. Found next to operator delete (reached from the sized delete thunk, the second most called target in a histogram of direct calls): it lies right before it and uses the same allocator global (0x26e810 in 26.51, 0x392920 in 26.60). The pattern is the body up to the first instruction of the out-of-memory path (`mov rcx,rsi`). Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60): the frame size, the allocator slot offset and the short branch distances. It is unique in both exes; `function_start` confirms the start. What changed in 26.60: the out-of-memory path no longer clears edx before the handler call; the pattern stops before it."

  recipe {
    step "pattern" {
      bytes = "56 48 83 EC ?? 48 89 CE 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 89 F2 FF 15 ?? ?? ?? ?? 48 85 F6 74 ?? 48 85 C0 74 ?? 48 83 C4 ?? 5E C3 48 89 F1"
    }
  }

  check "function_start" {}
}

entry "core.operator_delete" {
  kind      = "function"
  msvc_name = "??3@YAXPEAX@Z"
  notes     = "The game's unsized global operator delete(void*): if the pointer is not null it loads the global allocator object from a RIP-relative global and calls slot 2 of its vtable (release) through the CFG dispatch thunk. The sized operator delete(void*, size_t) of the static runtime is only a jmp to this function and has no unwind data, so the entry takes the unsized one; the loader passes a size that this overload ignores (x64). Found as the target of that sized thunk; it lies right after operator new and uses the same allocator global. The pattern is the whole body. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60): the frame size, the EH state constant -2, the release slot offset and the null-test branch. It is unique in both exes; `function_start` confirms the start. Checked in game by the loader's 64-byte heap round trip at load. What changed in 26.60: the null test jumps past the `nop` after the release call (+0Eh instead of +0Dh)."

  recipe {
    step "pattern" {
      bytes = "55 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CA 48 8B 0D ?? ?? ?? ?? 48 85 D2 74 ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 90 48 83 C4 ?? 5D C3"
    }
  }

  check "function_start" {}
}

entry "core.tls_index" {
  kind  = "global"
  notes = "The game's _tls_index (a u32 the Windows loader writes; the slot of the exe's TLS block in the gs:[58h] ThreadLocalStoragePointer array). Every thread_local access in the game reads it as `mov eax|ecx,[rip+..]; mov reg,gs:[58h]; mov reg,[reg+rax*8]`. Requested by dynamic-lighting to reach LightPropagation.tlsEmitterVector. Found by the requester (vv-hand-emitter report section 4) and checked here with inspect disasm. Recipe 0: ref LightPropagationCoordinator.propagateForMesh, call 0 (the light spread core), then the emitter vector reset `mov eax,[rip+..]; mov rcx,gs:[58h]; mov rcx,[rcx+rax*8]; lea reg,[rcx+..]; mov rax,[rcx+..]; cmp rax,[rcx+..]; je`, operand 1 rip_target. Recipe 1: ref MinecraftGame.endFrame, its thread-safe static guard `mov eax,[rip+..]; mov ecx,[rip+..]; mov rdx,gs:[58h]; mov rcx,[rdx+rcx*8]; cmp eax,[rcx+..]; mov [rbp+..],rdi`, offset 6 (the `mov ecx`), rip_target: another class and another function. Recipe 2: the spread core's init block `mov byte [rax+..],imm; lea rcx,[rip+dtor]; call`, the lea's rip_target (the TLS destructor of the emitter vector), then its first TLS load `mov eax,[rip+..]; mov rcx,gs:[58h]; mov rax,[rcx+rax*8]; mov rcx,[rax+..]`, rip_target. Every displacement, immediate and branch byte is wildcarded except the four bytes of gs:[58h] in each pattern (literal_ok, fixed by the Windows x64 ABI). Check: section .data. Values: 26.60 0x128747c8, 26.62 0x128767c8, 26.63 0x128767c8. 26.51 not looked at (out of scope). 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23): 0x18246a78 from recipes 0 and 2; recipe 1 does not run there because MinecraftGame.endFrame does not resolve on the Preview yet."

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes      = "8B 05 ?? ?? ?? ?? 65 48 8B 0C 25 58 00 00 00 48 8B 0C C1 48 8D ?? ?? ?? ?? ?? 48 8B 81 ?? ?? ?? ?? 48 3B 81 ?? ?? ?? ?? 74"
      literal_ok = ["11: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "12: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "13: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "14: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI"]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "ref" { entry = "MinecraftGame.endFrame" }
    step "pattern" {
      bytes      = "8B 05 ?? ?? ?? ?? 8B 0D ?? ?? ?? ?? 65 48 8B 14 25 58 00 00 00 48 8B 0C CA 3B 81 ?? ?? ?? ?? 48 89 BD"
      offset     = 6
      literal_ok = ["17: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "18: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "19: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "20: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI"]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "ref" { entry = "LightPropagationCoordinator.propagateForMesh" }
    step "call" { nth = 0 }
    step "pattern" {
      bytes  = "C6 80 ?? ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? E8"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
    step "pattern" {
      bytes      = "8B 05 ?? ?? ?? ?? 65 48 8B 0C 25 58 00 00 00 48 8B 04 C1 48 8B 88"
      literal_ok = ["11: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "12: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "13: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI", "14: gs:[58h] is the TEB slot that holds the thread-local storage array, fixed by the Windows x64 ABI"]
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" { name = ".data" }
}
