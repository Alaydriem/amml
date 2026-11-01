entry "IMinecraftGame.grabMouse" {
  kind     = "vslot"
  notes    = "Slot of void IMinecraftGame::grabMouse() in the IMinecraftGame vtable: 135 in 26.51 and 26.60. LeviLamina's IMinecraftGame header order gives 141 (header order, not a reference); every IMinecraftGame slot checked is the header count less 6, so the header is not used as the value. Recipe 0 is the call in MinecraftGame::onAppFocusGained (AppPlatformListener slot 7; it sets rsi to the MinecraftGame, `lea rsi,[rcx-90h]` in 26.51 and `lea rsi,[rcx-88h]` in 26.60, because the AppPlatformListener subobject moved from +90h to +88h): when the window gets focus back and two conditions hold, `and bl,al; cmp bl,imm; jne; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]`; the displacement is turned into a slot. Recipe 1 is `ref` to the MinecraftGame.grabMouse entry and `slot_of` in the MinecraftGame.vtable global. MinecraftGame.grabMouse is found by its body and checked with a slot window, not by a fixed slot, so this recipe still detects a slot shift. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). Both agree in both builds. The range check keeps the slot near the input part of the vtable. Plan 5b replaced the two per-build body recipes (one per push list) by recipe 1."

  recipe {
    step "pattern" {
      bytes  = "20 C3 80 FB ?? 75 ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "ref" {
      entry = "MinecraftGame.grabMouse"
    }
    step "slot_of" {
      vtable = "MinecraftGame.vtable"
    }
  }

  check "range" {
    min = 125
    max = 145
  }
}

entry "IMinecraftGame.releaseMouse" {
  kind     = "vslot"
  notes    = "Slot of void IMinecraftGame::releaseMouse() in the IMinecraftGame vtable: 136 in 26.51 and 26.60. The header order gives 142 (header order, not a reference); see IMinecraftGame.grabMouse. The function runs only when the grabbed flag (1E8h in 26.51, 1F8h in 26.60) is 1 and clears it; in 26.51 it has a frame, in 26.60 it is a leaf without .pdata that tail-jumps through a member. Recipe 0 is the last call in MinecraftGame::onAppFocusLost: `ref` to the MinecraftGame.onAppFocusLost entry, then a pattern inside that function, `cmp reg,imm; je; mov rcx,rsi; call; mov rax,[rsi]; mov rax,[rax+disp]; mov rcx,rsi; call [guard]; nop; add rsp,..`, where rsi is the MinecraftGame; the displacement is turned into a slot. Recipe 1 is `ref` to MinecraftGame.grabMouse, `slot_of` in the MinecraftGame.vtable global and `add 1`: it relies on releaseMouse being declared right after grabMouse in IMinecraftGame (true in LeviLamina's header and in both builds). If a virtual is ever inserted between them, recipes 0 and 2 disagree and the entry is refused. Recipe 2 is a second call site: ClientInstance::grabMouse and ClientInstance::releaseMouse are 80-byte forwarders that follow each other in both builds (`push rsi; sub rsp,..; mov rsi,rcx; call [isPrimaryClient]; test al,al; je; mov rcx,[rsi+member]; mov rax,[rcx]; mov rax,[rax+disp]; mov rdx,[guard]; add rsp,..; pop rsi; jmp rdx`). The pattern is both forwarders with the 23 bytes between them wildcarded (26.51 has a `nop` in the else path that 26.60 does not, and the int3 count differs), and offset 0x50 + 38 reads the displacement of the second one, releaseMouse's 440h; the shape alone matches three forwarders (getMouseGrabbed, grabMouse, releaseMouse), the pair only once. It relies on ClientInstance's link order (grabMouse's forwarder 0x50 bytes before releaseMouse's), not on IMinecraftGame's declaration order. Every immediate, displacement and branch byte of the patterns is a wildcard (the lint is clean on 26.51 and 26.60). All three agree in both builds. Plan 5b replaced the two per-build body recipes (framed in 26.51, leaf in 26.60) by recipe 1, because no single pattern finds both bodies and the 26.60 leaf has no .pdata."

  recipe {
    step "ref" {
      entry = "MinecraftGame.onAppFocusLost"
    }
    step "pattern" {
      bytes  = "83 ?? ?? 74 ?? 48 89 F1 E8 ?? ?? ?? ?? 48 8B 06 48 8B 80 ?? ?? ?? ?? 48 89 F1 FF 15 ?? ?? ?? ?? 90 48 81 C4"
      offset = 16
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "ref" {
      entry = "MinecraftGame.grabMouse"
    }
    step "slot_of" {
      vtable = "MinecraftGame.vtable"
    }
    step "add" {
      value = 1
    }
  }

  recipe {
    step "pattern" {
      bytes  = "56 48 83 EC ?? 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 48 83 C4 ?? 5E 48 FF E2 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 56 48 83 EC ?? 48 89 CE 48 8B 01 48 8B 80 ?? ?? ?? ?? FF 15 ?? ?? ?? ?? 84 C0 74 ?? 48 8B 8E ?? ?? ?? ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8B 15 ?? ?? ?? ?? 48 83 C4 ?? 5E 48 FF E2"
      offset = 118
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 125
    max = 145
  }
}
