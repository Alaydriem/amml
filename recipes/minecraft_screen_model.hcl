entry "MinecraftScreenModel.getVersionString" {
  kind      = "function"
  hooked    = true
  msvc_name = "?getVersionString@MinecraftScreenModel@@QEBA?AV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@XZ"
  notes     = "std::string MinecraftScreenModel::getVersionString() const: rcx the model (not read), rdx the hidden result, returned in rax. It builds 'v' followed by the cached game version text (a thread-safe static) into the result: `mov byte ptr [rbx],76h; lea rcx,[rbx+1]; mov r8,rdi; call memcpy`. The loader hooks it to show `AMML / <instance folder>` in the version label of the start and pause screens. Its users are all UI: the #version binding of the start and pause screen controllers (a folded lambda `mov rcx,[rdx+8]; mov rcx,[rcx+0D48h]; jmp getVersionString`; the #development_version lambda after it jumps to getDevelopmentVersionString), the $version screen variable (a direct call on [this+0BF0h]) and the settings about panel (a direct call on [this+0D48h], with the options.version.format text). Found from the FNV-1a 32 hash of the binding name #version (C471CCDEh), stored before the bindString call in both controllers, then the lambda's call slot. The LeviLamina header names both getters (MinecraftScreenModel.h). Recipe 0 is the direct call in the $version setter: after the loop over the listeners (`cmp rbx,r14; jne`), `mov rcx,[rdi+..]; lea rdi,[rbp+..]; mov rdx,rdi; call`, read as the branch target. Recipe 1 is the direct call in the settings about panel: `movaps/movups [rbp+..],xmm6; mov rcx,[rsi+..]; mov byte ptr [rbp+..],..; lea rdx,[rbp+..]; call`, read the same way (the store opcode byte is a wildcard). Recipe 2 is the body: `cmp r12,..; lea rdx,[rip+..]; cmovae rdx,r15; mov [rsi+..],r13; mov [rsi+..],rax; mov byte ptr [rbx],..; lea rcx,[rbx+..]; mov r8,rdi; call`, then function_start_of. The recipes are in three functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. Check function_start. Values: 0x492ace0 on 6aa482fd-2658118eba0dbac5 (26.51), 0x4d539a0 on 6ad79ee4-17d51891a1119914 (26.60), 0x4d53b70 on 6aea437e-7eff13daabb430da (26.62), 0x4d538d0 on 6aed1906-b127418e74d2f2f9 (26.63); all three recipes agree on every build. Signature: rdx only (the result; rcx, the model, is not read)."

  recipe {
    step "pattern" {
      bytes  = "4C 39 F3 75 ?? 48 8B 8F ?? ?? ?? ?? 48 8D 7D ?? 48 89 FA E8"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F ?? 75 ?? 48 8B 8E ?? ?? ?? ?? C6 45 ?? ?? 48 8D 55 ?? E8"
      offset = 19
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes = "49 83 FC ?? 48 8D 15 ?? ?? ?? ?? 49 0F 43 D7 4C 89 6E ?? 48 89 46 ?? C6 03 ?? 48 8D 4B ?? 49 89 F8 E8"
    }
    step "function_start_of" {}
  }

  check "function_start" {}
}
