entry "ClientInputUpdateSystem.extractRawHIDInput" {
  kind   = "function"
  hooked = true
  notes  = "static void ClientInputUpdateSystem::extractRawHIDInput(MovementAbilitiesComponent const& rcx, MoveInputComponent const& rdx, ActorDataFlagComponent const& r8, RawMoveInputComponent& r9, Optional<SneakingComponent const> [rsp+28h], Optional<WasInWaterFlagComponent const> [rsp+30h]), void. 0x6d51230 on 6aed1906-b127418e74d2f2f9 (26.63), a .pdata function of 13Dh bytes; 0x6d514d0 on 6aea437e-7eff13daabb430da (26.62). Found from the entt type id of RawMoveInputComponent (FNV-1a 0x3613E513, `mov edx,3613E513h` at 6 sites): the virtual system tick (0x6d63500 on 26.63, its pointer is in .rdata) builds a view of MovementAbilitiesComponent (0x6DE78DC5), MoveInputComponent (0x018B1887), ActorDataFlagComponent (0xC67426F3), RawMoveInputComponent, LocalPlayerComponent (0xACB47B38) and PlayerInputRequestComponent (0x6B04A058), stores `lea rax,[rip+fn]` in a local and calls it for each entity through an each-invoker (guard_dispatch_icall). The invoker adds the two optionals from the SneakingComponent (0x8918332C) and WasInWaterFlagComponent (0x78E89F39) storages. The body copies the 0Fh-byte MoveInputState from MoveInputComponent+10h (mRawInputState) to RawMoveInputComponent.mInputState (dword, two floats, word, byte), then calls the move-vector function (0x493020 on 26.63, its only call) and stores the result at RawMoveInputComponent.mRawMove. It runs every tick and rebuilds the raw state from the key state each time, so a write after the original wins for that tick and a tick without a write gives the real keys again. It is called only through the function pointer, never inlined. Recipe 0 anchors on the copy (`mov r10d,[rdx+..]; mov [rsi],r10d; movss; movss; movss; movss; movzx r10d,word; mov word; movzx edx,byte; mov byte`), every displacement a wildcard, then function_start_of. Recipe 1 is the `lea rax,[rip+..]` in the tick, followed by the view setup (`mov [rbp+..],rax; mov qword ptr [rbp+..],0; lea rdi,[r14+..]; mov rax,[r14+..]; cmp rax,imm; je; cmp qword ptr [rdi+rax*8],0; jne; mov rsi,[r14+..]; add rsi,imm` and four `mov rcx,rsi; mov edx,typeid; call` blocks, ending at the fourth call), every immediate, displacement and branch byte a wildcard (the type ids too), read as a rip target. The two share no bytes and are in two functions; both derived on 26.62 and 26.63 with the same value. function_start confirms the start (recipe 1 can fail it). Prologue `push rsi; push rdi; push rbx; sub rsp,40h` (7 bytes, no rip-relative operand) takes an inline detour. signature: args_int rcx, rdx, r8, r9 and stack args 28h, 30h; it reports returns float only because xmm0 is live at the ret (the movsd store). In-game (26.63, a manager probe): a post-hook that writes RawMoveInputComponent moves the local player on a dedicated server, with the window in the background, while no menu screen is on top. msvc_name is left out: the mangled name was not confirmed. 26.60 and the 26.70 preview were not derived (no exe on the machine)."

  recipe {
    step "pattern" {
      bytes = "44 8B ?? ?? 44 89 ?? F3 0F 10 ?? ?? F3 0F 11 ?? ?? F3 0F 10 ?? ?? F3 0F 11 ?? ?? 44 0F B7 ?? ?? 66 44 89 ?? ?? 0F B6 ?? ?? 88 ?? ??"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "48 8D 05 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 49 8D 7E ?? 49 8B 86 ?? ?? ?? ?? 48 83 F8 ?? 74 ?? 48 83 3C C7 ?? 0F 85 ?? ?? ?? ?? 49 8B 76 ?? 48 83 C6 ?? 48 89 F1 BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 89 C3 48 89 F1 BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 49 89 C6 48 89 F1 BA ?? ?? ?? ?? E8 ?? ?? ?? ?? 49 89 C7 48 89 F1 BA ?? ?? ?? ?? E8"
    }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  check "function_start" {}
}
