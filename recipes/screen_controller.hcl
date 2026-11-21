entry "ScreenController.dtor" {
  kind     = "function"
  hooked   = true
  notes    = "ScreenController::~ScreenController() body (not the deleting destructor in slot 0): rcx the controller, no other argument. It stores the ScreenController vtable at [rcx] and releases the members from +0BC0h down. The ScreenController deleting destructor (slot 0 of that table) calls it and then frees 0BC8h bytes; MinecraftScreenController's destructor ends with a tail jump to it, and the direct subclasses' destructors call it, so it runs for every screen controller, the HUD included (the HUD destructor ends in the MinecraftScreenController destructor). Recipe 0 is a block in the body that clears a std::string and tests a std::function (`movups [rbx],xmm0; mov qword ptr [rbx+..],..; mov rax,[rsi+..]; test rax,rax; je; lea rcx,[rsi+..]`), then function_start_of. Recipe 1 is a member list walk earlier in the body (`test rdi,rdi; je; lea r14,[rsi+..]; mov r15,[rsi+..]; cmp rdi,r15; jne`), then function_start_of. The anchors share no pattern bytes; the callers all have twins with the same shape, so no call site is unique. Every displacement, immediate and branch byte is a wildcard. The check is function_start. Recipe 0 now runs on past the std::function call to the next member release (`mov rcx,[rsi+..]; test rcx,rcx; je; mov rdx,[rsi+..]`); it runs past an indirect call (no nop on either build). What changed in 6ad79ee4-17d51891a1119914 (26.60): two other destructors got the same std::string and std::function block (3 matches); the longer pattern matches once on each build, and recipe 1 agrees. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "0F 11 03 48 C7 43 ?? ?? ?? ?? ?? 48 8B 86 ?? ?? ?? ?? 48 85 C0 74 ?? 48 8D 8E ?? ?? ?? ?? 48 8B 40 ?? FF 15 ?? ?? ?? ?? 48 8B 8E ?? ?? ?? ?? 48 85 C9 74 ?? 48 8B 96 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" { bytes = "48 85 FF 0F 84 ?? ?? ?? ?? 4C 8D B6 ?? ?? ?? ?? 4C 8B BE ?? ?? ?? ?? 4C 39 FF 0F 85 ?? ?? ?? ??" }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "ScreenController.isContainerScreen" {
  kind     = "vslot"
  notes    = "Slot of bool ScreenController::_isContainerScreen() const in the screen controller vtable: 38 on 26.60, 26.63 and 1.26.70.26 (header order: dtor 0, preFrameTick 1, tick 2). rcx the controller, the bool in al. The base returns false (`xor eax,eax; ret`) and ContainerScreenController.vtable holds `mov al,1; ret` there. Both bodies are shared by many tiny functions, so neither can be matched by its bytes; call it through the vtable and never hook it. Recipe 0 is a call site in the function that loads the string `RenderStage::render` and walks a vector of {controller*, flag} pairs: `mov [reg+..],al; mov rcx,[reg+reg]; mov rax,[rcx]; mov rax,[rax+..]; call [guard]; nop; test al,al; je`, read as the displacement of the virtual call and turned into a slot. The pattern starts at the store of the flag (`movzx eax,byte ptr [..+8]` before it is not in the pattern) and ends at the `je`; every REX, ModRM, displacement, immediate and branch byte except the fixed `8B 0C`, `48 8B 01` and `48 8B 80` of the call is a wildcard. The function after the `je` differs between builds, so the pattern stops there. Recipe 1 is the table: ContainerScreenController.vtable, the address of slot 38, the pointer in it, and the slot of that pointer in the same table (a fixed slot with a body check: the pointer in slot 38 is the only slot of the table with that function, so slot_of gives 38). The deriver refuses recipe 1 on its own (vtable-only rule); the entry resolves only when recipe 0 agrees. The check is a range window around the known slot. Concern: the owner class of the call site in recipe 0 is not named; the match with the table in recipe 1 is the evidence that slot 38 is the virtual it calls. What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the slot did not move; the function around the call site changed (a different register set and a different test after the call), so the old pattern, which ran on into it, had no match. Class: would have survived with a better recipe. Evidence for 70.26: (1) the string `RenderStage::render` is loaded by one function on 26.63 (the one with the old call site) and by one function on 70.26; (2) the pattern of recipe 0, drafted from the 26.63 call site, matches once on 26.63 and once on 70.26, in that string's function, with the same displacement (130h, slot 38). No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern" {
      bytes  = "88 ?? ?? ?? 8B 0C ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 90 84 C0 74 ??"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "ref" { entry = "ContainerScreenController.vtable" }
    step "vtable_slot" { slot = 38 }
    step "read_pointer" {}
    step "slot_of" { vtable = "ContainerScreenController.vtable" }
  }

  check "range" {
    min = 20
    max = 60
  }
}
