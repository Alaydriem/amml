entry "IOptionRegistry.setHideGUI" {
  kind     = "vslot"
  notes    = "Slot of void IOptionRegistry::setHideGUI(bool) in the IOptionRegistry vtable: 94 in 26.51 and 26.60, the same as LeviLamina's header order (header order, not a reference): IOptionsReader has 15 virtuals (0..14) and setHideGUI is the 80th new IOptionRegistry virtual after the destructor. In the OptionRegistry.vtable global, slot 94 gets the mutable option with id 1A5h and stores the bool from edx; 95 reads the same id (getHideHud); 93 and 96..98 are setDifficulty, getHideHand, setHideToolTips and getHideToolTips. Recipe 0 is the F1 toggle: on the IClientInstance, `options->setHideGUI(!client->getOptions()->getHideHud())`. The pattern runs from the load of the client (`mov rcx,[reg+member]`) through getOptions, getHideHud, `xor al,imm` to `mov r8,[rcx+disp]`; the registers holding the client and the options are wildcarded, and the setHideGUI displacement is turned into a slot. Four bytes are kept: the getHideHud vtable offset 2F8h, because the tool-tip and paper-doll toggles have the same code with other getters. Recipe 1 is the body of the setter (`push rsi; push rbx; sub rsp,..; mov ebx,edx; mov rsi,rcx`, stack cookie, the slot-27 call with an option id) and `slot_of` in the OptionRegistry.vtable global. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60) apart from the kept bytes. Both are unique in both exes and agree. What changed in 26.60: the F1 toggle is no longer a tail call (the registers are swapped and the call is plain); the pattern stops before that part."

  recipe {
    step "pattern" {
      bytes      = "48 8B ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8B 08 48 8B 91 F8 02 00 00 48 89 C1 48 89 D0 FF 15 ?? ?? ?? ?? 34 ?? 48 8B ?? 4C 8B 81"
      offset     = 47
      literal_ok = [
        "26: getHideHud vtable offset 2F8h: the tool-tip and paper-doll toggles have the same code with other getters (308h, 318h), so these bytes pick the F1 (hide GUI) toggle",
        "27: getHideHud vtable offset 2F8h: the tool-tip and paper-doll toggles have the same code with other getters (308h, 318h), so these bytes pick the F1 (hide GUI) toggle",
        "28: getHideHud vtable offset 2F8h: the tool-tip and paper-doll toggles have the same code with other getters (308h, 318h), so these bytes pick the F1 (hide GUI) toggle",
        "29: getHideHud vtable offset 2F8h: the tool-tip and paper-doll toggles have the same code with other getters (308h, 318h), so these bytes pick the F1 (hide GUI) toggle",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes = "56 53 48 83 EC ?? 89 D3 48 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? 41 B8 ?? ?? ?? ?? FF 15"
    }
    step "slot_of" {
      vtable = "OptionRegistry.vtable"
    }
  }

  check "range" {
    min = 80
    max = 110
  }
}

entry "IOptionRegistry.getHideHud" {
  kind     = "vslot"
  notes    = "Slot of bool IOptionRegistry::getHideHud() const in the IOptionRegistry vtable: 95 in 26.51 and 26.60, the same as the header order (header order, not a reference); see IOptionRegistry.setHideGUI for the neighbouring slots. Recipe 0 is a call site in the function that holds a NonOwnerPointer<IClientInstance> at [rdi+160h]: after the null-pointer trap, `mov rcx,[rdi+160h]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]` (getOptions) then `mov rcx,[rax]; mov rdx,[rcx+disp]; mov rcx,rax; mov rax,rdx` before the call; the getHideHud displacement is turned into a slot. Four bytes are kept: the member offset 160h, because on 26.51 six other getOptions-then-getter sites read the client from other members (26.60 is unique without these bytes). Recipe 1 is the body of the getter (`push rsi; sub rsp,..; mov rsi,rcx`, stack cookie, the IOptionsReader::get call with `mov r8d,1A5h`) and `slot_of` in the OptionRegistry.vtable global. Four bytes are kept: the option id 1A5h, because the getters of other options have the same code. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60) apart from the kept bytes. Both are unique in both exes and agree. The entry did not break in the 26.60 cold derive."

  recipe {
    step "pattern" {
      bytes      = "C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 8F 60 01 00 00 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8B 08 48 8B 91 ?? ?? ?? ?? 48 89 C1 48 89 D0"
      offset     = 37
      literal_ok = [
        "14: member 160h holds the NonOwnerPointer<IClientInstance> in this class; on 26.51 six other getOptions-then-getter sites read the client from other members (26.60 is unique without these bytes)",
        "15: member 160h holds the NonOwnerPointer<IClientInstance> in this class; on 26.51 six other getOptions-then-getter sites read the client from other members (26.60 is unique without these bytes)",
        "16: member 160h holds the NonOwnerPointer<IClientInstance> in this class; on 26.51 six other getOptions-then-getter sites read the client from other members (26.60 is unique without these bytes)",
        "17: member 160h holds the NonOwnerPointer<IClientInstance> in this class; on 26.51 six other getOptions-then-getter sites read the client from other members (26.60 is unique without these bytes)",
      ]
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern" {
      bytes      = "56 48 83 EC ?? 48 89 CE 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 01 48 8B 40 ?? 48 8D 54 24 ?? 41 B8 A5 01 00 00 FF 15"
      literal_ok = [
        "37: option id 1A5h (hide GUI) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "38: option id 1A5h (hide GUI) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "39: option id 1A5h (hide GUI) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "40: option id 1A5h (hide GUI) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
      ]
    }
    step "slot_of" {
      vtable = "OptionRegistry.vtable"
    }
  }

  check "range" {
    min = 80
    max = 110
  }
}

entry "IOptionRegistry.getViewDistanceChunks" {
  kind     = "vslot"
  notes    = "Slot of int IOptionRegistry::getViewDistanceChunks() const in the IOptionRegistry vtable: 59 on 26.51 (header 45 plus the 14 IOptionsReader virtuals before it, as for setHideGUI 94 = header 80 + 14). In the OptionRegistry.vtable global, slot 59 picks the option by graphics mode (a slot-74 test, then option ids 241h, 242h or 24h through IOptionsReader::get); 60 takes a GraphicsMode in dl (`cmp dl,2; cmp dl,3`, getDefaultViewDistanceChunks), 61 returns the back of a vector (getMaxViewDistanceChunksRaw) and 62 returns a float (getParticleViewDistance). Recipe 0 is the body of 59 (`mov rax,[rsi]; mov rax,[rax+8]; lea rdx,[rsp+..]; mov rcx,rsi; mov r8d,241h; call`), then function_start_of and slot_of OptionRegistry.vtable; the option id is kept. Recipe 1 is the leaf at 60 (`xor eax,eax; cmp dl,2; sete al; lea rax,[rax*8+..]; cmp dl,3`) and slot_of, minus 1. Range window 51..67."

  recipe {
    step "pattern" {
      bytes      = "48 8B 06 48 8B 40 ?? 48 8D 54 24 ?? 48 89 F1 41 B8 41 02 00 00 FF 15"
      literal_ok = [
        "17: option id 241h: the other option getters pass other ids",
        "18: option id 241h, second byte",
        "19: upper bytes of the option id",
        "20: upper bytes of the option id",
      ]
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "OptionRegistry.vtable" }
  }

  recipe {
    step "pattern" {
      bytes      = "31 C0 80 FA 02 0F 94 C0 48 8D 04 C5 ?? ?? ?? ?? 80 FA 03"
      literal_ok = [
        "4: GraphicsMode 2 in getDefaultViewDistanceChunks: tells it from other leaves that test dl",
        "18: GraphicsMode 3, the second mode the leaf tests",
      ]
    }
    step "slot_of" { vtable = "OptionRegistry.vtable" }
    step "add" { value = -1 }
  }

  check "range" {
    min = 51
    max = 67
  }
}

entry "IOptionRegistry.isGraphicsModeDeferred" {
  kind     = "vslot"
  notes    = "Slot of bool IOptionRegistry::isGraphicsModeDeferred() const (name from its behaviour, no header) in the OptionRegistry.vtable global: 75 on 26.60. It returns true when the GRAPHICS_MODE option (id 2BFh) holds GraphicsMode 2, the Deferred (Vibrant Visuals) mode: `get(2BFh)` through IOptionsReader slot 1, the override walk over [opt+8]+238h, then `cmp dword ptr [rax+18h],2; sete al`. GraphicsMode values from the graphics-mode radio table in the video settings screen (0x51cd350 on 26.60): #graphics_mode_radio_simple 0, _fancy 1, _deferred 2, _ray_traced 3. The option id name comes from the OptionID name table (string views, 16 bytes each; id 2BFh = GRAPHICS_MODE, 242h = DEFERRED_VIEW_DISTANCE, 241h = RAY_TRACING_VIEW_DISTANCE). It reads the option only: no device or platform check (see IOptionRegistry.isDeferredRenderingEnabled). Slot 74 is the ray-traced twin (a device check and mode 3), 76 is isDeferredRenderingEnabled, 77 returns the GRAPHICS_QUALITY_PRESET_MODE int (2C1h). Recipe 0 is the body (`sub rsp,..`, stack cookie, `mov rax,[rcx]; mov rax,[rax+8]; lea rdx,[rsp+..]; mov r8d,2BFh; call [guard]`, the override walk, `cmp dword ptr [rax+disp8],imm8; sete al`), then slot_of OptionRegistry.vtable; four bytes are kept, the option id 2BFh, because the getters of other options have the same code (0x5aff230 tests option 130h). Recipe 1 is the call site in isDeferredRenderingEnabled (slot 76): `mov ebx,eax; mov rax,[rsi]; mov rax,[rax+disp32]; mov rcx,rsi; call [guard]; mov esi,eax` after the device-capability call; it reads that displacement and turns it into a slot. The two recipes are in two functions. Every other immediate, displacement and branch byte is a wildcard. Both are unique in 26.60 and agree. 26.51 was not derived (no exe on this machine); treat the entry as optional. Value: slot 75 (0x5b6ede0) in 26.60."

  recipe {
    step "pattern" {
      bytes      = "48 83 EC ?? 48 8B 05 ?? ?? ?? ?? 48 31 E0 48 89 44 24 ?? 48 8B 01 48 8B 40 ?? 48 8D 54 24 ?? 41 B8 BF 02 00 00 FF 15 ?? ?? ?? ?? 48 8B 4C 24 ?? 48 89 C8 48 8B 49 ?? 48 8B 89 ?? ?? ?? ?? 48 85 C9 75 ?? 83 78 ?? ?? 0F 94 C0"
      literal_ok = [
        "33: option id 2BFh (GRAPHICS_MODE) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "34: option id 2BFh (GRAPHICS_MODE) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "35: option id 2BFh (GRAPHICS_MODE) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
        "36: option id 2BFh (GRAPHICS_MODE) passed to IOptionsReader::get: the getters of other options have the same code with other ids",
      ]
    }
    step "slot_of" { vtable = "OptionRegistry.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 89 CE 48 8B 81 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 80 38 ?? 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 89 C3 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 89 C6"
      offset = 60
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 65
    max = 85
  }
}

entry "IOptionRegistry.isDeferredRenderingEnabled" {
  kind     = "vslot"
  notes    = "Slot of bool IOptionRegistry::isDeferredRenderingEnabled() const (name from its behaviour, no header) in the OptionRegistry.vtable global: 76 on 26.60. It returns true only when all three hold: the device capability object at this+1B68h says deferred is supported (its vfn at 10h), isGraphicsModeDeferred (slot 75: GRAPHICS_MODE == 2, Deferred / Vibrant Visuals) and a ServiceLocator object (the global read under its lock at 0x128c3038..48, 261 references; its vfn 8 then vfn 230h) agrees. getViewDistanceChunks (slot 59) uses it to choose DEFERRED_VIEW_DISTANCE (242h); the ray-traced test before it (slot 74) picks RAY_TRACING_VIEW_DISTANCE (241h). The body first checks the NonOwnerPointer at this+1B58h and traps (`mov dword ptr [0],0DEADC0DEh`) when it is null or dead; slot 59 runs the same trap on every view-distance read, so during play it does not fire. Cost: three virtual calls, one lock and unlock and two atomic reference-count pairs. Recipe 0 is the body after the prologue (`mov rsi,rcx; mov rax,[rcx+disp32]; test rax,rax; je; cmp byte ptr [rax],imm; jne; mov dword ptr [0],imm; mov rcx,[rsi+disp32]; mov rax,[rcx]; mov rax,[rax+disp8]; call [guard]; mov ebx,eax; mov rax,[rsi]; mov rax,[rax+disp32]; mov rcx,rsi; call [guard]; mov esi,eax`), then function_start_of and slot_of OptionRegistry.vtable. Slot 74 has the same start but reads the next getter with a disp8 `mov rax,[rax+8]`, so the disp32 form tells the two apart without a kept byte. Recipe 1 is the call site in getViewDistanceChunks (slot 59, 0x5b6e950): `mov rax,[rsi]; mov rax,[rax+disp32]; mov rcx,rsi; call [guard]; test al,al; je; mov rax,[rsi]; mov rax,[rax+8]; lea rdx,[rsp+..]; mov rcx,rsi; mov r8d,imm; call`, offset 3, the displacement turned into a slot. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard; there are no kept bytes. Both are unique in 26.60 and agree. 26.51 was not derived (no exe on this machine); treat the entry as optional. Value: slot 76 (0x5b6eba0) in 26.60."

  recipe {
    step "pattern" {
      bytes = "48 89 CE 48 8B 81 ?? ?? ?? ?? 48 85 C0 0F 84 ?? ?? ?? ?? 80 38 ?? 75 ?? C7 04 25 ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? FF 15 ?? ?? ?? ?? 89 C3 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 89 C6"
    }
    step "function_start_of" {}
    step "slot_of" { vtable = "OptionRegistry.vtable" }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 06 48 8B 40 ?? 48 8D 54 24 ?? 48 89 F1 41 B8 ?? ?? ?? ?? FF 15"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 65
    max = 85
  }
}
