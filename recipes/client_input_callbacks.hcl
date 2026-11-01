entry "ClientInputCallbacks.handleBuildAction" {
  kind     = "function"
  hooked   = true
  notes    = "static bool ClientInputCallbacks::handleBuildAction(IClientInstance&, BuildActionIntention&, HitResult const& solid, HitResult const& liquid): rcx the client, rdx the intention, r8 and r9 the two hit results, the bool in al. Found from its callers: two input functions pass `Level::getHitResult()` (the Level slot read as Level.getHitResult) in r8, and a ClientInstance function passes the intention that ClientInstance::getInProgressBAI returns (the same member, `mov rdx,[rdi+290h]`) with two stack hit results. The body saves r9, r8 and rdx, reads the intention (`mov ebx,[rdx]`), calls a ClientInstance virtual, later tests BlockType::isInteractiveBlock on the hit block and calls GameMode::startBuildBlock (slot 5). Recipe 0 is the head of the body (`mov r13,r9; mov [rbp+..],r8; mov r14,rdx; mov rsi,rcx; mov ebx,[rdx]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]`), then function_start_of. Recipe 1 is the ClientInstance call site (`mov rdx,[rdi+..]; lea r8,[rbp+..]; lea r9,[rbp-..]; mov rcx,rdi; call`), read as the call target. The two recipes are in two functions. Every displacement, immediate and branch byte is a wildcard. The check is function_start, which recipe 1 can fail. Recipe 0 is now a pattern_any, one alternative per shape of the argument saves, both ending at the virtual call: alternative 0 (26.51) `mov r14,rdx; mov rsi,rcx; mov ebx,[rdx]; ..`; alternative 1 (26.60) `mov r14,r8; mov rsi,rcx; mov [rbp+..],rdx; mov ebx,[rdx]; ..`. What changed in 6ad79ee4-17d51891a1119914 (26.60): register allocation of the saved arguments (r9 goes to the frame, r8 to r14, rdx to the frame); the four arguments are the same registers, and recipe 1 (the ClientInstance call site) agrees. Would have survived with a better recipe."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 89 D6 48 89 CE 8B 1A 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15",
        "4D 89 C6 48 89 CE 48 89 95 ?? ?? ?? ?? 8B 1A 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15",
      ]
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 97 ?? ?? ?? ?? 4C 8D 85 ?? ?? ?? ?? 4C 8D 4D ?? 48 89 F9 E8"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
