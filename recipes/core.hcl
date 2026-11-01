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
