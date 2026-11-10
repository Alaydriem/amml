entry "SurvivalMode.vtable" {
  kind     = "global"
  notes    = "The vtable of SurvivalMode (no RTTI), the GameMode subclass that the player gets in survival. The function that sets the player's game type (it replaces type 5, Default, with the level's default type through Level slot 140) allocates 110h bytes and calls the SurvivalMode constructor when the type is 0, else C8h bytes and the GameMode constructor, and stores the result in Player::mGameMode (+0AC8h). A helper for the attack hook: slot 15 of this table is a leaf that jumps past the base slot (see GameMode.attack). The class adds trial-mode state at +0C8h; its overrides test that flag and a global and else go to the base body: slots 1 and 11 jump to the base bodies, slots 2, 6, 12 and 14 are leaves that jump to them, slot 3 is the base body itself, slot 13 calls the base useItemOn body. 20 slots, in the GameMode order. Recipe 0 is the SurvivalMode constructor after its GameMode constructor call: `lea rax,[rip+..]; mov [rdi],rax; mov word ptr [rdi+..],..; mov dword ptr [rdi+..],..; lea rax,[rdi+..]` (the table, the trial flags, the timer); it reads the RIP target of the lea. Recipe 1 is the SurvivalMode destructor: its second half (`mov qword ptr [rsi+..],..; lea rax,[rip+..]; mov [rsi],rax; mov rcx,[rsi+..]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax]; mov edx,..; call [guard]`, the GameMode table store and the release of a member, ended at the call) finds the function, then a pattern in that function finds its first table store (`mov edi,edx; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax`), read as the RIP target. The head of the destructor alone matches the destructor of another class. The two recipes are in two functions. Every displacement, immediate and RIP offset is a wildcard. The check is section .rdata. Not derived on 26.51 or 26.60: those exes were not at hand when the entry was written."

  recipe {
    step "pattern" { bytes = "48 8D 05 ?? ?? ?? ?? 48 89 07 66 C7 87 ?? ?? ?? ?? ?? ?? C7 87 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 87 ?? ?? ?? ??" }
    step "operand" {
      index = 1
      kind  = "rip_target"
    }
  }

  recipe {
    step "pattern" { bytes = "48 C7 86 ?? ?? ?? ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 48 89 06 48 8B 8E ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 01 48 8B 00 BA ?? ?? ?? ?? FF 15" }
    step "pattern" {
      bytes  = "89 D7 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01"
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
