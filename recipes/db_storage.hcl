entry "DBStorage.vtable" {
  kind     = "global"
  notes    = "The primary vtable of DBStorage (no RTTI). Its slot 9 is getFullPath (`lea rax,[rcx+98h]; ret`, a leaf) and slot 19 is checkShutdownDone (DBStorage.checkShutdownDone), the LeviLamina LevelStorage header order. Recipe 0 is the DBStorage constructor (DBStorage.ctor): it saves r8 and rdx, stores this table at this+0, takes this+8, zeroes +8 to +18h and allocates the 90h-byte leveldb Options (`mov [rbp+..],r8; mov [rbp+..],rdx; lea rax,[rip+..]; mov [rcx],rax; lea rax,[rcx+8]; ...; mov edx,90h; call [guard]`). Recipe 1 is the deleting destructor: it stores the table and calls checkShutdownDone directly (`mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; call; mov al,1; xchg [rsi+..],al`). Both read the RIP target of the lea. These are its only two references. Every displacement, immediate and branch byte is a wildcard. The section check keeps the address inside .rdata. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 1 is now a pattern_any with the nop alternative after the base destructor call. Recipe 0 still matched. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "pattern" {
      bytes  = "4C 89 85 ?? ?? ?? ?? 48 89 95 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 41 ?? 48 89 85 ?? ?? ?? ?? 0F 57 C0 0F 11 41 ?? 48 89 8D ?? ?? ?? ?? 48 C7 41 ?? ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? BA ?? ?? ?? ?? FF 15"
      offset = 14
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 E8 ?? ?? ?? ?? B0 ?? 86 86", "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 E8 ?? ?? ?? ?? 90 B0 ?? 86 86"]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "section" {
    name = ".rdata"
  }
}

entry "DBStorage.ctor" {
  kind     = "function"
  hooked   = true
  notes    = "DBStorage::DBStorage(DBStorageConfig config, Bedrock::NotNullNonOwnerPtr<LevelDbEnv> levelDbEnv). Detour shape `(this, config_ptr, env_ptr) -> this`: rcx = this, rdx = a pointer to the DBStorageConfig and r8 = a pointer to the NotNullNonOwnerPtr (both by-value arguments larger than 8 bytes, so MSVC passes a pointer to a caller copy). The prologue saves r8 and rdx (`mov [rbp+730h],r8; mov [rbp+830h],rdx`) and later reads [r8+8] (the control block of the pointer) and config fields through rdx; the three callers pass `lea rdx,[rbp+..]` and `lea r8,[rbp+..]` (two stack copies) and do not read rax after the call (an MSVC constructor returns this in rax). Recipe 0 is the only function that loads the string `DBStorage::DBStorage`. Recipe 1 is a direct call (`lea rdx,[rbp+..]; lea r8,[rbp+..]; mov rcx,rbx; mov [rbp+..],rbx; call; mov rcx,[rbp+..]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rbp+..]`), read as a branch target. The recipes share no anchor. Every displacement, immediate and branch byte is a wildcard. Check function_start. Grade: two recipes only. Hooked: `signature` probes it. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 1 is now a pattern_any with the nop alternative (the call is read before the nop). Recipe 0 still matched. Class: would have survived with a pattern that ends at the call."

  recipe {
    step "string_xref" { text = "DBStorage::DBStorage" }
  }

  recipe {
    step "pattern_any" {
      patterns = ["48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? 48 89 D9 48 89 9D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55", "48 8D 55 ?? 4C 8D 85 ?? ?? ?? ?? 48 89 D9 48 89 9D ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8B 8D ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 55"]
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "DBStorage.checkShutdownDone" {
  kind     = "function"
  hooked   = true
  notes    = "bool DBStorage::checkShutdownDone(), DBStorage.vtable slot 19. Detour shape `(this) -> bool`: rcx = this and nothing else is read before it is set (the prologue is `push rsi; push rdi; push rbx; sub rsp,80h; test byte ptr [rcx+1BBh],1`); the result is the low byte of eax (`mov eax,edi` with dil 0 or 1). The first test reads +1BBh (mShutdownStarted by header order); mShutdownDone is +1BDh. Recipe 0 is the start of the body (`test byte ptr [rcx+disp],imm; je; mov rsi,rcx; mov rcx,[rcx+disp]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax+disp]; call [guard]`), then function_start_of. Recipe 1 is the direct call in the DBStorage deleting destructor (DBStorage.vtable recipe 1), read as a branch target; that is its only direct caller. Checks: function_start, and in_vtable of DBStorage.vtable in slots 15 to 23 (the known slot 19, four either side). Hooked: `signature` probes it. What changed in 6ad79ee4-17d51891a1119914 (26.60), repaired in Plan 6c Task 8: the compiler now puts a nop after many calls, so an old pattern that ran past a call stopped matching. Recipe 1 (the DBStorage destructor call) is now a pattern_any with the nop alternative (the call is read before the nop). Recipe 0 is now a pattern_any that ends before the first virtual call: `test byte ptr [rcx+..],1; je` (26.51) or `movzx eax,byte ptr [rcx+..]; test al,1; je` (26.60), then `mov rsi,rcx; mov rcx,[rcx+..]; test rcx,rcx; je`, then function_start_of. Evidence on 26.60: the destructor call, drafted from the old destructor, goes to the candidate; the candidate tests the same byte (1BBh, mShutdownStarted) and the same member at +48h as the old body. Class: real code change (flag test encoding, virtual call through a sub-object)."

  recipe {
    step "pattern_any" {
      patterns = ["F6 81 ?? ?? ?? ?? ?? 0F 84 ?? ?? ?? ?? 48 89 CE 48 8B 49 ?? 48 85 C9 74", "0F B6 81 ?? ?? ?? ?? A8 ?? 0F 84 ?? ?? ?? ?? 48 89 CE 48 8B 49 ?? 48 85 C9 74"]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 E8 ?? ?? ?? ?? B0 ?? 86 86", "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 E8 ?? ?? ?? ?? 90 B0 ?? 86 86"]
      offset = 15
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

  check "in_vtable" {
    vtable   = "DBStorage.vtable"
    min_slot = 15
    max_slot = 23
  }
}
